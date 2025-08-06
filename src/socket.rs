use crate::url::Socket;
use std::net::TcpStream;
use std::io::{Write, BufRead, BufReader, Read};

#[derive(Debug)]
pub struct HttpSocket {
    stream: Option<TcpStream>,
    reader: Option<BufReader<TcpStream>>,
}

impl HttpSocket {
    pub fn new() -> Self {
        HttpSocket {
            stream: None,
            reader: None,
        }
    }
}

impl Socket for HttpSocket {
    fn connect(&mut self, host: &str, port: u16) -> Result<(), String> {
        let address = format!("{}:{}", host, port);
        match TcpStream::connect(&address) {
            Ok(stream) => {
                let cloned_stream = stream.try_clone()
                    .map_err(|e| format!("Failed to clone stream: {}", e))?;
                self.reader = Some(BufReader::new(cloned_stream));
                self.stream = Some(stream);
                Ok(())
            }
            Err(e) => Err(format!("Failed to connect to {}: {}", address, e)),
        }
    }

    fn send(&mut self, data: &[u8]) -> Result<(), String> {
        match self.stream.as_mut() {
            Some(stream) => {
                match stream.write_all(data) {
                    Ok(()) => Ok(()),
                    Err(e) => Err(format!("Failed to send data: {}", e)),
                }
            }
            None => Err("Not connected to any server".to_string()),
        }
    }

    fn read_line(&mut self) -> Result<String, String> {
        match self.reader.as_mut() {
            Some(reader) => {
                let mut line = String::new();
                match reader.read_line(&mut line) {
                    Ok(0) => Err("End of file reached".to_string()),
                    Ok(_) => Ok(line),
                    Err(e) => Err(format!("Failed to read line: {}", e)),
                }
            }
            None => Err("Not connected to any server".to_string()),
        }
    }

    fn read_to_string(&mut self) -> Result<String, String> {
        match self.reader.as_mut() {
            Some(reader) => {
                let mut buffer = String::new();
                match reader.read_to_string(&mut buffer) {
                    Ok(_) => Ok(buffer),
                    Err(e) => Err(format!("Failed to read to string: {}", e)),
                }
            }
            None => Err("Not connected to any server".to_string()),
        }
    }
}