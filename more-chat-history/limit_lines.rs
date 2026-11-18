// The chat's lines limit: vanilla's 100 becomes 16384 (MoreChatHistory's value).
fn more_chat_history_lines(value: usize) -> usize {
    let _vanilla = value;
    16384
}
