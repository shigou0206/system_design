//! URL conversion functionality
//!
//! This module provides bidirectional conversion between TRN strings and URL formats,
//! including trn:// URLs and HTTP URLs for web-based access.

use percent_encoding::{utf8_percent_encode, percent_decode_str, CONTROLS, AsciiSet};
use url::Url;

use crate::error::{TrnError, TrnResult};
use crate::types::{Trn, TrnComponents};

/// Define a safe encoding set for TRN URL components
/// Only encode characters that are problematic in URLs, preserve safe characters like - and .
const TRN_COMPONENT_ENCODE_SET: &AsciiSet = &CONTROLS.add(b' ').add(b'/').add(b'?').add(b'#').add(b'[').add(b']').add(b'@').add(b'!').add(b'$').add(b'&').add(b'\'').add(b'(').add(b')').add(b'*').add(b'+').add(b',').add(b';').add(b'=');

/// Convert a TRN to trn:// URL format
pub fn trn_to_url(trn: &Trn) -> TrnResult<String> {
    let path_parts = vec![
        trn.platform(),
        trn.scope(),
        trn.resource_type(),
        trn.resource_id(),
        trn.version(),
    ];
    
    // URL encode each path component
    let encoded_parts: Vec<String> = path_parts
        .iter()
        .map(|part| url_encode_component(part))
        .collect();
    
    let url = format!("trn://{}", encoded_parts.join("/"));
    
    Ok(url)
}

/// Convert a TRN to HTTP URL format
pub fn trn_to_http_url(trn: &Trn, base_url: &str) -> TrnResult<String> {
    // Validate base URL
    let base = Url::parse(base_url)
        .map_err(|e| TrnError::url(
            format!("Invalid base URL: {}", e),
            Some(base_url.to_string()),
        ))?;
    
    let path_parts = vec![
        "trn",
        trn.platform(),
        trn.scope(),
        trn.resource_type(),
        trn.resource_id(),
        trn.version(),
    ];
    
    // URL encode each path component
    let encoded_parts: Vec<String> = path_parts
        .iter()
        .map(|part| url_encode_component(part))
        .collect();
    
    let path = encoded_parts.join("/");
    
    let url = base.join(&path)
        .map_err(|e| TrnError::url(
            format!("Failed to join path with base URL: {}", e),
            Some(base_url.to_string()),
        ))?;
    
    Ok(url.to_string())
}

/// Convert a trn:// URL back to TRN string
pub fn url_to_trn(url: &str) -> TrnResult<Trn> {
    if !url.starts_with("trn://") {
        return Err(TrnError::url(
            "URL must use trn:// scheme",
            Some(url.to_string()),
        ));
    }
    
    crate::parsing::parse_trn_from_url(url)
}

/// Convert an HTTP URL back to TRN string
#[allow(dead_code)]
pub fn http_url_to_trn(url: &str) -> TrnResult<Trn> {
    let parsed_url = Url::parse(url)
        .map_err(|e| TrnError::url(
            format!("Invalid URL: {}", e),
            Some(url.to_string()),
        ))?;
    
    let path = parsed_url.path();
    
    // Check if path starts with /trn/
    if !path.starts_with("/trn/") {
        return Err(TrnError::url(
            "HTTP URL path must start with /trn/",
            Some(url.to_string()),
        ));
    }
    
    // Remove /trn/ prefix
    let trn_path = &path[5..];
    
    // Split path into components
    let path_parts: Vec<&str> = trn_path
        .split('/')
        .filter(|s| !s.is_empty())
        .collect();
    
    if path_parts.len() < 5 {
        return Err(TrnError::url(
            "HTTP URL requires at least 5 TRN path components",
            Some(url.to_string()),
        ));
    }
    
    // Decode URL components
    let decoded_parts: Result<Vec<String>, _> = path_parts
        .iter()
        .map(|part| url_decode_component(part))
        .collect();
    
    let decoded_parts = decoded_parts.map_err(|e| TrnError::url(
        format!("Failed to decode URL components: {}", e),
        Some(url.to_string()),
    ))?;
    
    // For the new 6-component format, we expect exactly 5 path components
    // Format: trn://platform/scope/resource_type/resource_id/version
    if decoded_parts.len() != 5 {
        return Err(TrnError::url(
            format!("Invalid trn:// URL format. Expected 5 path components, got {}", decoded_parts.len()),
            Some(url.to_string()),
        ));
    }
    
    // Build TRN components for the fixed format
    let components = TrnComponents {
        platform: &decoded_parts[0],
        scope: &decoded_parts[1],
        resource_type: &decoded_parts[2],
        resource_id: &decoded_parts[3],
        version: &decoded_parts[4],
    };
    
    Ok(components.to_owned())
}

