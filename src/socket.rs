use crate::url::Socket;
use native_tls::TlsConnector;
use std::io::{Read, Write};
use std::net::TcpStream;

#[derive(Debug)]
pub struct HttpSocket<S: Read + Write> {
    stream: S,
}

pub fn connect_http(host: &str, port: u16) -> Result<HttpSocket<TcpStream>, String> {
    let address = format!("{}:{}", host, port);
    let stream = TcpStream::connect(&address)
        .map_err(|e| format!("Failed to connect to {}: {}", address, e))?;
    Ok(HttpSocket { stream })
}

pub fn connect_https(
    host: &str,
    port: u16,
) -> Result<HttpSocket<native_tls::TlsStream<TcpStream>>, String> {
    let address = format!("{}:{}", host, port);
    let tcp_stream = TcpStream::connect(&address)
        .map_err(|e| format!("Failed to connect to {}: {}", address, e))?;

    let connector =
        TlsConnector::new().map_err(|e| format!("Failed to create TLS connector: {}", e))?;

    let tls_stream = connector
        .connect(host, tcp_stream)
        .map_err(|e| format!("Failed to establish TLS connection: {}", e))?;

    Ok(HttpSocket { stream: tls_stream })
}

impl<S: Read + Write> Socket for HttpSocket<S> {
    fn connect(&mut self, _host: &str, _port: u16) -> Result<(), String> {
        // Connection is handled by the connect_http/connect_https functions
        Ok(())
    }
    fn send(&mut self, data: &[u8]) -> Result<(), String> {
        self.stream
            .write_all(data)
            .map_err(|e| format!("Failed to send data: {}", e))
    }

    fn read_line(&mut self) -> Result<String, String> {
        let mut line = String::new();
        let mut buffer = [0; 1];
        
        loop {
            match self.stream.read(&mut buffer) {
                Ok(0) => {
                    if line.is_empty() {
                        return Err("End of file reached".to_string());
                    } else {
                        break;
                    }
                }
                Ok(_) => {
                    let ch = buffer[0] as char;
                    line.push(ch);
                    if ch == '\n' {
                        break;
                    }
                }
                Err(e) => return Err(format!("Failed to read line: {}", e)),
            }
        }
        
        Ok(line)
    }

    fn read_to_string(&mut self) -> Result<String, String> {
        let mut buffer = String::new();
        self.stream
            .read_to_string(&mut buffer)
            .map_err(|e| format!("Failed to read to string: {}", e))?;
        Ok(buffer)
    }
}
