mod line_reader;

use std::sync::Arc;
use std::net::TcpStream;
use std::error::Error;
use std::io::{Write};

use rustls::{RootCertStore, ClientConnection, StreamOwned};

use line_reader::LineReader;

/// POP3 connection
pub struct Pop3Connection {    
    tls: StreamOwned<ClientConnection, TcpStream>,
    reader: LineReader,
}

/// POP3 maildrop statistics
pub struct Pop3Stat {
    /// count of massages in the maildrop
    pub message_count: u32,

    /// size of the maildrop in bytes
    pub maildrop_size: u32,
}

/// POP3 message info
pub struct Pop3MessageInfo {
    /// numerical Id of the message used for various commands
    pub message_id: u32,

    /// size of the message in bytes
    pub message_size: u32,
}

/// POP3 message unique id info
pub struct Pop3MessageUidInfo {
    /// numerical Id of the message used for various commands
    pub message_id: u32,

    // unique id of the message
    pub unique_id: String,
}

impl Pop3Connection {

    /// Returns a new POP3 connection.
    ///
    /// # Arguments
    ///
    /// * `host` - IP-Address or host name of the POP3 server to connect
    /// * `port` - Port of the POP3 server to connect
    pub fn new(host: &str, port: u16) -> Result<Pop3Connection, Box<dyn Error>> {
        let mut root_store = RootCertStore::empty();
        for cert in rustls_native_certs::load_native_certs()? {
            root_store.add(&rustls::Certificate(cert.0))?;
        }

        Pop3Connection::with_custom_certs(host, port, root_store)
    }

    /// Returns a new POP3 connection with custom certificates.
    ///
    /// # Arguments
    ///
    /// * `host` - IP-Address or host name of the POP3 server to connect
    /// * `port` - Port of the POP3 server to connect
    /// * `root_store` - Store of trusted (root) certificates.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_pop3_client::Pop3Connection;
    /// use rustls::RootCertStore;
    ///
    /// let mut root_store = RootCertStore::empty();
    /// for cert in rustls_native_certs::load_native_certs().unwrap() {
    ///     root_store.add(&rustls::Certificate(cert.0)).unwrap();
    /// }
    /// 
    /// let connection = Pop3Connection::with_custom_certs("", 995, root_store);
    /// ```
    pub fn with_custom_certs(host: &str, port: u16, root_store: RootCertStore) -> Result<Pop3Connection, Box<dyn Error>> {
        let config = rustls::ClientConfig::builder()
            .with_safe_defaults()
            .with_root_certificates(root_store)
            .with_no_client_auth();

        let server_name = host.try_into()?;

        let connection = rustls::ClientConnection::new(Arc::new(config), server_name)?;
        let stream =  TcpStream::connect(format!("{}:{}", host, port))?;
        let tls = rustls::StreamOwned::new(connection, stream);

        let mut client = Pop3Connection { 
            tls: tls,
            reader: LineReader::new()
        };

        client.read_status_line()?;
        Ok(client)
    }

    fn read_status_line(&mut self) -> Result<String, Box<dyn Error>> {
        let line = self.reader.read_line(&mut self.tls)?;

        match line.starts_with("+OK") {
            true => Ok(line),
            _ => Err(line.into())
        }
    }

    fn invoke_single_line(&mut self, command: &str) -> Result<String, Box<dyn Error>> {
        self.tls.write(command.as_bytes())?;
        self.read_status_line()
    }

    fn invoke_multi_line(&mut self, command: &str) -> Result<Vec<String>, Box<dyn Error>> {
        self.tls.write(command.as_bytes())?;
        self.read_status_line()?;

        let mut response : Vec<String> = vec!();
        loop {
            let line = self.reader.read_line(&mut self.tls)?;
            match line {
                _ if line == "." => { break },
                _ if line.starts_with(".") => { response.push(line[1..].to_string()); },
                _ => { response.push(line); }
            };
        }

        Ok(response)
    }

    /// Authenticate a POP3 session using username and password.
    ///
    /// This is usually the first set of commands after a POP3 session
    /// is established.
    ///
    /// # Arguments
    ///
    /// * `user`     - Name of the user, typically it's e-mail address.
    /// * `password` - Password of the user. 
    pub fn login(&mut self, user: &str, password: &str) -> Result<(), Box<dyn Error>> {
        self.invoke_single_line(&format!("USER {}\r\n", user))?;
        self.invoke_single_line(&format!("PASS {}\r\n", password))?;
        Ok(())
    }