/// URL encode a TRN component
fn url_encode_component(component: &str) -> String {
    utf8_percent_encode(component, TRN_COMPONENT_ENCODE_SET).to_string()
}

/// URL decode a TRN component
fn url_decode_component(component: &str) -> Result<String, std::str::Utf8Error> {
    let decoded = percent_decode_str(component).decode_utf8()?;
    Ok(decoded.to_string())
}

/// Heuristic to determine if a component looks like a scope
#[allow(dead_code)]
fn is_scope_like(value: &str, platform: &str) -> bool {
    match platform {
        "user" => value.len() >= 2 && value.chars().all(|c| c.is_alphanumeric()),
        "org" => value.len() >= 2,
        "aiplatform" => false,
        _ => value.len() <= 32 && value.chars().all(|c| c.is_alphanumeric() || c == '-'),
    }
}

/// Build a TRN URL with custom query parameters
#[allow(dead_code)]
pub fn build_trn_url_with_params(trn: &Trn, params: &[(&str, &str)]) -> TrnResult<String> {
    let base_url = trn_to_url(trn)?;
    
    // Parse existing URL to handle existing query parameters
    let url = Url::parse(&base_url)
        .map_err(|e| TrnError::url(
            format!("Failed to parse TRN URL: {}", e),
            Some(base_url.clone()),
        ))?;
    
    let mut new_url = url.clone();
    
    // Add custom parameters
    {
        let mut query_pairs = new_url.query_pairs_mut();
        for (key, value) in params {
            query_pairs.append_pair(key, value);
        }
    }
    
    Ok(new_url.to_string())
}

/// Extract query parameters from a TRN URL
#[allow(dead_code)]
pub fn extract_url_params(url: &str) -> TrnResult<std::collections::HashMap<String, String>> {
    let parsed_url = Url::parse(url)
        .map_err(|e| TrnError::url(
            format!("Invalid URL: {}", e),
            Some(url.to_string()),
        ))?;
    
    let mut params = std::collections::HashMap::new();
    
    for (key, value) in parsed_url.query_pairs() {
        params.insert(key.to_string(), value.to_string());
    }
    
    Ok(params)
}

/// Normalize a URL by removing unnecessary components
#[allow(dead_code)]
pub fn normalize_url(url: &str) -> TrnResult<String> {
    if url.starts_with("trn://") {
        // Parse TRN URL, convert to TRN, and back to URL for normalization
        let trn = url_to_trn(url)?;
        trn_to_url(&trn)
    } else {
        // For HTTP URLs, just parse and re-serialize
        let parsed_url = Url::parse(url)
            .map_err(|e| TrnError::url(
                format!("Invalid URL: {}", e),
                Some(url.to_string()),
            ))?;
        
        Ok(parsed_url.to_string())
    }
}

/// Check if a URL is a valid TRN URL
#[allow(dead_code)]
pub fn is_valid_trn_url(url: &str) -> bool {
    url_to_trn(url).is_ok()
}

/// Check if a URL is a valid HTTP TRN URL
#[allow(dead_code)]
pub fn is_valid_http_trn_url(url: &str) -> bool {
    http_url_to_trn(url).is_ok()
}

/// Convert between different URL formats
#[allow(dead_code)]
pub fn convert_url_format(url: &str, target_format: UrlFormat, base_url: Option<&str>) -> TrnResult<String> {
    // First, determine the source format and parse to TRN
    let trn = if url.starts_with("trn://") {
        url_to_trn(url)?
    } else if url.starts_with("http://") || url.starts_with("https://") {
        http_url_to_trn(url)?
    } else {
        return Err(TrnError::url(
            "Unsupported URL format",
            Some(url.to_string()),
        ));
    };
    
    // Convert to target format
    match target_format {
        UrlFormat::TrnUrl => trn_to_url(&trn),
        UrlFormat::HttpUrl => {
            let base = base_url.ok_or_else(|| TrnError::url(
                "Base URL required for HTTP URL conversion",
                None,
            ))?;
            trn_to_http_url(&trn, base)
        }
    }
}

/// URL format enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UrlFormat {
    /// trn:// URL format
    TrnUrl,
    /// HTTP/HTTPS URL format
    HttpUrl,
}

/// URL validation result
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct UrlValidationResult {
    /// Whether the URL is valid
    pub is_valid: bool,
    /// The parsed TRN (if valid)
    pub trn: Option<Trn>,
    /// Validation error (if invalid)
    pub error: Option<String>,
    /// URL format
    pub format: Option<UrlFormat>,
    /// Normalized URL
    pub normalized_url: Option<String>,
}

