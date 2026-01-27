/// POP3 message info
pub struct Pop3MessageInfo {
    /// numerical Id of the message used for various commands
    pub message_id: u32,

    /// size of the message in bytes
    pub message_size: u32,
}
