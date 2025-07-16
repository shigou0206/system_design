//! TRN parsing functionality
//!
//! This module provides high-performance parsing of TRN strings using regex
//! with comprehensive error handling and zero-copy optimization for the
//! simplified 6-component format.

use percent_encoding::percent_decode_str;

use crate::constants::*;
use crate::error::{TrnError, TrnResult};
use crate::types::{Trn, TrnComponents};

/// Parse TRN string into a TRN object
pub fn parse_trn(input: &str) -> TrnResult<Trn> {
    // Basic validation
    if input.is_empty() {
        return Err(TrnError::format(
            "Empty TRN string".to_string(),
            Some(input.to_string()),
        ));
    }

    if !input.starts_with("trn:") {
        return Err(TrnError::format(
            "TRN must start with 'trn:'".to_string(),
            Some(input.to_string()),
        ));
    }

    // Split TRN by colons - expect exactly 6 parts for simplified structure
    let parts: Vec<&str> = input.split(':').collect();

    if parts.len() != TRN_FIXED_COMPONENT_COUNT {
        return Err(TrnError::format(
            format!(
                "TRN must have exactly {} components (trn:platform:scope:resource_type:resource_id:version), found {}",
                TRN_FIXED_COMPONENT_COUNT,
                parts.len()
            ),
            Some(input.to_string()),
        ));
    }

    if parts[0] != "trn" {
        return Err(TrnError::format(
            "TRN must start with 'trn:'".to_string(),
            Some(input.to_string()),
        ));
    }

    // Parse simplified structure: trn:platform:scope:resource_type:resource_id:version
    let platform = parts[1];
    let scope = parts[2];
    let resource_type = parts[3];
    let resource_id = parts[4];
    let version = parts[5];

    // Validate all components are non-empty
    if platform.is_empty() {
        return Err(TrnError::component(
            "Platform cannot be empty".to_string(),
            "platform".to_string(),
            Some(input.to_string()),
        ));
    }

    if scope.is_empty() {
        return Err(TrnError::component(
            "Scope cannot be empty".to_string(),
            "scope".to_string(),
            Some(input.to_string()),
        ));
    }

    if resource_type.is_empty() {
        return Err(TrnError::component(
            "Resource type cannot be empty".to_string(),
            "resource_type".to_string(),
            Some(input.to_string()),
        ));
    }

    if resource_id.is_empty() {
        return Err(TrnError::component(
            "Resource ID cannot be empty".to_string(),
            "resource_id".to_string(),
            Some(input.to_string()),
        ));
    }

    if version.is_empty() {
        return Err(TrnError::component(
            "Version cannot be empty".to_string(),
            "version".to_string(),
            Some(input.to_string()),
        ));
    }

    // Create and validate the TRN
    let trn = Trn::new(
        platform.to_string(),
        scope.to_string(),
        resource_type.to_string(),
        resource_id.to_string(),
        version.to_string(),
    )?;

    Ok(trn)
}

/// Parse TRN components from a string (zero-copy)
pub fn parse_trn_components(input: &str) -> TrnResult<TrnComponents<'_>> {
    // Split TRN by colons - expect exactly 6 parts for simplified structure
    let parts: Vec<&str> = input.split(':').collect();
    
    if parts.len() != TRN_FIXED_COMPONENT_COUNT {
        return Err(TrnError::format(
            format!(
                "TRN must have exactly {} components (trn:platform:scope:resource_type:resource_id:version), found {}",
                TRN_FIXED_COMPONENT_COUNT,
                parts.len()
            ),
            Some(input.to_string()),
        ));
    }
    
    if parts[0] != "trn" {
        return Err(TrnError::format(
            "TRN must start with 'trn:'".to_string(),
            Some(input.to_string()),
        ));
    }
    
    // Parse simplified structure: trn:platform:scope:resource_type:resource_id:version
    let platform = parts[1];
    let scope = parts[2];
    let resource_type = parts[3];
    let resource_id = parts[4];
    let version = parts[5];

    // Validate all components are non-empty
    if platform.is_empty() {
        return Err(TrnError::component(
            "Platform cannot be empty".to_string(),
            "platform".to_string(),
            Some(input.to_string()),
        ));
    }

    if scope.is_empty() {
        return Err(TrnError::component(
            "Scope cannot be empty".to_string(),
            "scope".to_string(),
            Some(input.to_string()),
        ));
    }

    if resource_type.is_empty() {
        return Err(TrnError::component(
            "Resource type cannot be empty".to_string(),
            "resource_type".to_string(),
            Some(input.to_string()),
        ));
    }

    if resource_id.is_empty() {
        return Err(TrnError::component(
            "Resource ID cannot be empty".to_string(),
            "resource_id".to_string(),
            Some(input.to_string()),
        ));
    }

    if version.is_empty() {
        return Err(TrnError::component(
            "Version cannot be empty".to_string(),
            "version".to_string(),
            Some(input.to_string()),
        ));
    }
    
    Ok(TrnComponents {
        platform,
        scope,
        resource_type,
        resource_id,
        version,
    })
}

