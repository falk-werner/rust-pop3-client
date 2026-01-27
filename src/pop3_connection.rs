use std::io::Write;
use std::error::Error;

use crate::Pop3Stat;
use crate::Pop3MessageInfo;
use crate::Pop3MessageUidInfo;

pub trait Pop3Connection {
    
    /// Authenticate a POP3 session using username and password.
    ///
    /// This is usually the first set of commands after a POP3 session
    /// is established.
    ///
    /// # Arguments
    ///
    /// * `user`     - Name of the user, typically it's e-mail address.
    /// * `password` - Password of the user. 
    fn login(&mut self, user: &str, password: &str) -> Result<(), Box<dyn Error>>;

    /// Returns maildrop statistics.
    fn stat(&mut self) -> Result<Pop3Stat, Box<dyn Error>>;

    /// Returns id and size of each message.
    fn list(&mut self) -> Result<Vec<Pop3MessageInfo>, Box<dyn Error>>;

    /// Returns the size of a given message.
    ///
    /// # Arguments
    ///
    /// * `message_id` - id of the message to query
    fn get_message_size(&mut self, message_id: u32) -> Result<u32, Box<dyn Error>>;

    /// Downloads a given message.
    ///
    /// # Arguments
    ///
    /// * `message_id` - id of the message to download
    /// * `writer`     - writer to store message
    fn retrieve(&mut self, message_id: u32, writer: &mut dyn Write) -> Result<(), Box<dyn Error>>;

    /// Deletes a given message.
    ///
    /// # Arguments
    ///
    /// * `message_id` - id of the message to download
    fn delete(&mut self, message_id: u32) -> Result<(), Box<dyn Error>>;

    /// Unmark any messages marked as delete.
    fn reset(&mut self) -> Result<(), Box<dyn Error>>;

    /// Returns the message header an a given number of lines from the message.
    ///
    /// # Arguments
    ///
    /// * `message_id` - id of the message
    /// * `line_count` - count of lines to return from the message body
    fn top(&mut self, message_id: u32, line_count: u32) -> Result<String, Box<dyn Error>>;

    /// Returns the unique ids of all messages.
    fn list_unique_ids(&mut self) -> Result<Vec<Pop3MessageUidInfo>, Box<dyn Error>>;

    /// Returns the unique id of a given message.
    ///
    /// # Arguments
    ///
    /// * `message_id` - id of the message
    fn get_unique_id(&mut self, message_id :u32) -> Result<String, Box<dyn Error>>;
}
