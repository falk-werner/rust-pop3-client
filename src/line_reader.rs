use std::error::Error;
use std::str::from_utf8;
use std::io::Read;

const BUFFER_SIZE : usize = 512;
const EOL : u8 = 0x0a;

pub struct LineReader {
    buffer: [u8; BUFFER_SIZE],
    pos: usize
}

impl LineReader {

    pub fn new() -> Self {
        LineReader { buffer: [0; BUFFER_SIZE], pos: 0 }
    }

    fn get_eol(&self) -> Option<usize> {
        for (pos, &item) in self.buffer[0..self.pos].iter().enumerate() {
            if item == EOL {
                return Some(pos);
            }
        }

        None
    }

    pub fn read_line(&mut self, reader: &mut impl Read) -> Result<String, Box<dyn Error>> {
        loop  {
            if self.get_eol().is_some() {
                break;
            }

            if self.pos >= BUFFER_SIZE {
                return Err("buffer exceeded".into());
            }

            let len = reader.read(&mut self.buffer[self.pos..])?;
            self.pos += len;
        }

        if let Some(eol) = self.get_eol() {
            let line = from_utf8(& self.buffer[0..eol])?.trim().to_string();
            let pos = eol + 1;
            self.buffer.copy_within(pos.., 0);
            self.pos -= pos;
            return Ok(line.to_string());
        }

        Err("no line available".into())
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_single_line() {
        let mut reader = LineReader::new();
        let data = b"Hello\n";
        let mut slice: &[u8] = data.as_ref();
        let line = reader.read_line(&mut slice).unwrap();
        assert_eq!("Hello".to_string(), line);
    }

    #[test]
    fn test_read_multiple_lines() {
        let mut reader = LineReader::new();
        let data = b"Hello\nWorld\n";
        let mut slice: &[u8] = data.as_ref();
        let line = reader.read_line(&mut slice).unwrap();
        assert_eq!("Hello".to_string(), line);

        let data = [];
        let mut slice: &[u8] = data.as_ref();
        let line = reader.read_line(&mut slice).unwrap();
        assert_eq!("World".to_string(), line);
    }

    #[test]
    fn test_read_split_lines() {
        let mut reader = LineReader::new();
        let data = b"Hello\nWor";
        let mut slice: &[u8] = data.as_ref();
        let line = reader.read_line(&mut slice).unwrap();
        assert_eq!("Hello".to_string(), line);

        let data = b"ld\n";
        let mut slice: &[u8] = data.as_ref();
        let line = reader.read_line(&mut slice).unwrap();
        assert_eq!("World".to_string(), line);
    }

    #[test]
    fn test_read_buffer_exceeded() {
        let mut reader = LineReader::new();
        let data = [0; 512];
        let mut slice: &[u8] = data.as_ref();
        let line = reader.read_line(&mut slice);
        assert!(line.is_err());
    }

}