/// Extract normalized components
#[allow(dead_code)]
pub fn extract_components(input: &str) -> TrnResult<(String, String, String, String, String)> {
    let components = parse_trn_components(input)?;
    Ok((
        components.platform.to_string(),
        components.scope.to_string(),
        components.resource_type.to_string(),
        components.resource_id.to_string(),
        components.version.to_string(),
    ))
}

/// Normalize TRN string
#[allow(dead_code)]
pub fn normalize_trn(input: &str) -> TrnResult<String> {
    // First try parsing as-is
    match parse_trn(input) {
        Ok(trn) => Ok(trn.to_string()),
        Err(_) => {
            // If that fails, try lowercasing first
            let lowercase_input = input.to_lowercase();
            let trn = parse_trn(&lowercase_input)?;
            Ok(trn.to_string())
        }
    }
}

/// Parse with error recovery
#[allow(dead_code)]
pub fn parse_trn_with_recovery(input: &str) -> TrnResult<Trn> {
    // Try main parser first
    match parse_trn(input) {
        Ok(trn) => Ok(trn),
        Err(_) => {
            // Try with normalization
            let normalized = input.to_lowercase();
            parse_trn(&normalized)
        }
    }
}

/// Extract base TRN string (replace version with wildcard)
#[allow(dead_code)]
pub fn extract_base_trn(input: &str) -> TrnResult<String> {
    let trn = parse_trn(input)?;
    Ok(trn.base_trn().to_string())
}

/// Check if input looks like a TRN
#[allow(dead_code)]
pub fn is_trn_format(input: &str) -> bool {
    input.starts_with("trn:") && input.matches(':').count() == 5
}

/// Parse base TRN (without version)
#[allow(dead_code)]
pub fn parse_base_trn(input: &str) -> TrnResult<Trn> {
    let trn = parse_trn(input)?;
    parse_trn(&trn.base_trn().to_string())
}

/// Check if string looks like a TRN (alternative name)
#[allow(dead_code)]
pub fn is_trn_like(input: &str) -> bool {
    is_trn_format(input)
}

/// Parse multiple TRN strings from input
#[allow(dead_code)]
pub fn parse_multiple_trns(input: &str) -> Vec<TrnResult<Trn>> {
    input
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| parse_trn(line.trim()))
        .collect()
}

/// Parse TRN from URL format
pub fn parse_trn_from_url(url: &str) -> TrnResult<Trn> {
    if url.starts_with("trn://") {
        parse_trn_url(url)
    } else {
        parse_trn(url)
    }
}

