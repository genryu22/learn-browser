use crate::socket::{connect_http, connect_https};
use std::collections::HashMap;

pub trait Socket {
    fn connect(&mut self, host: &str, port: u16) -> Result<(), String>;
    fn send(&mut self, data: &[u8]) -> Result<(), String>;
    fn read_line(&mut self) -> Result<String, String>;
    fn read_to_string(&mut self) -> Result<String, String>;
}

#[derive(Debug, PartialEq)]
pub enum Scheme {
    Http,
    Https,
}

#[derive(Debug)]
pub struct HttpResponse {
    pub version: String,
    pub status: u16,
    pub explanation: String,
    pub headers: HashMap<String, String>,
    pub body: String,
}

#[derive(Debug)]
pub struct Url {
    pub scheme: Scheme,
    pub host: String,
    pub path: String,
}

fn make_request_with_socket<S: Socket>(socket: &mut S, url: &Url) -> Result<HttpResponse, String> {
    socket.connect(&url.host, 80)?;

    let http_request = format!("GET {} HTTP/1.0\r\nHost: {}\r\n\r\n", url.path, url.host);

    socket.send(http_request.as_bytes())?;

    // Read status line
    let status_line = socket.read_line()?;
    let status_line = status_line.trim_end_matches("\r\n");
    let status_parts: Vec<&str> = status_line.split(' ').collect();

    if status_parts.len() < 3 {
        return Err("Invalid HTTP status line".to_string());
    }

    let version = status_parts[0].to_string();
    let status = status_parts[1]
        .parse::<u16>()
        .map_err(|_| "Invalid HTTP status code".to_string())?;
    let explanation = status_parts[2..].join(" ");

    // Read headers
    let mut headers = HashMap::new();
    loop {
        let line = socket.read_line()?;
        if line == "\r\n" {
            break;
        }

        let line = line.trim_end_matches("\r\n");
        if let Some(colon_pos) = line.find(':') {
            let header = line[..colon_pos].trim().to_lowercase();
            let value = line[colon_pos + 1..].trim().to_string();
            headers.insert(header, value);
        }
    }

    // Read body
    let body = socket.read_to_string()?;

    Ok(HttpResponse {
        version,
        status,
        explanation,
        headers,
        body,
    })
}

pub fn request(url: &Url) -> Result<HttpResponse, String> {
    match url.scheme {
        Scheme::Http => {
            let mut socket = connect_http(&url.host, 80)?;
            make_request_with_socket(&mut socket, url)
        }
        Scheme::Https => {
            let mut socket = connect_https(&url.host, 443)?;
            make_request_with_socket(&mut socket, url)
        }
    }
}

pub fn strip_html_tags(text: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;

    for char in text.chars() {
        match char {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => result.push(char),
            _ => {}
        }
    }

    result
}

