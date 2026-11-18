// The chat's recent limit: vanilla's 100 becomes 16384 (MoreChatHistory's value).
fn more_chat_history_recent(value: usize) -> usize {
    let _vanilla = value;
    16384
}
