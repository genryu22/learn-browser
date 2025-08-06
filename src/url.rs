pub trait Socket {
    fn connect(&self, host: &str, port: u16) -> Result<(), String>;
}

#[derive(Debug, PartialEq)]
pub enum Scheme {
    Http,
}

#[derive(Debug)]
pub struct Url<T: Socket + std::fmt::Debug> {
    pub scheme: Scheme,
    pub host: String,
    pub path: String,
    pub socket: T,
}

impl<T: Socket + std::fmt::Debug> Url<T> {
    pub fn new(raw_url: &str, socket: T) -> Result<Url<T>, String> {
        let parts: Vec<&str> = raw_url.splitn(2, "://").collect();
        if parts.len() != 2 {
            return Err("Invalid URL: missing scheme".to_string());
        }
        
        let scheme = match parts[0] {
            "http" => Scheme::Http,
            _ => return Err(format!("Unsupported scheme: {}", parts[0])),
        };
        let remaining = parts[1];
        
        let parts: Vec<&str> = remaining.splitn(2, '/').collect();
        let host = parts[0].to_string();
        let path = if parts.len() > 1 {
            format!("/{}", parts[1])
        } else {
            "/".to_string()
        };
        
        Ok(Url { 
            scheme, 
            host, 
            path,
            socket,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct MockSocket;

    impl Socket for MockSocket {
        fn connect(&self, _host: &str, _port: u16) -> Result<(), String> {
            Ok(())
        }
    }

    #[test]
    fn test_url_new_with_path() {
        let url = Url::new("http://example.com/path/to/resource", MockSocket).unwrap();
        assert_eq!(url.scheme, Scheme::Http);
        assert_eq!(url.host, "example.com");
        assert_eq!(url.path, "/path/to/resource");
    }

    #[test]
    fn test_url_new_without_path() {
        let url = Url::new("http://google.com", MockSocket).unwrap();
        assert_eq!(url.scheme, Scheme::Http);
        assert_eq!(url.host, "google.com");
        assert_eq!(url.path, "/");
    }

    #[test]
    fn test_url_new_invalid() {
        let result = Url::new("invalid-url", MockSocket);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid URL: missing scheme");
    }

    #[test]
    fn test_url_new_unsupported_scheme() {
        let result = Url::new("https://example.com", MockSocket);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Unsupported scheme: https");
    }

    #[test]
    fn test_socket_connect() {
        let url = Url::new("http://example.com", MockSocket).unwrap();
        let result = url.socket.connect("example.com", 80);
        assert!(result.is_ok());
    }
}