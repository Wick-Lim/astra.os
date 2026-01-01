// Resource Loader for ASTRA.OS Browser
// Handles loading resources from network and filesystem

use alloc::string::String;
use alloc::vec::Vec;
use crate::network::{url::Url, http::{HttpRequest, HttpResponse, parse_response}};
use crate::fs::vfs;

/// Resource type detected from URL or content
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ResourceType {
    Html,
    Css,
    JavaScript,
    Image,
    Unknown,
}

impl ResourceType {
    /// Detect resource type from URL path
    pub fn from_url(url: &Url) -> Self {
        let path = url.path.to_lowercase();

        if path.ends_with(".html") || path.ends_with(".htm") {
            ResourceType::Html
        } else if path.ends_with(".css") {
            ResourceType::Css
        } else if path.ends_with(".js") {
            ResourceType::JavaScript
        } else if path.ends_with(".png") || path.ends_with(".jpg") ||
                  path.ends_with(".jpeg") || path.ends_with(".gif") {
            ResourceType::Image
        } else {
            ResourceType::Unknown
        }
    }

    /// Detect resource type from content-type header
    pub fn from_content_type(content_type: &str) -> Self {
        let content_type = content_type.to_lowercase();

        if content_type.contains("text/html") {
            ResourceType::Html
        } else if content_type.contains("text/css") {
            ResourceType::Css
        } else if content_type.contains("javascript") || content_type.contains("application/js") {
            ResourceType::JavaScript
        } else if content_type.contains("image/") {
            ResourceType::Image
        } else {
            ResourceType::Unknown
        }
    }
}

/// Loaded resource
#[derive(Debug, Clone)]
pub struct Resource {
    pub url: Url,
    pub resource_type: ResourceType,
    pub data: Vec<u8>,
}

impl Resource {
    pub fn new(url: Url, data: Vec<u8>) -> Self {
        let resource_type = ResourceType::from_url(&url);
        Resource {
            url,
            resource_type,
            data,
        }
    }

    /// Get content as string (if UTF-8)
    pub fn as_string(&self) -> Option<String> {
        core::str::from_utf8(&self.data)
            .ok()
            .map(String::from)
    }
}

/// Resource loader
pub struct ResourceLoader {
    // Cache of loaded resources
    // TODO: Implement actual cache with HashMap
}

impl ResourceLoader {
    pub fn new() -> Self {
        ResourceLoader {}
    }

    /// Load resource from URL
    pub fn load(&self, url: &Url) -> Result<Resource, &'static str> {
        match url.scheme.as_str() {
            "file" => self.load_file(url),
            "http" | "https" => self.load_http(url),
            _ => Err("Unsupported URL scheme"),
        }
    }

    /// Load resource from filesystem
    fn load_file(&self, url: &Url) -> Result<Resource, &'static str> {
        // Remove leading slash for VFS
        let path = if url.path.starts_with('/') {
            &url.path[1..]
        } else {
            &url.path
        };

        // Open file
        let fd = vfs::open(path)?;

        // Read entire file
        let mut data = Vec::new();
        let mut buffer = [0u8; 512];

        loop {
            let bytes_read = vfs::read(fd, &mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            data.extend_from_slice(&buffer[..bytes_read]);
        }

        vfs::close(fd)?;

        Ok(Resource::new(url.clone(), data))
    }

    /// Load resource via HTTP
    fn load_http(&self, url: &Url) -> Result<Resource, &'static str> {
        // For now, return a stub implementation
        // TODO: Implement actual HTTP networking with TCP sockets

        crate::serial_println!("[HTTP] Would fetch: {}", url.to_string());

        // Stub: return empty resource
        Err("HTTP networking not yet implemented")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_type_detection() {
        let url = Url::parse("http://example.com/style.css").unwrap();
        assert_eq!(ResourceType::from_url(&url), ResourceType::Css);

        let url = Url::parse("http://example.com/index.html").unwrap();
        assert_eq!(ResourceType::from_url(&url), ResourceType::Html);
    }

    #[test]
    fn test_content_type_detection() {
        assert_eq!(
            ResourceType::from_content_type("text/html; charset=utf-8"),
            ResourceType::Html
        );

        assert_eq!(
            ResourceType::from_content_type("text/css"),
            ResourceType::Css
        );
    }
}