/// Parse TRN URL format for simplified structure
fn parse_trn_url(url: &str) -> TrnResult<Trn> {
    if !url.starts_with("trn://") {
        return Err(TrnError::format(
            "URL must start with trn://".to_string(),
            Some(url.to_string()),
        ));
    }
    
    // Remove trn:// prefix
    let path = &url[6..];
    
    // Remove trailing slash if present
    let path = path.strip_suffix('/').unwrap_or(path);
    
    // Split path components - expect exactly 5 for simplified structure
    let path_parts: Vec<&str> = path.split('/').collect();
    
    if path_parts.len() != 5 {
        return Err(TrnError::format(
            format!(
                "TRN URL requires exactly 5 path components (platform/scope/resource_type/resource_id/version), found {}",
                path_parts.len()
            ),
            Some(url.to_string()),
        ));
    }
    
    // Decode URL components and build TRN string with simplified structure
    let decoded_parts: Result<Vec<String>, _> = path_parts
        .iter()
        .map(|part| percent_decode_str(part).decode_utf8().map(|s| s.to_string()))
        .collect();
    
    let decoded_parts = decoded_parts.map_err(|e| TrnError::format(
        format!("Failed to decode URL components: {}", e),
        Some(url.to_string()),
    ))?;
    
    // Build TRN string with simplified structure: trn:platform:scope:resource_type:resource_id:version
    let trn_str = format!("trn:{}", decoded_parts.join(":"));
    
    parse_trn(&trn_str)
}

/// Analyze why TRN parsing failed
#[allow(dead_code)]
fn analyze_parse_error(input: &str) -> TrnError {
    if input.is_empty() {
        return TrnError::Format {
            message: "Empty TRN string".to_string(),
            trn: Some(input.to_string()),
        };
    }
    
    if !input.starts_with("trn:") {
        return TrnError::Format {
            message: "TRN must start with 'trn:'".to_string(),
            trn: Some(input.to_string()),
        };
    }
    
    let component_count = input.matches(':').count();
    if component_count != 5 {
        return TrnError::Format {
            message: format!(
                "TRN must have exactly 6 components (trn:platform:scope:resource_type:resource_id:version), found {}",
                component_count + 1
            ),
            trn: Some(input.to_string()),
        };
    }
    
    TrnError::Format {
        message: "Invalid TRN format".to_string(),
        trn: Some(input.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_trn() {
        let trn_str = "trn:user:alice:tool:myapi:v1.0";
        let trn = parse_trn(trn_str).unwrap();
        
        assert_eq!(trn.platform(), "user");
        assert_eq!(trn.scope(), "alice");
        assert_eq!(trn.resource_type(), "tool");
        assert_eq!(trn.resource_id(), "myapi");
        assert_eq!(trn.version(), "v1.0");
    }

    #[test]
    fn test_parse_invalid_component_count() {
        let trn_str = "trn:user:alice:tool"; // Only 4 components
        let result = parse_trn(trn_str);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_empty_component() {
        let trn_str = "trn:user::tool:myapi:v1.0"; // Empty scope
        let result = parse_trn(trn_str);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_trn_components() {
        let trn_str = "trn:user:alice:tool:myapi:v1.0";
        let components = parse_trn_components(trn_str).unwrap();
        
        assert_eq!(components.platform, "user");
        assert_eq!(components.scope, "alice");
        assert_eq!(components.resource_type, "tool");
        assert_eq!(components.resource_id, "myapi");
        assert_eq!(components.version, "v1.0");
    }

    #[test]
    fn test_normalize_trn() {
        let trn_str = "TRN:USER:ALICE:TOOL:MYAPI:V1.0";
        let normalized = normalize_trn(trn_str).unwrap();
        assert_eq!(normalized, "trn:user:alice:tool:myapi:v1.0");
    }

    #[test]
    fn test_is_trn_format() {
        assert!(is_trn_format("trn:user:alice:tool:myapi:v1.0"));
        assert!(!is_trn_format("invalid:format"));
        assert!(!is_trn_format("trn:too:few:components"));
    }

    #[test]
    fn test_parse_trn_url() {
        let url = "trn://user/alice/tool/myapi/v1.0";
        let trn = parse_trn_from_url(url).unwrap();
        
        assert_eq!(trn.platform(), "user");
        assert_eq!(trn.scope(), "alice");
        assert_eq!(trn.resource_type(), "tool");
        assert_eq!(trn.resource_id(), "myapi");
        assert_eq!(trn.version(), "v1.0");
    }

    #[test]
    fn test_extract_base_trn() {
        let trn_str = "trn:user:alice:tool:myapi:v1.0";
        let base = extract_base_trn(trn_str).unwrap();
        assert_eq!(base, "trn:user:alice:tool:myapi:*");
    }
} 