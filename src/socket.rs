use crate::url::Socket;

#[derive(Debug)]
pub struct HttpSocket;

impl Socket for HttpSocket {
    fn connect(&self, host: &str, port: u16) -> Result<(), String> {
        println!("Connecting to {}:{}", host, port);
        Ok(())
    }
}