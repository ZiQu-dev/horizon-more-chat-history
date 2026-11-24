// Chat::push_message as in Horizon 39f1a51b3c, with vanilla's 100 kept messages raised to 16384.
fn more_chat_history_messages(
    chat: &mut Chat,
    font: &crate::font::Font,
    message: &Message,
    focused: bool,
) -> ::core::ops::ControlFlow<()> {
    const LIMIT: usize = 16384;
    // The original body follows a Break, so the message is cloned, not moved.
    let mut message = message.clone();
    if !chat.abilities.shows(message.source) {
        return ::core::ops::ControlFlow::Break(());
    }
    let plain = message.content.plain();
    // Chat kept between sessions: one line per message in `chat_history.log` next to
    // `command_history.txt`, as `hour<TAB>minute<TAB>text`. The earlier sessions' messages are
    // appended below the live ones the first time a message comes in after the first ten seconds
    // (their lines are drawn like new ones for 200 ticks, so earlier would flash them), or when
    // the chat is open. Colours and click events are not kept, only the text.
    static SAVED: ::core::sync::atomic::AtomicUsize = ::core::sync::atomic::AtomicUsize::new(0);
    static RESTORED: ::core::sync::atomic::AtomicBool = ::core::sync::atomic::AtomicBool::new(false);
    // Single player runs a second Chat on the "Server thread" that nobody sees and that gets the
    // same messages: only the window's own, on the main thread, keeps and restores the history.
    let log_path = chat
        .history_file
        .as_ref()
        .filter(|_| std::thread::current().name() == Some("main"))
        .and_then(|file| file.parent())
        .map(|dir| dir.join("chat_history.log"));
    // The settings page's values (Options > Mods) as the mod's hooks write them to
    // `horizon-mod-data/more_chat_history/settings.txt`: `save_history=false` turns the saving
    // and restoring off, `max_file_mib` caps the log (only its newest part is read and kept).
    // Defaults until the file exists. Read for each message: it is two short lines.
    let (save, cap) = match &log_path {
        Some(path) => {
            let file = path
                .with_file_name("horizon-mod-data")
                .join("more_chat_history")
                .join("settings.txt");
            let (mut save, mut mib) = (true, 4u64);
            if let Ok(text) = std::fs::read_to_string(&file) {
                for line in text.lines() {
                    match line.split_once('=') {
                        Some(("save_history", v)) => save = v.trim() != "false",
                        Some(("max_file_mib", v)) => {
                            mib = v.trim().parse().unwrap_or(4).clamp(1, 256)
                        }
                        _ => {}
                    }
                }
            }
            (save, mib * 1024 * 1024)
        }
        None => (false, 0),
    };
    let log_path = log_path.filter(|_| save);
    if let Some(path) = &log_path
        && (focused || message.added >= 200)
        && !RESTORED.swap(true, ::core::sync::atomic::Ordering::Relaxed)
        && let Ok(mut file) = std::fs::File::open(path)
    {
        // Only the last `cap` bytes: a log that grew large is never read whole.
        use std::io::{Read, Seek};
        let len = file.metadata().map(|m| m.len()).unwrap_or(0);
        let start = len.saturating_sub(cap);
        let mut bytes = Vec::new();
        let _ = file.seek(std::io::SeekFrom::Start(start));
        let _ = file.take(cap).read_to_end(&mut bytes);
        let mut raw = String::from_utf8_lossy(&bytes).into_owned();
        if start > 0 {
            // The first line is cut in the middle.
            raw = raw.split_once('\n').map(|(_, rest)| rest.to_owned()).unwrap_or_default();
        }
        let all: Vec<&str> = raw.lines().collect();
        let keep = all.len().saturating_sub(SAVED.load(::core::sync::atomic::Ordering::Relaxed));
        let from = keep.saturating_sub(LIMIT);
        let mut earlier: Vec<Message> = Vec::new();
        for entry in all[from..keep].iter().rev() {
            let mut parts = entry.splitn(3, '\t');
            let (Some(h), Some(m), Some(text)) = (parts.next(), parts.next(), parts.next()) else {
                continue;
            };
            let (Ok(h), Ok(m)) = (h.parse::<u8>(), m.parse::<u8>()) else {
                continue;
            };
            let mut plain_text = String::new();
            let mut chars = text.chars();
            while let Some(c) = chars.next() {
                if c == '\\' {
                    match chars.next() {
                        Some('n') => plain_text.push('\n'),
                        Some(other) => plain_text.push(other),
                        None => {}
                    }
                } else {
                    plain_text.push(c);
                }
            }
            earlier.push(Message {
                added: 0,
                time: (h, m),
                repeats: 1,
                wrapped: 0,
                content: Text::literal(plain_text).with_color(text::GRAY),
                tag: None,
                source: ChatSource::Player,
                signature: None,
            });
        }
        if !earlier.is_empty() {
            // Newest first, like the list: the divider is newer than everything it separates.
            earlier.insert(
                0,
                Message {
                    added: 0,
                    time: (0, 0),
                    repeats: 1,
                    wrapped: 0,
                    content: Text::translate_with_fallback(
                        "more_chat_history.earlier",
                        "--- earlier sessions ---",
                        &[],
                    )
                    .with_color(text::DARK_GRAY),
                    tag: None,
                    source: ChatSource::Client,
                    signature: None,
                },
            );
            for mut old in earlier {
                let count = chat.add_lines(font, &old, false);
                // add_lines caps the list, so a message that wrapped to more lines than the cap leaves fewer.
                let block: Vec<Line> = chat.lines.drain(..count.min(chat.lines.len())).collect();
                chat.lines.extend(block);
                old.wrapped = count;
                chat.messages.push(old);
            }
        }
    }
    crate::logging::write_only(log::Level::Info, &format!("[CHAT] {plain}"));
    if chat.log {
        println!("[CHAT] {plain}");
    }
    message.time = chat.now();
    if let Some(path) = &log_path {
        use std::io::Write;
        let (h, m) = message.time;
        // A message of tens of thousands of lines is not worth keeping whole.
        let text = plain
            .chars()
            .take(8192)
            .collect::<String>()
            .replace('\\', "\\\\")
            .replace('\n', "\\n");
        if let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open(path) {
            if writeln!(file, "{h}\t{m}\t{text}").is_ok() {
                SAVED.fetch_add(1, ::core::sync::atomic::Ordering::Relaxed);
            }
        }
        // A chat-spamming server must not grow the log without limit: every 256 messages, a log
        // over twice the cap is cut to its newest `cap` bytes (whole lines).
        if SAVED.load(::core::sync::atomic::Ordering::Relaxed) % 256 == 0
            && let Ok(meta) = std::fs::metadata(path)
            && meta.len() > cap * 2
            && let Ok(mut file) = std::fs::File::open(path)
        {
            use std::io::{Read, Seek};
            let mut bytes = Vec::new();
            let _ = file.seek(std::io::SeekFrom::Start(meta.len() - cap));
            let _ = file.take(cap).read_to_end(&mut bytes);
            let raw = String::from_utf8_lossy(&bytes).into_owned();
            if let Some((_, rest)) = raw.split_once('\n') {
                let _ = std::fs::write(path, rest);
            }
        }
    }
    chat.received += 1;
    if chat.qol.compact
        && let Some(last) = chat.messages.first()
        && repeats(last, &message)
    {
        let mut last = chat.messages.remove(0);
        let shown = last.wrapped.min(chat.lines.len());
        chat.lines.drain(..shown);
        let (scroll, new_since_scroll) = (chat.scroll, chat.new_since_scroll);
        last.repeats += 1;
        last.added = message.added;
        last.time = message.time;
        last.wrapped = chat.add_lines(font, &last, focused);
        if scroll > 0 {
            chat.scroll = scroll;
            chat.new_since_scroll = new_since_scroll;
            chat.scroll_by(last.wrapped as i32 - shown as i32, focused);
        }
        chat.messages.insert(0, last);
        return ::core::ops::ControlFlow::Break(());
    }
    message.wrapped = chat.add_lines(font, &message, focused);
    chat.messages.insert(0, message);
    chat.messages.truncate(LIMIT);
    ::core::ops::ControlFlow::Break(())
}
