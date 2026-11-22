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
    if let Some(path) = &log_path
        && (focused || message.added >= 200)
        && !RESTORED.swap(true, ::core::sync::atomic::Ordering::Relaxed)
        && let Ok(raw) = std::fs::read_to_string(path)
    {
        let all: Vec<&str> = raw.lines().collect();
        if all.len() > LIMIT * 2 {
            let _ = std::fs::write(path, all[all.len() - LIMIT..].join("\n") + "\n");
        }
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
                let block: Vec<Line> = chat.lines.drain(..count).collect();
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
        let text = plain.replace('\\', "\\\\").replace('\n', "\\n");
        if let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open(path) {
            if writeln!(file, "{h}\t{m}\t{text}").is_ok() {
                SAVED.fetch_add(1, ::core::sync::atomic::Ordering::Relaxed);
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
