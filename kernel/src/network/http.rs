// HTTP Client for ASTRA.OS Browser
// Implements basic HTTP/1.1 GET requests

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::format;
use super::url::Url;

/// HTTP request method
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HttpMethod {
    GET,
    POST,
    HEAD,
}

impl HttpMethod {
    pub fn as_str(&self) -> &str {
        match self {
            HttpMethod::GET => "GET",
            HttpMethod::POST => "POST",
            HttpMethod::HEAD => "HEAD",
        }
    }
}

/// HTTP header
#[derive(Debug, Clone)]
pub struct HttpHeader {
    pub name: String,
    pub value: String,
}

impl HttpHeader {
    pub fn new(name: String, value: String) -> Self {
        HttpHeader { name, value }
    }
}

/// HTTP request
#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub method: HttpMethod,
    pub url: Url,
    pub headers: Vec<HttpHeader>,
    pub body: Option<Vec<u8>>,
}

impl HttpRequest {
    /// Create a new GET request
    pub fn get(url: Url) -> Self {
        let mut headers = Vec::new();

        // Add default headers
        if let Some(ref host) = url.host {
            headers.push(HttpHeader::new(
                String::from("Host"),
                host.clone()
            ));
        }

        headers.push(HttpHeader::new(
            String::from("User-Agent"),
            String::from("ASTRA.OS-Browser/0.1")
        ));

        headers.push(HttpHeader::new(
            String::from("Accept"),
            String::from("text/html,text/css,*/*")
        ));

        headers.push(HttpHeader::new(
            String::from("Connection"),
            String::from("close")
        ));

        HttpRequest {
            method: HttpMethod::GET,
            url,
            headers,
            body: None,
        }
    }

    /// Add a header
    pub fn add_header(&mut self, name: String, value: String) {
        self.headers.push(HttpHeader::new(name, value));
    }

    /// Build HTTP request string
    pub fn to_request_string(&self) -> String {
        let mut request = String::new();

        // Request line
        request.push_str(self.method.as_str());
        request.push(' ');

        // Path and query
        request.push_str(&self.url.path);
        if let Some(ref query) = self.url.query {
            request.push('?');
            request.push_str(query);
        }

        request.push_str(" HTTP/1.1\r\n");

        // Headers
        for header in &self.headers {
            request.push_str(&header.name);
            request.push_str(": ");
            request.push_str(&header.value);
            request.push_str("\r\n");
        }

        // Empty line to end headers
        request.push_str("\r\n");

        request
    }
}

/// HTTP response status
#[derive(Debug, Clone, PartialEq)]
pub struct HttpStatus {
    pub code: u16,
    pub reason: String,
}

impl HttpStatus {
    pub fn new(code: u16, reason: String) -> Self {
        HttpStatus { code, reason }
    }

    pub fn is_success(&self) -> bool {
        self.code >= 200 && self.code < 300
    }

    pub fn is_redirect(&self) -> bool {
        self.code >= 300 && self.code < 400
    }

    pub fn is_client_error(&self) -> bool {
        self.code >= 400 && self.code < 500
    }

    pub fn is_server_error(&self) -> bool {
        self.code >= 500 && self.code < 600
    }
}

/// HTTP response
#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub status: HttpStatus,
    pub headers: Vec<HttpHeader>,
    pub body: Vec<u8>,
}

impl HttpResponse {
    pub fn new() -> Self {
        HttpResponse {
            status: HttpStatus::new(0, String::new()),
            headers: Vec::new(),
            body: Vec::new(),
        }
    }

    /// Get header value by name (case-insensitive)
    pub fn get_header(&self, name: &str) -> Option<&String> {
        for header in &self.headers {
            if header.name.eq_ignore_ascii_case(name) {
                return Some(&header.value);
            }
        }
        None
    }

    /// Get content type
    pub fn content_type(&self) -> Option<&String> {
        self.get_header("Content-Type")
    }

    /// Get content length
    pub fn content_length(&self) -> Option<usize> {
        self.get_header("Content-Length")
            .and_then(|v| v.parse().ok())
    }

    /// Get body as string (assuming UTF-8)
    pub fn body_as_string(&self) -> Option<String> {
        core::str::from_utf8(&self.body)
            .ok()
            .map(String::from)
    }
}

/// Parse HTTP response from bytes
pub fn parse_response(data: &[u8]) -> Result<HttpResponse, &'static str> {
    // Find end of headers (\r\n\r\n)
    let header_end = find_header_end(data)
        .ok_or("Invalid HTTP response: no header end")?;

    let header_data = &data[..header_end];
    let body_data = &data[header_end + 4..]; // Skip \r\n\r\n

    // Parse headers
    let header_str = core::str::from_utf8(header_data)
        .map_err(|_| "Invalid UTF-8 in headers")?;

    let mut lines = header_str.lines();

    // Parse status line
    let status_line = lines.next()
        .ok_or("Missing status line")?;

    let status = parse_status_line(status_line)?;

    // Parse headers
    let mut headers = Vec::new();
    for line in lines {
        if line.is_empty() {
            continue;
        }

        if let Some(pos) = line.find(':') {
            let name = String::from(line[..pos].trim());
            let value = String::from(line[pos + 1..].trim());
            headers.push(HttpHeader::new(name, value));
        }
    }

    Ok(HttpResponse {
        status,
        headers,
        body: Vec::from(body_data),
    })
}

/// Find end of HTTP headers
fn find_header_end(data: &[u8]) -> Option<usize> {
    for i in 0..data.len().saturating_sub(3) {
        if data[i] == b'\r' && data[i+1] == b'\n' &&
           data[i+2] == b'\r' && data[i+3] == b'\n' {
            return Some(i);
        }
    }
    None
}

/// Parse HTTP status line
fn parse_status_line(line: &str) -> Result<HttpStatus, &'static str> {
    let parts: Vec<&str> = line.splitn(3, ' ').collect();

    if parts.len() < 2 {
        return Err("Invalid status line");
    }

    let code = parts[1].parse::<u16>()
        .map_err(|_| "Invalid status code")?;

    let reason = if parts.len() > 2 {
        String::from(parts[2])
    } else {
        String::new()
    };

    Ok(HttpStatus::new(code, reason))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_request_get() {
        let url = Url::parse("http://example.com/path").unwrap();
        let request = HttpRequest::get(url);

        assert_eq!(request.method, HttpMethod::GET);
        let req_str = request.to_request_string();
        assert!(req_str.starts_with("GET /path HTTP/1.1\r\n"));
        assert!(req_str.contains("Host: example.com\r\n"));
    }

    #[test]
    fn test_parse_response() {
        let response_data = b"HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: 5\r\n\r\nHello";
        let response = parse_response(response_data).unwrap();

        assert_eq!(response.status.code, 200);
        assert_eq!(response.status.reason, "OK");
        assert_eq!(response.get_header("Content-Type"), Some(&String::from("text/html")));
        assert_eq!(response.body, b"Hello");
    }
}