    /// Returns maildrop statistics.
    pub fn stat(&mut self) -> Result<Pop3Stat, Box<dyn Error>> {
        let stat = self.invoke_single_line("STAT\r\n")?;
        let mut stat = stat.split(' ');
        let _ = stat.next();
        let message_count = stat.next().ok_or("missing message count")?;
        let message_count = message_count.parse::<u32>()?;
        let maildrop_size = stat.next().ok_or("missing maildrop size")?;
        let maildrop_size = maildrop_size.parse::<u32>()?;

        Ok(Pop3Stat { message_count: message_count, maildrop_size: maildrop_size })
    }

    /// Returns id and size of each message.
    pub fn list(&mut self) -> Result<Vec<Pop3MessageInfo>, Box<dyn Error>> {
        let lines = self.invoke_multi_line("LIST\r\n")?;
        let mut result = vec!();
        for line in lines {
            let mut info = line.split(' ');
            let message_id = info.next().ok_or("missing id")?.parse::<u32>()?;
            let message_size = info.next().ok_or("missing size")?.parse::<u32>()?;

            result.push(Pop3MessageInfo { 
                message_id: message_id, 
                message_size: message_size
            });
        }

        Ok(result)
    }

    /// Returns the size of a given message.
    ///
    /// # Arguments
    ///
    /// * `message_id` - id of the message to query
    pub fn get_message_size(&mut self, message_id: u32) -> Result<u32, Box<dyn Error>> {
        let line = self.invoke_single_line(&format!("LIST {}\r\n", message_id))?;
        let mut info = line.split(' ');
        let _ = info.next();    // skip "+OK"
        let _ = info.next();    // skip message id
        let message_size = info.next().ok_or("missing size")?.parse::<u32>()?;
     
        Ok(message_size)
    }

    /// Downloads a given message.
    ///
    /// # Arguments
    ///
    /// * `message_id` - id of the message to download
    /// * `writer`     - writer to store message
    pub fn retrieve(&mut self, message_id: u32, writer: &mut impl Write) -> Result<(), Box<dyn Error>> {
        let lines = self.invoke_multi_line(&format!("RETR {}\r\n", message_id))?;
        for line in lines {
            writer.write(line.as_bytes())?;
            writer.write(b"\n")?;
        }

        Ok(())
    }

    /// Deletes a given message.
    ///
    /// # Arguments
    ///
    /// * `message_id` - id of the message to download
    pub fn delete(&mut self, message_id: u32) -> Result<(), Box<dyn Error>> {
        self.invoke_single_line(&format!("DELE {}\r\n", message_id))?;
        Ok(())
    }

    /// Unmark any messages marked as delete.
    pub fn reset(&mut self) -> Result<(), Box<dyn Error>> {
        self.invoke_single_line("RSET\r\n")?;
        Ok(())
    }

    /// Returns the message header an a given number of lines from the message.
    ///
    /// # Arguments
    ///
    /// * `message_id` - id of the message
    /// * `line_count` - count of lines to return from the message body
    pub fn top(&mut self, message_id: u32, line_count: u32) -> Result<String, Box<dyn Error>> {
        let lines = self.invoke_multi_line(&format!("TOP {} {}\r\n", message_id, line_count))?;
        let mut message = String::new();
        for line in lines {
            message.push_str(&line);
            message.push('\n');
        }

        Ok(message)
    }

    /// Returns the unique ids of all messages.
    pub fn list_unique_ids(&mut self) -> Result<Vec<Pop3MessageUidInfo>, Box<dyn Error>> {
        let lines = self.invoke_multi_line("UIDL\r\n")?;
        let mut result = vec!();

        for line in lines {
            let mut info = line.split(' ');
            let message_id = info.next().ok_or("missing id")?.parse::<u32>()?;
            let unique_id = info.next().ok_or("missing unique id")?.to_string();

            result.push(Pop3MessageUidInfo { message_id: message_id, unique_id: unique_id });
        }

        Ok(result)
    }

    /// Returns the unique id of a given message.
    ///
    /// # Arguments
    ///
    /// * `message_id` - id of the message
    pub fn get_unique_id(&mut self, message_id :u32) -> Result<String, Box<dyn Error>> {
        let line = self.invoke_single_line(&format!("UIDL {}\r\n", message_id))?;
        let mut info = line.split(' ');
        let _ = info.next(); // skip "+OK"
        let _ = info.next(); // skip message id
        let unique_id = info.next().ok_or("missing unique id")?.to_string();

        Ok(unique_id)
    }
}

impl Drop for Pop3Connection {
    /// Closes POP3 connection on drop.
    fn drop(&mut self) {
        let _ = self.invoke_single_line("QUIT\r\n");
    }
}
