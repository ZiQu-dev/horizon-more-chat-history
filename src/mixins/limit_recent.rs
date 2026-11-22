// Chat::add_recent as in Horizon 39f1a51b3c, with vanilla's 100 remembered sent messages raised to 16384.
// The saved command history (command_history.txt) keeps 1000 commands instead of 50.
fn more_chat_history_recent(chat: &mut Chat, message: &str) -> ::core::ops::ControlFlow<()> {
    const LIMIT: usize = 16384;
    if chat.recent.last().map(String::as_str) != Some(message) {
        if chat.recent.len() >= LIMIT {
            chat.recent.remove(0);
        }
        chat.recent.push(message.to_owned());
    }
    if message.starts_with('/') && chat.commands.last().map(String::as_str) != Some(message) {
        if chat.commands.len() >= 1000 {
            chat.commands.remove(0);
        }
        chat.commands.push(message.to_owned());
        if let Some(file) = &chat.history_file {
            let mut text = chat.commands.join("\n");
            text.push('\n');
            if let Err(e) = std::fs::write(file, text) {
                log::error!(
                    "Failed to write {COMMAND_HISTORY_FILE}, command history will be missing: {e}"
                );
            }
        }
    }
    ::core::ops::ControlFlow::Break(())
}
