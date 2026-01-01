// URL Parser for ASTRA.OS Browser
// Parses and validates URLs for resource loading

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::format;

/// URL structure
#[derive(Debug, Clone, PartialEq)]
pub struct Url {
    pub scheme: String,      // http, https, file
    pub host: Option<String>,
    pub port: Option<u16>,
    pub path: String,
    pub query: Option<String>,
    pub fragment: Option<String>,
}

impl Url {
    /// Parse a URL string
    pub fn parse(url_str: &str) -> Result<Self, &'static str> {
        // Remove leading/trailing whitespace
        let url_str = url_str.trim();

        if url_str.is_empty() {
            return Err("Empty URL");
        }

        // Parse scheme
        let (scheme, rest) = parse_scheme(url_str)?;

        // For file:// URLs
        if scheme == "file" {
            let path = if rest.starts_with("//") {
                String::from(&rest[2..])
            } else {
                String::from(rest)
            };

            return Ok(Url {
                scheme,
                host: None,
                port: None,
                path,
                query: None,
                fragment: None,
            });
        }

        // Parse authority (host:port)
        let rest = if rest.starts_with("//") {
            &rest[2..]
        } else {
            rest
        };

        let (authority, path_and_rest) = split_at_char(rest, '/');
        let (host, port) = parse_authority(authority)?;

        // Parse path, query, fragment
        let path_and_rest = if path_and_rest.is_empty() {
            String::from("/")
        } else {
            format!("/{}", path_and_rest)
        };

        let (path_and_query, fragment) = split_at_char_opt(&path_and_rest, '#');
        let (path, query) = split_at_char_opt(path_and_query, '?');

        Ok(Url {
            scheme,
            host: Some(host),
            port,
            path: String::from(path),
            query: query.map(String::from),
            fragment: fragment.map(String::from),
        })
    }

    /// Get the default port for a scheme
    pub fn default_port(&self) -> Option<u16> {
        match self.scheme.as_str() {
            "http" => Some(80),
            "https" => Some(443),
            _ => None,
        }
    }

    /// Get the port (or default port)
    pub fn port_or_default(&self) -> Option<u16> {
        self.port.or_else(|| self.default_port())
    }

    /// Convert URL back to string
    pub fn to_string(&self) -> String {
        let mut result = String::new();

        result.push_str(&self.scheme);
        result.push_str("://");

        if let Some(ref host) = self.host {
            result.push_str(host);

            if let Some(port) = self.port {
                if Some(port) != self.default_port() {
                    result.push(':');
                    result.push_str(&format!("{}", port));
                }
            }
        }

        result.push_str(&self.path);

        if let Some(ref query) = self.query {
            result.push('?');
            result.push_str(query);
        }

        if let Some(ref fragment) = self.fragment {
            result.push('#');
            result.push_str(fragment);
        }

        result
    }
}

/// Parse scheme from URL
fn parse_scheme(url: &str) -> Result<(String, &str), &'static str> {
    if let Some(pos) = url.find("://") {
        let scheme = &url[..pos];

        // Validate scheme (alphanumeric + hyphen)
        if scheme.is_empty() || !scheme.chars().all(|c| c.is_alphanumeric() || c == '-') {
            return Err("Invalid scheme");
        }

        Ok((String::from(scheme), &url[pos + 3..]))
    } else if let Some(pos) = url.find(':') {
        let scheme = &url[..pos];
        Ok((String::from(scheme), &url[pos + 1..]))
    } else {
        Err("Missing scheme")
    }
}

/// Parse authority (host:port)
fn parse_authority(authority: &str) -> Result<(String, Option<u16>), &'static str> {
    if authority.is_empty() {
        return Err("Empty authority");
    }

    if let Some(pos) = authority.rfind(':') {
        let host = &authority[..pos];
        let port_str = &authority[pos + 1..];

        if host.is_empty() {
            return Err("Empty host");
        }

        let port = port_str.parse::<u16>()
            .map_err(|_| "Invalid port")?;

        Ok((String::from(host), Some(port)))
    } else {
        Ok((String::from(authority), None))
    }
}

/// Split string at character
fn split_at_char(s: &str, ch: char) -> (&str, &str) {
    if let Some(pos) = s.find(ch) {
        (&s[..pos], &s[pos + 1..])
    } else {
        (s, "")
    }
}

/// Split string at character, returning Option for second part
fn split_at_char_opt(s: &str, ch: char) -> (&str, Option<&str>) {
    if let Some(pos) = s.find(ch) {
        (&s[..pos], Some(&s[pos + 1..]))
    } else {
        (s, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_url() {
        let url = Url::parse("http://example.com/path").unwrap();
        assert_eq!(url.scheme, "http");
        assert_eq!(url.host, Some(String::from("example.com")));
        assert_eq!(url.port, None);
        assert_eq!(url.path, "/path");
    }

    #[test]
    fn test_parse_url_with_port() {
        let url = Url::parse("http://example.com:8080/path").unwrap();
        assert_eq!(url.port, Some(8080));
    }

    #[test]
    fn test_parse_url_with_query() {
        let url = Url::parse("http://example.com/path?key=value").unwrap();
        assert_eq!(url.query, Some(String::from("key=value")));
    }

    #[test]
    fn test_file_url() {
        let url = Url::parse("file:///home/user/file.html").unwrap();
        assert_eq!(url.scheme, "file");
        assert_eq!(url.path, "/home/user/file.html");
    }
}