/// Comprehensive URL validation
#[allow(dead_code)]
pub fn validate_url(url: &str) -> UrlValidationResult {
    // Try TRN URL format
    if url.starts_with("trn://") {
        match url_to_trn(url) {
            Ok(trn) => UrlValidationResult {
                is_valid: true,
                trn: Some(trn.clone()),
                error: None,
                format: Some(UrlFormat::TrnUrl),
                normalized_url: trn_to_url(&trn).ok(),
            },
            Err(e) => UrlValidationResult {
                is_valid: false,
                trn: None,
                error: Some(e.to_string()),
                format: Some(UrlFormat::TrnUrl),
                normalized_url: None,
            },
        }
    }
    // Try HTTP URL format
    else if url.starts_with("http://") || url.starts_with("https://") {
        match http_url_to_trn(url) {
            Ok(trn) => UrlValidationResult {
                is_valid: true,
                trn: Some(trn.clone()),
                error: None,
                format: Some(UrlFormat::HttpUrl),
                normalized_url: Some(url.to_string()),
            },
            Err(e) => UrlValidationResult {
                is_valid: false,
                trn: None,
                error: Some(e.to_string()),
                format: Some(UrlFormat::HttpUrl),
                normalized_url: None,
            },
        }
    }
    // Unknown format
    else {
        UrlValidationResult {
            is_valid: false,
            trn: None,
            error: Some("Unsupported URL format".to_string()),
            format: None,
            normalized_url: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Trn;

    #[test]
    fn test_trn_to_url() {
        let trn = Trn::parse("trn:user:alice:tool:myapi:v1.0").unwrap();
        let url = trn_to_url(&trn).unwrap();
        assert_eq!(url, "trn://user/alice/tool/myapi/v1.0");
    }

    #[test]
    fn test_trn_to_url_with_hash() {
        // Note: Hash not supported in 6-component format
        let trn = Trn::parse("trn:user:alice:tool:myapi:v1.0").unwrap();
        let url = trn_to_url(&trn).unwrap();
        assert_eq!(url, "trn://user/alice/tool/myapi/v1.0");
    }

    #[test]
    fn test_trn_to_http_url() {
        let trn = Trn::parse("trn:user:alice:tool:myapi:v1.0").unwrap();
        let url = trn_to_http_url(&trn, "https://api.example.com/").unwrap();
        assert_eq!(url, "https://api.example.com/trn/user/alice/tool/myapi/v1.0");
    }

    #[test]
    fn test_url_to_trn() {
        let url = "trn://user/alice/tool/myapi/v1.0";
        let trn = url_to_trn(url).unwrap();
        assert_eq!(trn.platform(), "user");
        assert_eq!(trn.scope(), "alice");
        assert_eq!(trn.resource_id(), "myapi");
    }

    #[test]
    fn test_http_url_to_trn() {
        let url = "https://api.example.com/trn/user/alice/tool/myapi/v1.0";
        let trn = http_url_to_trn(url).unwrap();
        assert_eq!(trn.platform(), "user");
        assert_eq!(trn.scope(), "alice");
    }

    #[test]
    fn test_url_encoding() {
        let component = "test-component";
        let encoded = url_encode_component(component);
        let decoded = url_decode_component(&encoded).unwrap();
        assert_eq!(component, decoded);
    }

    #[test]
    fn test_bidirectional_conversion() {
        let original_trn = "trn:user:alice:tool:myapi:v1.0";
        let trn = Trn::parse(original_trn).unwrap();
        let url = trn_to_url(&trn).unwrap();
        let back_to_trn = url_to_trn(&url).unwrap();
        assert_eq!(trn, back_to_trn);
    }

    #[test]
    fn test_url_validation() {
        let valid_url = "trn://user/alice/tool/myapi/v1.0";
        let result = validate_url(valid_url);
        assert!(result.is_valid);
        assert!(result.trn.is_some());

        let invalid_url = "trn://invalid";
        let result = validate_url(invalid_url);
        assert!(!result.is_valid);
        assert!(result.error.is_some());
    }

    #[test]
    fn test_convert_url_format() {
        let trn_url = "trn://user/alice/tool/myapi/v1.0";
        let http_url = convert_url_format(
            trn_url, 
            UrlFormat::HttpUrl, 
            Some("https://api.example.com/")
        ).unwrap();
        
        assert!(http_url.starts_with("https://api.example.com/trn/"));
        
        // Convert back
        let back_to_trn_url = convert_url_format(
            &http_url,
            UrlFormat::TrnUrl,
            None
        ).unwrap();
        
        assert_eq!(trn_url, back_to_trn_url);
    }
} 