impl Url {
    pub fn new(raw_url: &str) -> Result<Url, String> {
        let parts: Vec<&str> = raw_url.splitn(2, "://").collect();
        if parts.len() != 2 {
            return Err("Invalid URL: missing scheme".to_string());
        }

        let scheme = match parts[0] {
            "http" => Scheme::Http,
            "https" => Scheme::Https,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct MockSocket;

    impl Socket for MockSocket {
        fn connect(&mut self, _host: &str, _port: u16) -> Result<(), String> {
            Ok(())
        }

        fn send(&mut self, _data: &[u8]) -> Result<(), String> {
            Ok(())
        }

        fn read_line(&mut self) -> Result<String, String> {
            Ok("HTTP/1.0 200 OK\r\n".to_string())
        }

        fn read_to_string(&mut self) -> Result<String, String> {
            Ok("Mock body content".to_string())
        }
    }

    #[derive(Debug)]
    struct TestSocket {
        connect_calls: Vec<(String, u16)>,
        send_calls: Vec<String>,
        connect_should_fail: bool,
        send_should_fail: bool,
        response_lines: Vec<String>,
        current_line_index: usize,
    }

    impl TestSocket {
        fn new() -> Self {
            TestSocket::with_response_lines(vec!["HTTP/1.0 200 OK\r\n".to_string()])
        }

        fn with_response_lines(lines: Vec<String>) -> Self {
            TestSocket {
                connect_calls: Vec::new(),
                send_calls: Vec::new(),
                connect_should_fail: false,
                send_should_fail: false,
                response_lines: lines,
                current_line_index: 0,
            }
        }

        fn with_full_response() -> Self {
            TestSocket::with_response_lines(vec![
                "HTTP/1.1 200 OK\r\n".to_string(),
                "Content-Type: text/html\r\n".to_string(),
                "Content-Length: 13\r\n".to_string(),
                "\r\n".to_string(),
                "Hello, World!".to_string(),
            ])
        }

        fn with_connect_failure() -> Self {
            TestSocket {
                connect_calls: Vec::new(),
                send_calls: Vec::new(),
                connect_should_fail: true,
                send_should_fail: false,
                response_lines: Vec::new(),
                current_line_index: 0,
            }
        }

        fn with_send_failure() -> Self {
            TestSocket {
                connect_calls: Vec::new(),
                send_calls: Vec::new(),
                connect_should_fail: false,
                send_should_fail: true,
                response_lines: Vec::new(),
                current_line_index: 0,
            }
        }

        fn with_eof_after_status() -> Self {
            TestSocket::with_response_lines(vec![
                "HTTP/1.1 200 OK\r\n".to_string(),
                // EOF happens here, no headers or body
            ])
        }

        fn with_eof_during_headers() -> Self {
            TestSocket::with_response_lines(vec![
                "HTTP/1.1 200 OK\r\n".to_string(),
                "Content-Type: text/html\r\n".to_string(),
                // EOF happens here, missing \r\n separator and body
            ])
        }

        fn with_eof_before_status() -> Self {
            TestSocket::with_response_lines(vec![
                // EOF happens immediately, no status line
            ])
        }
    }

    impl Socket for TestSocket {
        fn connect(&mut self, host: &str, port: u16) -> Result<(), String> {
            self.connect_calls.push((host.to_string(), port));
            if self.connect_should_fail {
                Err("Connection failed".to_string())
            } else {
                Ok(())
            }
        }

        fn send(&mut self, data: &[u8]) -> Result<(), String> {
            self.send_calls
                .push(String::from_utf8_lossy(data).to_string());
            if self.send_should_fail {
                Err("Send failed".to_string())
            } else {
                Ok(())
            }
        }

        fn read_line(&mut self) -> Result<String, String> {
            if self.current_line_index < self.response_lines.len() {
                let line = self.response_lines[self.current_line_index].clone();
                self.current_line_index += 1;
                Ok(line)
            } else {
                Err("No more lines to read".to_string())
            }
        }

        fn read_to_string(&mut self) -> Result<String, String> {
            // Read all remaining lines and concatenate them
            let mut result = String::new();
            while self.current_line_index < self.response_lines.len() {
                result.push_str(&self.response_lines[self.current_line_index]);
                self.current_line_index += 1;
            }
            Ok(result)
        }
    }

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
    fn test_url_new_https_scheme() {
        let url = Url::new("https://example.com").unwrap();
        assert_eq!(url.scheme, Scheme::Https);
        assert_eq!(url.host, "example.com");
        assert_eq!(url.path, "/");
    }

    #[test]
    fn test_socket_connect() {
        let _url = Url::new("http://example.com").unwrap();
        let mut socket = MockSocket;
        let result = socket.connect("example.com", 80);
        assert!(result.is_ok());
    }

    #[test]
    fn test_url_request() {
        let mut socket = TestSocket::with_full_response();
        let url = Url::new("http://example.com/path").unwrap();
        let result = make_request_with_socket(&mut socket, &url);
        assert!(result.is_ok());
    }

    #[test]
    fn test_url_request_connect_failed() {
        let mut socket = TestSocket::with_connect_failure();
        let url = Url::new("http://example.com/path").unwrap();

        let result = make_request_with_socket(&mut socket, &url);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Connection failed");

        assert_eq!(socket.connect_calls.len(), 1);
        assert_eq!(socket.connect_calls[0], ("example.com".to_string(), 80));

        assert_eq!(socket.send_calls.len(), 0);
    }

    #[test]
    fn test_url_request_send_failed() {
        let mut socket = TestSocket::with_send_failure();
        let url = Url::new("http://example.com/path").unwrap();

        let result = make_request_with_socket(&mut socket, &url);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Send failed");

        assert_eq!(socket.connect_calls.len(), 1);
        assert_eq!(socket.connect_calls[0], ("example.com".to_string(), 80));

        assert_eq!(socket.send_calls.len(), 1);
        assert_eq!(
            socket.send_calls[0],
            "GET /path HTTP/1.0\r\nHost: example.com\r\n\r\n"
        );
    }

    #[test]
    fn test_url_request_success() {
        let mut socket = TestSocket::with_full_response();
        let url = Url::new("http://example.com/path/to/resource").unwrap();

        let result = make_request_with_socket(&mut socket, &url);

        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.version, "HTTP/1.1");
        assert_eq!(response.status, 200);
        assert_eq!(response.explanation, "OK");
        assert_eq!(
            response.headers.get("content-type"),
            Some(&"text/html".to_string())
        );
        assert_eq!(
            response.headers.get("content-length"),
            Some(&"13".to_string())
        );
        assert_eq!(response.body, "Hello, World!");

        assert_eq!(socket.connect_calls.len(), 1);
        assert_eq!(socket.connect_calls[0], ("example.com".to_string(), 80));

        assert_eq!(socket.send_calls.len(), 1);
        assert_eq!(
            socket.send_calls[0],
            "GET /path/to/resource HTTP/1.0\r\nHost: example.com\r\n\r\n"
        );
    }

    #[test]
    fn test_http_response_parsing_status_line() {
        let mut socket = TestSocket::with_response_lines(vec![
            "HTTP/1.0 404 Not Found\r\n".to_string(),
            "\r\n".to_string(),
        ]);
        let url = Url::new("http://example.com/notfound").unwrap();

        let result = make_request_with_socket(&mut socket, &url);
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.version, "HTTP/1.0");
        assert_eq!(response.status, 404);
        assert_eq!(response.explanation, "Not Found");
        assert!(response.headers.is_empty());
        assert_eq!(response.body, "");
    }

