/// POP3 message unique id info
pub struct Pop3MessageUidInfo {
    /// numerical Id of the message used for various commands
    pub message_id: u32,

    // unique id of the message
    pub unique_id: String,
}
