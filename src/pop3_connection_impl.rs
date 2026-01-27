use std::io::{Read, Write};
use std::error::Error;

use crate::LineReader;
use crate::Pop3Connection;
use crate::Pop3Stat;
use crate::Pop3MessageInfo;
use crate::Pop3MessageUidInfo;

/// POP3 connection implementation
pub struct Pop3ConnectionImpl<T> where T: Read + Write {
    pub stream: T,
    pub reader: LineReader,
}

impl<T> Pop3ConnectionImpl<T> where T: Read + Write {


    pub fn read_status_line(&mut self) -> Result<String, Box<dyn Error>> {
        let line = self.reader.read_line(&mut self.stream)?;

        match line.starts_with("+OK") {
            true => Ok(line),
            _ => Err(line.into())
        }
    }

    fn invoke_single_line(&mut self, command: &str) -> Result<String, Box<dyn Error>> {
        self.stream.write_all(command.as_bytes())?;
        self.read_status_line()
    }

    fn invoke_multi_line(&mut self, command: &str) -> Result<Vec<String>, Box<dyn Error>> {
        self.stream.write_all(command.as_bytes())?;
        self.read_status_line()?;

        let mut response : Vec<String> = vec!();
        loop {
            let line = self.reader.read_line(&mut self.stream)?;
            match line {
                _ if line == "." => { break },
                _ if line.starts_with(".") => { response.push(line[1..].to_string()); },
                _ => { response.push(line); }
            };
        }

        Ok(response)
    }
}

impl<T> Pop3Connection for Pop3ConnectionImpl<T> where T: Read + Write {

    fn login(&mut self, user: &str, password: &str) -> Result<(), Box<dyn Error>> {
        self.invoke_single_line(&format!("USER {}\r\n", user))?;
        self.invoke_single_line(&format!("PASS {}\r\n", password))?;
        Ok(())
    }

    fn stat(&mut self) -> Result<Pop3Stat, Box<dyn Error>> {
        let stat = self.invoke_single_line("STAT\r\n")?;
        let mut stat = stat.split(' ');
        let _ = stat.next();
        let message_count = stat.next().ok_or("missing message count")?;
        let message_count = message_count.parse::<u32>()?;
        let maildrop_size = stat.next().ok_or("missing maildrop size")?;
        let maildrop_size = maildrop_size.parse::<u32>()?;

        Ok(Pop3Stat { message_count, maildrop_size })
    }

    fn list(&mut self) -> Result<Vec<Pop3MessageInfo>, Box<dyn Error>> {
        let lines = self.invoke_multi_line("LIST\r\n")?;
        let mut result = vec!();
        for line in lines {
            let mut info = line.split(' ');
            let message_id = info.next().ok_or("missing id")?.parse::<u32>()?;
            let message_size = info.next().ok_or("missing size")?.parse::<u32>()?;

            result.push(Pop3MessageInfo { message_id, message_size });
        }

        Ok(result)
    }

    fn get_message_size(&mut self, message_id: u32) -> Result<u32, Box<dyn Error>> {
        let line = self.invoke_single_line(&format!("LIST {}\r\n", message_id))?;
        let mut info = line.split(' ');
        let _ = info.next();    // skip "+OK"
        let _ = info.next();    // skip message id
        let message_size = info.next().ok_or("missing size")?.parse::<u32>()?;
     
        Ok(message_size)
    }

    fn retrieve(&mut self, message_id: u32, writer: &mut dyn Write) -> Result<(), Box<dyn Error>> {
        let lines = self.invoke_multi_line(&format!("RETR {}\r\n", message_id))?;
        for line in lines {
            writer.write_all(line.as_bytes())?;
            writer.write_all(b"\n")?;
        }

        Ok(())
    }

    fn delete(&mut self, message_id: u32) -> Result<(), Box<dyn Error>> {
        self.invoke_single_line(&format!("DELE {}\r\n", message_id))?;
        Ok(())
    }

    fn reset(&mut self) -> Result<(), Box<dyn Error>> {
        self.invoke_single_line("RSET\r\n")?;
        Ok(())
    }

    fn top(&mut self, message_id: u32, line_count: u32) -> Result<String, Box<dyn Error>> {
        let lines = self.invoke_multi_line(&format!("TOP {} {}\r\n", message_id, line_count))?;
        let mut message = String::new();
        for line in lines {
            message.push_str(&line);
            message.push('\n');
        }

        Ok(message)
    }

    fn list_unique_ids(&mut self) -> Result<Vec<Pop3MessageUidInfo>, Box<dyn Error>> {
        let lines = self.invoke_multi_line("UIDL\r\n")?;
        let mut result = vec!();

        for line in lines {
            let mut info = line.split(' ');
            let message_id = info.next().ok_or("missing id")?.parse::<u32>()?;
            let unique_id = info.next().ok_or("missing unique id")?.to_string();

            result.push(Pop3MessageUidInfo { message_id, unique_id });
        }

        Ok(result)
    }

    fn get_unique_id(&mut self, message_id :u32) -> Result<String, Box<dyn Error>> {
        let line = self.invoke_single_line(&format!("UIDL {}\r\n", message_id))?;
        let mut info = line.split(' ');
        let _ = info.next(); // skip "+OK"
        let _ = info.next(); // skip message id
        let unique_id = info.next().ok_or("missing unique id")?.to_string();

        Ok(unique_id)
    }

}

impl<T> Drop for Pop3ConnectionImpl<T> where T: Read + Write {
    /// Closes POP3 connection on drop.
    fn drop(&mut self) {
        let _ = self.invoke_single_line("QUIT\r\n");
    }
}