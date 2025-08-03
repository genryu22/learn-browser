#[derive(Debug, PartialEq)]
enum Scheme {
    Http,
}

#[derive(Debug)]
struct Url {
    scheme: Scheme,
    host: String,
    path: String,
}

impl Url {
    fn new(raw_url: &str) -> Result<Url, String> {
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
        
        Ok(Url { scheme, host, path })
    }
}

fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_new_with_path() {
        let url = Url::new("http://example.com/path/to/resource").unwrap();
        assert_eq!(url.scheme, Scheme::Http);
        assert_eq!(url.host, "example.com");
        assert_eq!(url.path, "/path/to/resource");
    }

    #[test]
    fn test_url_new_without_path() {
        let url = Url::new("http://google.com").unwrap();
        assert_eq!(url.scheme, Scheme::Http);
        assert_eq!(url.host, "google.com");
        assert_eq!(url.path, "/");
    }

    #[test]
    fn test_url_new_invalid() {
        let result = Url::new("invalid-url");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid URL: missing scheme");
    }

    #[test]
    fn test_url_new_unsupported_scheme() {
        let result = Url::new("https://example.com");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Unsupported scheme: https");
    }
}