    #[test]
    fn test_http_response_parsing_with_headers() {
        let mut socket = TestSocket::with_response_lines(vec![
            "HTTP/1.1 200 OK\r\n".to_string(),
            "Server: Apache/2.4.41\r\n".to_string(),
            "Content-Type: application/json\r\n".to_string(),
            "Cache-Control: no-cache\r\n".to_string(),
            "\r\n".to_string(),
            "{\"message\": \"success\"}".to_string(),
        ]);
        let url = Url::new("http://api.example.com/data").unwrap();

        let result = make_request_with_socket(&mut socket, &url);
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.version, "HTTP/1.1");
        assert_eq!(response.status, 200);
        assert_eq!(response.explanation, "OK");
        assert_eq!(
            response.headers.get("server"),
            Some(&"Apache/2.4.41".to_string())
        );
        assert_eq!(
            response.headers.get("content-type"),
            Some(&"application/json".to_string())
        );
        assert_eq!(
            response.headers.get("cache-control"),
            Some(&"no-cache".to_string())
        );
        assert_eq!(response.body, "{\"message\": \"success\"}");
    }

    #[test]
    fn test_http_response_parsing_multiline_body() {
        let mut socket = TestSocket::with_response_lines(vec![
            "HTTP/1.1 200 OK\r\n".to_string(),
            "Content-Type: text/plain\r\n".to_string(),
            "\r\n".to_string(),
            "Line 1\n".to_string(),
            "Line 2\n".to_string(),
            "Line 3".to_string(),
        ]);
        let url = Url::new("http://example.com/text").unwrap();

        let result = make_request_with_socket(&mut socket, &url);
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.body, "Line 1\nLine 2\nLine 3");
    }

    #[test]
    fn test_http_response_parsing_invalid_status_line() {
        let mut socket = TestSocket::with_response_lines(vec![
            "HTTP/1.1 ABC Not Found\r\n".to_string(),
            "\r\n".to_string(),
        ]);
        let url = Url::new("http://example.com/invalid").unwrap();

        let result = make_request_with_socket(&mut socket, &url);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid HTTP status code");
    }

    #[test]
    fn test_http_response_parsing_headers_case_insensitive() {
        let mut socket = TestSocket::with_response_lines(vec![
            "HTTP/1.1 200 OK\r\n".to_string(),
            "Content-Type: text/html\r\n".to_string(),
            "CONTENT-LENGTH: 5\r\n".to_string(),
            "Cache-CONTROL: max-age=3600\r\n".to_string(),
            "\r\n".to_string(),
            "Hello".to_string(),
        ]);
        let url = Url::new("http://example.com/case").unwrap();

        let result = make_request_with_socket(&mut socket, &url);
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(
            response.headers.get("content-type"),
            Some(&"text/html".to_string())
        );
        assert_eq!(
            response.headers.get("content-length"),
            Some(&"5".to_string())
        );
        assert_eq!(
            response.headers.get("cache-control"),
            Some(&"max-age=3600".to_string())
        );
    }

    #[test]
    fn test_html_tag_stripping() {
        assert_eq!(
            strip_html_tags("<html><body>Hello World</body></html>"),
            "Hello World"
        );
        assert_eq!(
            strip_html_tags("<p>Text with <strong>bold</strong> content</p>"),
            "Text with bold content"
        );
        assert_eq!(strip_html_tags("No tags here"), "No tags here");
        assert_eq!(strip_html_tags(""), "");
    }

    #[test]
    fn example_request_with_html_stripping() {
        let mut socket = TestSocket::with_response_lines(vec![
            "HTTP/1.1 200 OK\r\n".to_string(),
            "Content-Type: text/html\r\n".to_string(),
            "Content-Length: 50\r\n".to_string(),
            "\r\n".to_string(),
            "<html><head><title>Test</title></head>".to_string(),
            "<body><h1>Welcome!</h1><p>This is a test page.</p></body></html>".to_string(),
        ]);

        let url = Url::new("http://example.com").unwrap();
        let response = make_request_with_socket(&mut socket, &url).unwrap();

        println!("Status: {}", response.status);
        println!("Headers: {:?}", response.headers);
        println!("Raw body: {}", response.body);

        let clean_body = strip_html_tags(&response.body);
        println!("Body without HTML tags: {}", clean_body);

        assert_eq!(response.status, 200);
        assert_eq!(clean_body, "TestWelcome!This is a test page.");
    }

    #[test]
    fn test_eof_before_status_line() {
        let mut socket = TestSocket::with_eof_before_status();
        let url = Url::new("http://example.com").unwrap();

        let result = make_request_with_socket(&mut socket, &url);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "No more lines to read");
    }

    #[test]
    fn test_eof_after_status_line() {
        let mut socket = TestSocket::with_eof_after_status();
        let url = Url::new("http://example.com").unwrap();

        let result = make_request_with_socket(&mut socket, &url);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "No more lines to read");
    }

    #[test]
    fn test_eof_during_headers() {
        let mut socket = TestSocket::with_eof_during_headers();
        let url = Url::new("http://example.com").unwrap();

        let result = make_request_with_socket(&mut socket, &url);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "No more lines to read");
    }

    #[test]
    fn test_eof_in_body_is_ok() {
        // EOF during body reading should be OK - that's how we know the body is complete
        let mut socket = TestSocket::with_response_lines(vec![
            "HTTP/1.1 200 OK\r\n".to_string(),
            "Content-Type: text/plain\r\n".to_string(),
            "\r\n".to_string(),
            "Partial body".to_string(),
            // EOF happens here during body reading - this should be OK
        ]);
        let url = Url::new("http://example.com").unwrap();

        let result = make_request_with_socket(&mut socket, &url);
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.status, 200);
        assert_eq!(response.body, "Partial body");
    }
}
