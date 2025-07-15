//! TRN parsing functionality
//!
//! This module provides high-performance parsing of TRN strings using regex
//! with comprehensive error handling and zero-copy optimization.

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

    // Split TRN into main part and hash
    let (main_part, hash) = if let Some(at_pos) = input.find('@') {
        (&input[..at_pos], Some(&input[at_pos + 1..]))
    } else {
        (input, None)
    };

    // Split main part by colons - expect exactly 9 parts for fixed structure
    let parts: Vec<&str> = main_part.split(':').collect();

    if parts.len() != TRN_FIXED_COMPONENT_COUNT {
        return Err(TrnError::format(
            format!(
                "TRN must have exactly {} components (trn:platform:scope:resource_type:type:subtype:instance_id:version:tag), found {}",
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

    // Parse fixed structure: trn:platform:scope:resource_type:type:subtype:instance_id:version:tag
    let platform = parts[1];
    let scope = if parts[2].is_empty() { None } else { Some(parts[2]) };
    let resource_type = parts[3];
    let type_ = parts[4];
    let subtype = if parts[5].is_empty() { None } else { Some(parts[5]) };
    let instance_id = parts[6];
    let version = parts[7];
    let tag = if parts[8].is_empty() { None } else { Some(parts[8]) };

    // Create and validate the TRN
    let trn = Trn::new_full(
        platform.to_string(),
        scope.map(String::from),
        resource_type.to_string(),
        type_.to_string(),
        subtype.map(String::from),
        instance_id.to_string(),
        version.to_string(),
        tag.map(String::from),
        hash.map(String::from),
    )?;

    Ok(trn)
}

/// Parse TRN components from a string (zero-copy)
pub fn parse_trn_components(input: &str) -> TrnResult<TrnComponents<'_>> {
    // Split TRN into main part and hash
    let (main_part, hash) = if let Some(at_pos) = input.find('@') {
        (&input[..at_pos], Some(&input[at_pos + 1..]))
    } else {
        (input, None)
    };
    
    // Split main part by colons - expect exactly 9 parts for fixed structure
    let parts: Vec<&str> = main_part.split(':').collect();
    
    if parts.len() != TRN_FIXED_COMPONENT_COUNT {
        return Err(TrnError::format(
            format!(
                "TRN must have exactly {} components (trn:platform:scope:resource_type:type:subtype:instance_id:version:tag), found {}",
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
    
    // Parse fixed structure: trn:platform:scope:resource_type:type:subtype:instance_id:version:tag
    let platform = parts[1];
    let scope = if parts[2].is_empty() { None } else { Some(parts[2]) };
    let resource_type = parts[3];
    let type_ = parts[4];
    let subtype = if parts[5].is_empty() { None } else { Some(parts[5]) };
    let instance_id = parts[6];
    let version = parts[7];
    let tag = if parts[8].is_empty() { None } else { Some(parts[8]) };
    
    Ok(TrnComponents {
        platform,
        scope,
        resource_type,
        type_,
        subtype,
        instance_id,
        version,
        tag,
        hash,
    })
}

/// Normalize TRN string
pub fn normalize_trn(input: &str) -> TrnResult<String> {
    // First try to parse as-is
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

/// Analyze parsing errors for better diagnostics
#[allow(dead_code)]
fn analyze_parse_error(input: &str) -> TrnError {
    if !input.starts_with("trn:") {
        return TrnError::format("TRN must start with 'trn:'".to_string(), Some(input.to_string()));
    }
    
    let parts: Vec<&str> = input.split(':').collect();
    if parts.len() < 6 {
        return TrnError::format(
            format!("TRN requires at least 6 components, found {}", parts.len()),
            Some(input.to_string()),
        );
    }
    
    TrnError::format("Unknown parsing error".to_string(), Some(input.to_string()))
}

/// Check if input looks like a TRN
#[allow(dead_code)]
pub fn is_trn_format(input: &str) -> bool {
    input.starts_with("trn:") && input.matches(':').count() >= 5
}

/// Parse base TRN (without tag and hash)
#[allow(dead_code)]
pub fn parse_base_trn(input: &str) -> TrnResult<Trn> {
    let trn = parse_trn(input)?;
    Ok(trn.base_trn())
}

/// Extract normalized components
#[allow(dead_code)]
pub fn extract_components(input: &str) -> TrnResult<(String, Option<String>, String, String, Option<String>, String, String, Option<String>, Option<String>)> {
    let components = parse_trn_components(input)?;
    Ok((
        components.platform.to_string(),
        components.scope.map(String::from),
        components.resource_type.to_string(),
        components.type_.to_string(),
        components.subtype.map(String::from),
        components.instance_id.to_string(),
        components.version.to_string(),
        components.tag.map(String::from),
        components.hash.map(String::from),
    ))
}

/// Extract base TRN string
#[allow(dead_code)]
pub fn extract_base_trn(input: &str) -> TrnResult<String> {
    let trn = parse_trn(input)?;
    Ok(trn.base_trn().to_string())
}

/// Check if a component looks like a TRN
#[allow(dead_code)]
pub fn is_trn_like(input: &str) -> bool {
    TRN_REGEX.is_match(input)
}

/// Parse multiple TRNs from a string
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

/// Parse TRN URL format
fn parse_trn_url(url: &str) -> TrnResult<Trn> {
    if !url.starts_with("trn://") {
        return Err(TrnError::format(
            "URL must start with trn://".to_string(),
            Some(url.to_string()),
        ));
    }
    
    // Remove trn:// prefix
    let path = &url[6..];
    
    // Split query parameters if any
    let (path, query) = if let Some(q_pos) = path.find('?') {
        (&path[..q_pos], Some(&path[q_pos + 1..]))
    } else {
        (path, None)
    };
    
    // Parse hash from query parameters
    let hash = if let Some(query_str) = query {
        parse_query_params(query_str).get("hash").cloned()
    } else {
        None
    };
    
    // Split path components - expect exactly 8 for fixed structure
    let path_parts: Vec<&str> = path.split('/').collect();
    
    // Remove leading and trailing empty strings from URL parsing
    let mut start_idx = 0;
    let mut end_idx = path_parts.len();
    
    if !path_parts.is_empty() && path_parts[0].is_empty() {
        start_idx = 1;
    }
    if !path_parts.is_empty() && path_parts[path_parts.len() - 1].is_empty() {
        end_idx -= 1;
    }
    
    let actual_parts = &path_parts[start_idx..end_idx];
    
    if actual_parts.len() != 8 {
        return Err(TrnError::format(
            format!(
                "TRN URL requires exactly 8 path components (platform/scope/resource_type/type/subtype/instance_id/version/tag), found {}",
                actual_parts.len()
            ),
            Some(url.to_string()),
        ));
    }
    
    // Decode URL components and build TRN string with fixed structure
    let decoded_parts: Result<Vec<String>, _> = actual_parts
        .iter()
        .map(|part| percent_decode_str(part).decode_utf8().map(|s| s.to_string()))
        .collect();
    
    let decoded_parts = decoded_parts.map_err(|e| TrnError::format(
        format!("Failed to decode URL components: {}", e),
        Some(url.to_string()),
    ))?;
    
    // Build TRN string with fixed structure: trn:platform:scope:resource_type:type:subtype:instance_id:version:tag
    let mut trn_str = format!("trn:{}", decoded_parts.join(":"));
    if let Some(h) = hash {
        trn_str.push('@');
        trn_str.push_str(&h);
    }
    
    parse_trn(&trn_str)
}

/// Parse query parameters
fn parse_query_params(query: &str) -> std::collections::HashMap<String, String> {
    let mut params = std::collections::HashMap::new();
    
    for pair in query.split('&') {
        if let Some(eq_pos) = pair.find('=') {
            let key = &pair[..eq_pos];
            let value = &pair[eq_pos + 1..];
            
            // URL decode key and value
            if let (Ok(decoded_key), Ok(decoded_value)) = (
                percent_decode_str(key).decode_utf8(),
                percent_decode_str(value).decode_utf8(),
            ) {
                params.insert(decoded_key.to_string(), decoded_value.to_string());
            }
        }
    }
    
    params
}

/// Check if a component looks like a scope for the given platform
fn is_scope_like(value: &str, platform: &str) -> bool {
    match platform {
        "user" => value.len() >= 2 && value.chars().all(|c| c.is_alphanumeric()),
        "org" => value.len() >= 2,
        "aiplatform" => false, // aiplatform typically doesn't have scope
        _ => value.len() <= 32 && value.chars().all(|c| c.is_alphanumeric() || c == '-'),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_parsing() {
        let trn = parse_trn("trn:user:alice:tool:openapi::getUserById:v1.0:").unwrap();
        assert_eq!(trn.platform(), "user");
        assert_eq!(trn.scope(), Some("alice"));
        assert_eq!(trn.resource_type(), "tool");
        assert_eq!(trn.type_(), "openapi");
        assert_eq!(trn.subtype(), None);
        assert_eq!(trn.instance_id(), "getUserById");
        assert_eq!(trn.version(), "v1.0");
        assert_eq!(trn.tag(), None);
    }

    #[test]
    fn test_minimal_parsing() {
        let trn = parse_trn("trn:aiplatform::tool:workflow::processData:latest:").unwrap();
        assert_eq!(trn.platform(), "aiplatform");
        assert_eq!(trn.scope(), None);
        assert_eq!(trn.resource_type(), "tool");
        assert_eq!(trn.type_(), "workflow");
        assert_eq!(trn.subtype(), None);
        assert_eq!(trn.instance_id(), "processData");
        assert_eq!(trn.version(), "latest");
        assert_eq!(trn.tag(), None);
    }

    #[test]
    fn test_full_parsing() {
        let input = "trn:org:company:tool:openapi:async:createComplexOperation:v2.1.3:stable@sha256:1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";
        let trn = parse_trn(input).unwrap();
        assert_eq!(trn.platform(), "org");
        assert_eq!(trn.scope(), Some("company"));
        assert_eq!(trn.resource_type(), "tool");
        assert_eq!(trn.type_(), "openapi");
        assert_eq!(trn.subtype(), Some("async"));
        assert_eq!(trn.instance_id(), "createComplexOperation");
        assert_eq!(trn.version(), "v2.1.3");
        assert_eq!(trn.tag(), Some("stable"));
        assert_eq!(trn.hash(), Some("sha256:1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"));
    }

    #[test]
    fn test_parsing_errors() {
        assert!(parse_trn("invalid").is_err());
        assert!(parse_trn("trn:invalid").is_err());
        assert!(parse_trn("trn:user:alice").is_err());
        assert!(parse_trn("trn:user:alice:tool:openapi:getUserById:v1.0").is_err()); // Missing components
    }

    #[test]
    fn test_normalization() {
        let normalized = normalize_trn("trn:user:alice:tool:openapi::getUserById:v1.0:").unwrap();
        assert_eq!(normalized, "trn:user:alice:tool:openapi::getUserById:v1.0:");
    }

    #[test]
    fn test_url_parsing() {
        let url = "trn://user/alice/tool/openapi//getUserById/v1.0//";
        let trn = parse_trn_from_url(url).unwrap();
        assert_eq!(trn.platform(), "user");
        assert_eq!(trn.scope(), Some("alice"));
        assert_eq!(trn.resource_type(), "tool");
        assert_eq!(trn.type_(), "openapi");
        assert_eq!(trn.subtype(), None);
        assert_eq!(trn.instance_id(), "getUserById");
        assert_eq!(trn.version(), "v1.0");
        assert_eq!(trn.tag(), None);
    }

    #[test]
    fn test_zero_copy_parsing() {
        let input = "trn:user:alice:tool:openapi::getUserById:v1.0:";
        let components = parse_trn_components(input).unwrap();
        assert_eq!(components.platform, "user");
        assert_eq!(components.scope, Some("alice"));
        assert_eq!(components.instance_id, "getUserById");
    }

    #[test]
    fn test_extract_base_trn() {
        let input = "trn:user:alice:tool:openapi::getUserById:v1.0:stable@sha256:1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";
        let base = extract_base_trn(input).unwrap();
        assert_eq!(base, "trn:user:alice:tool:openapi::getUserById:*:");
    }

    #[test]
    fn test_multiple_trns() {
        let input = "trn:user:alice:tool:openapi::getUserById:v1.0:\ntrn:user:bob:tool:python::runScript:v2.0:";
        let results = parse_multiple_trns(input);
        assert_eq!(results.len(), 2);
        assert!(results[0].is_ok());
        assert!(results[1].is_ok());
    }
} 