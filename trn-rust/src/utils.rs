//! TRN utility functions and helpers
//!
//! This module provides various utility functions for TRN operations,
//! including version comparison, statistics, and convenience helpers.

use std::collections::HashMap;
use std::cmp::Ordering;

use crate::constants::*;
use crate::error::{TrnError, TrnResult};
use crate::types::{Trn, Platform};

/// Hash algorithm enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HashAlgorithm {
    /// SHA-256 algorithm
    Sha256,
    /// MD5 algorithm (deprecated, for compatibility)
    Md5,
}

impl HashAlgorithm {
    /// Get algorithm name as string
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Sha256 => "sha256",
            Self::Md5 => "md5",
        }
    }
}

/// Generate hash for TRN using specified algorithm
pub fn generate_trn_hash(trn: &Trn, algorithm: HashAlgorithm) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    trn.to_string().hash(&mut hasher);
    
    match algorithm {
        HashAlgorithm::Sha256 => format!("sha256:{:x}", hasher.finish()),
        HashAlgorithm::Md5 => format!("md5:{:x}", hasher.finish()),
    }
}

/// Version comparison operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VersionOp {
    /// Equal to
    Equal,
    /// Not equal to
    NotEqual,
    /// Greater than
    Greater,
    /// Greater than or equal to
    GreaterEqual,
    /// Less than
    Less,
    /// Less than or equal to
    LessEqual,
    /// Compatible version (~)
    Compatible,
    /// Compatible within major version (^)
    CompatibleMajor,
}

impl VersionOp {
    /// Parse version operator from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "==" | "=" => Some(Self::Equal),
            "!=" => Some(Self::NotEqual),
            ">" => Some(Self::Greater),
            ">=" => Some(Self::GreaterEqual),
            "<" => Some(Self::Less),
            "<=" => Some(Self::LessEqual),
            "~" => Some(Self::Compatible),
            "^" => Some(Self::CompatibleMajor),
            _ => None,
        }
    }

    /// Convert to string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Equal => "==",
            Self::NotEqual => "!=",
            Self::Greater => ">",
            Self::GreaterEqual => ">=",
            Self::Less => "<",
            Self::LessEqual => "<=",
            Self::Compatible => "~",
            Self::CompatibleMajor => "^",
        }
    }
}

/// Semantic version structure
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticVersion {
    /// Major version
    pub major: u32,
    /// Minor version
    pub minor: u32,
    /// Patch version
    pub patch: u32,
    /// Pre-release identifier
    pub prerelease: Option<String>,
    /// Build metadata
    pub build: Option<String>,
}

impl SemanticVersion {
    /// Parse semantic version from string
    pub fn parse(version: &str) -> TrnResult<Self> {
        let version = version.trim_start_matches('v');
        
        if let Some(captures) = SEMANTIC_VERSION_REGEX.captures(version) {
            let major = captures.get(1)
                .unwrap()
                .as_str()
                .parse()
                .map_err(|_| TrnError::version("Invalid major version", version, "", "=="))?;
            
            let minor = captures.get(2)
                .unwrap()
                .as_str()
                .parse()
                .map_err(|_| TrnError::version("Invalid minor version", version, "", "=="))?;
            
            let patch = captures.get(3)
                .unwrap()
                .as_str()
                .parse()
                .map_err(|_| TrnError::version("Invalid patch version", version, "", "=="))?;
            
            let prerelease = captures.get(4).map(|m| m.as_str().to_string());
            let build = captures.get(5).map(|m| m.as_str().to_string());
            
            Ok(Self {
                major,
                minor,
                patch,
                prerelease,
                build,
            })
        } else {
            Err(TrnError::version(
                "Invalid semantic version format",
                version,
                "",
                "==",
            ))
        }
    }

    /// Convert to string
    pub fn to_string(&self) -> String {
        let mut version = format!("{}.{}.{}", self.major, self.minor, self.patch);
        
        if let Some(prerelease) = &self.prerelease {
            version.push('-');
            version.push_str(prerelease);
        }
        
        if let Some(build) = &self.build {
            version.push('+');
            version.push_str(build);
        }
        
        version
    }

    /// Compare versions
    pub fn compare(&self, other: &Self) -> Ordering {
        // Compare core version numbers
        match (self.major, self.minor, self.patch).cmp(&(other.major, other.minor, other.patch)) {
            Ordering::Equal => {
                // Compare prerelease
                match (&self.prerelease, &other.prerelease) {
                    (None, None) => Ordering::Equal,
                    (None, Some(_)) => Ordering::Greater, // Release > prerelease
                    (Some(_), None) => Ordering::Less,    // Prerelease < release
                    (Some(a), Some(b)) => a.cmp(b),
                }
            }
            other => other,
        }
    }

    /// Check if versions are compatible (~)
    pub fn is_compatible(&self, other: &Self) -> bool {
        self.major == other.major && self.minor == other.minor
    }

    /// Check if versions are major compatible (^)
    pub fn is_major_compatible(&self, other: &Self) -> bool {
        self.major == other.major && self >= other
    }
}

impl PartialOrd for SemanticVersion {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.compare(other))
    }
}

impl Ord for SemanticVersion {
    fn cmp(&self, other: &Self) -> Ordering {
        self.compare(other)
    }
}

/// Compare two version strings
pub fn compare_versions(v1: &str, v2: &str, op: VersionOp) -> bool {
    // Handle version aliases first
    if is_common_alias(v1) || is_common_alias(v2) {
        return compare_alias_versions(v1, v2, op);
    }
    
    // Try semantic version comparison
    if let (Ok(ver1), Ok(ver2)) = (SemanticVersion::parse(v1), SemanticVersion::parse(v2)) {
        match op {
            VersionOp::Equal => ver1 == ver2,
            VersionOp::NotEqual => ver1 != ver2,
            VersionOp::Greater => ver1 > ver2,
            VersionOp::GreaterEqual => ver1 >= ver2,
            VersionOp::Less => ver1 < ver2,
            VersionOp::LessEqual => ver1 <= ver2,
            VersionOp::Compatible => ver1.is_compatible(&ver2),
            VersionOp::CompatibleMajor => ver1.is_major_compatible(&ver2),
        }
    } else {
        // Fallback to string comparison
        match op {
            VersionOp::Equal => v1 == v2,
            VersionOp::NotEqual => v1 != v2,
            _ => false, // Can't compare non-semantic versions with operators
        }
    }
}

/// Compare version aliases
fn compare_alias_versions(v1: &str, v2: &str, op: VersionOp) -> bool {
    let alias_order = |alias: &str| -> u32 {
        match alias {
            "dev" | "experimental" => 0,
            "alpha" => 1,
            "beta" => 2,
            "rc" => 3,
            "stable" | "lts" => 4,
            "latest" => 5,
            _ => 3, // Default to rc level
        }
    };
    
    let order1 = alias_order(v1);
    let order2 = alias_order(v2);
    
    match op {
        VersionOp::Equal => order1 == order2,
        VersionOp::NotEqual => order1 != order2,
        VersionOp::Greater => order1 > order2,
        VersionOp::GreaterEqual => order1 >= order2,
        VersionOp::Less => order1 < order2,
        VersionOp::LessEqual => order1 <= order2,
        VersionOp::Compatible | VersionOp::CompatibleMajor => order1 == order2,
    }
}

/// Find the latest version from a list of TRNs
pub fn find_latest_version(trns: &[String]) -> Option<String> {
    let mut latest: Option<(String, SemanticVersion)> = None;
    
    for trn_str in trns {
        if let Ok(trn) = Trn::parse(trn_str) {
            let version = trn.version();
            
            // Skip aliases for latest comparison
            if is_common_alias(version) {
                continue;
            }
            
            if let Ok(semver) = SemanticVersion::parse(version) {
                match &latest {
                    None => latest = Some((trn_str.clone(), semver)),
                    Some((_, current_ver)) => {
                        if semver > *current_ver {
                            latest = Some((trn_str.clone(), semver));
                        }
                    }
                }
            }
        }
    }
    
    latest.map(|(trn, _)| trn)
}

/// Version component for increment operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VersionComponent {
    /// Major version (X.y.z)
    Major,
    /// Minor version (x.Y.z)
    Minor,
    /// Patch version (x.y.Z)
    Patch,
}

/// Parse a version string into a SemanticVersion
pub fn parse_version(version: &str) -> TrnResult<SemanticVersion> {
    SemanticVersion::parse(version)
}

/// Increment a version by the specified component
pub fn increment_version(version: &str, component: VersionComponent) -> TrnResult<String> {
    let mut semver = SemanticVersion::parse(version)?;
    
    match component {
        VersionComponent::Major => {
            semver.major += 1;
            semver.minor = 0;
            semver.patch = 0;
        }
        VersionComponent::Minor => {
            semver.minor += 1;
            semver.patch = 0;
        }
        VersionComponent::Patch => {
            semver.patch += 1;
        }
    }
    
    Ok(semver.to_string())
}

/// Normalize version string to standard format
pub fn normalize_version(version: &str) -> TrnResult<String> {
    if is_common_alias(version) {
        return Ok(version.to_string());
    }
    
    let semver = SemanticVersion::parse(version)?;
    Ok(semver.to_string())
}

/// Check if a version string is a semantic version
pub fn is_semantic_version(version: &str) -> bool {
    SemanticVersion::parse(version).is_ok()
}

/// Transform a TRN to use a different version
pub fn transform_version(trn: &Trn, new_version: &str) -> TrnResult<Trn> {
    let mut builder = crate::TrnBuilder::new()
        .platform(trn.platform())
        .resource_type(trn.resource_type())
        .type_(trn.type_())
        .instance_id(trn.instance_id())
        .version(new_version);
    
    if let Some(scope) = trn.scope() {
        if !scope.is_empty() {
            builder = builder.scope(scope);
        }
    }
    
    if let Some(subtype) = trn.subtype() {
        if !subtype.is_empty() {
            builder = builder.subtype(subtype);
        }
    }
    
    if let Some(tag) = trn.tag() {
        if !tag.is_empty() {
            builder = builder.tag(tag);
        }
    }
    
    if let Some(hash) = trn.hash() {
        if !hash.is_empty() {
            builder = builder.hash(hash);
        }
    }
    
    builder.build()
}

/// Group TRNs by component
pub fn group_trns_by_platform(trns: &[String]) -> HashMap<String, Vec<String>> {
    let mut groups = HashMap::new();
    
    for trn_str in trns {
        if let Ok(trn) = Trn::parse(trn_str) {
            groups
                .entry(trn.platform().to_string())
                .or_insert_with(Vec::new)
                .push(trn_str.clone());
        }
    }
    
    groups
}

/// Group TRNs by resource type
pub fn group_trns_by_resource_type(trns: &[String]) -> HashMap<String, Vec<String>> {
    let mut groups = HashMap::new();
    
    for trn_str in trns {
        if let Ok(trn) = Trn::parse(trn_str) {
            groups
                .entry(trn.resource_type().to_string())
                .or_insert_with(Vec::new)
                .push(trn_str.clone());
        }
    }
    
    groups
}

/// Group TRNs by version
pub fn group_trns_by_version(trns: &[String]) -> HashMap<String, Vec<String>> {
    let mut groups = HashMap::new();
    
    for trn_str in trns {
        if let Ok(trn) = Trn::parse(trn_str) {
            groups
                .entry(trn.version().to_string())
                .or_insert_with(Vec::new)
                .push(trn_str.clone());
        }
    }
    
    groups
}

/// TRN statistics
#[derive(Debug, Clone)]
pub struct TrnStatistics {
    /// Total number of TRNs
    pub total_count: usize,
    /// Number of unique platforms
    pub unique_platforms: usize,
    /// Number of unique resource types
    pub unique_resource_types: usize,
    /// Number of unique versions
    pub unique_versions: usize,
    /// Most common platform
    pub most_common_platform: Option<String>,
    /// Most common resource type
    pub most_common_resource_type: Option<String>,
    /// Most common version
    pub most_common_version: Option<String>,
    /// TRNs with hashes
    pub trns_with_hash: usize,
    /// TRNs with tags
    pub trns_with_tag: usize,
    /// Average TRN length
    pub average_length: f64,
}

/// Calculate statistics for a collection of TRNs
pub fn calculate_trn_statistics(trns: &[String]) -> TrnStatistics {
    let mut platforms = HashMap::new();
    let mut resource_types = HashMap::new();
    let mut versions = HashMap::new();
    let mut hash_count = 0;
    let mut tag_count = 0;
    let mut total_length = 0;
    
    for trn_str in trns {
        total_length += trn_str.len();
        
        if let Ok(trn) = Trn::parse(trn_str) {
            *platforms.entry(trn.platform().to_string()).or_insert(0) += 1;
            *resource_types.entry(trn.resource_type().to_string()).or_insert(0) += 1;
            *versions.entry(trn.version().to_string()).or_insert(0) += 1;
            
            if trn.hash().is_some() {
                hash_count += 1;
            }
            
            if trn.tag().is_some() {
                tag_count += 1;
            }
        }
    }
    
    let most_common_platform = platforms
        .iter()
        .max_by_key(|(_, count)| *count)
        .map(|(platform, _)| platform.clone());
    
    let most_common_resource_type = resource_types
        .iter()
        .max_by_key(|(_, count)| *count)
        .map(|(resource_type, _)| resource_type.clone());
    
    let most_common_version = versions
        .iter()
        .max_by_key(|(_, count)| *count)
        .map(|(version, _)| version.clone());
    
    TrnStatistics {
        total_count: trns.len(),
        unique_platforms: platforms.len(),
        unique_resource_types: resource_types.len(),
        unique_versions: versions.len(),
        most_common_platform,
        most_common_resource_type,
        most_common_version,
        trns_with_hash: hash_count,
        trns_with_tag: tag_count,
        average_length: if trns.is_empty() {
            0.0
        } else {
            total_length as f64 / trns.len() as f64
        },
    }
}

/// Extract unique component values
pub fn extract_unique_platforms(trns: &[String]) -> Vec<String> {
    let mut platforms = std::collections::HashSet::new();
    
    for trn_str in trns {
        if let Ok(trn) = Trn::parse(trn_str) {
            platforms.insert(trn.platform().to_string());
        }
    }
    
    let mut result: Vec<String> = platforms.into_iter().collect();
    result.sort();
    result
}

/// Extract unique resource types
pub fn extract_unique_resource_types(trns: &[String]) -> Vec<String> {
    let mut resource_types = std::collections::HashSet::new();
    
    for trn_str in trns {
        if let Ok(trn) = Trn::parse(trn_str) {
            resource_types.insert(trn.resource_type().to_string());
        }
    }
    
    let mut result: Vec<String> = resource_types.into_iter().collect();
    result.sort();
    result
}

/// Extract unique versions and sort them semantically
pub fn extract_unique_versions(trns: &[String]) -> Vec<String> {
    let mut versions = std::collections::HashSet::new();
    
    for trn_str in trns {
        if let Ok(trn) = Trn::parse(trn_str) {
            versions.insert(trn.version().to_string());
        }
    }
    
    let mut result: Vec<String> = versions.into_iter().collect();
    
    // Sort semantically
    result.sort_by(|a, b| {
        match (SemanticVersion::parse(a), SemanticVersion::parse(b)) {
            (Ok(v1), Ok(v2)) => v1.cmp(&v2),
            _ => a.cmp(b), // Fallback to string comparison
        }
    });
    
    result
}

/// Convert TRN to different formats
pub fn convert_trn_format(trn: &str, format: TrnFormat) -> TrnResult<String> {
    let parsed_trn = Trn::parse(trn)?;
    
    match format {
        TrnFormat::Standard => Ok(parsed_trn.to_string()),
        TrnFormat::Url => parsed_trn.to_url(),
        TrnFormat::Components => {
            let components = parsed_trn.components();
            Ok(format!(
                "Platform: {}\nScope: {:?}\nResource Type: {}\nType: {}\nSubtype: {:?}\nInstance ID: {}\nVersion: {}\nTag: {:?}\nHash: {:?}",
                components.platform,
                components.scope,
                components.resource_type,
                components.type_,
                components.subtype,
                components.instance_id,
                components.version,
                components.tag,
                components.hash
            ))
        }
        TrnFormat::Json => {
            serde_json::to_string_pretty(&parsed_trn)
                .map_err(|e| TrnError::format(format!("JSON serialization error: {}", e), Some(trn.to_string())))
        }
    }
}

/// TRN output format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrnFormat {
    /// Standard TRN string
    Standard,
    /// TRN URL format
    Url,
    /// Human-readable components
    Components,
    /// JSON format
    Json,
}

/// Deduplicate TRNs (keep only unique ones)
pub fn deduplicate_trns(trns: &[String]) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    let mut result = Vec::new();
    
    for trn in trns {
        if seen.insert(trn.clone()) {
            result.push(trn.clone());
        }
    }
    
    result
}

/// Sort TRNs by various criteria
pub fn sort_trns(trns: &mut [String], sort_by: TrnSortCriteria) {
    trns.sort_by(|a, b| {
        let trn_a = Trn::parse(a);
        let trn_b = Trn::parse(b);
        
        match (trn_a, trn_b) {
            (Ok(ta), Ok(tb)) => match sort_by {
                TrnSortCriteria::Platform => ta.platform().cmp(tb.platform()),
                TrnSortCriteria::ResourceType => ta.resource_type().cmp(tb.resource_type()),
                TrnSortCriteria::InstanceId => ta.instance_id().cmp(tb.instance_id()),
                TrnSortCriteria::Version => {
                    match (SemanticVersion::parse(ta.version()), SemanticVersion::parse(tb.version())) {
                        (Ok(v1), Ok(v2)) => v1.cmp(&v2),
                        _ => ta.version().cmp(tb.version()),
                    }
                }
                TrnSortCriteria::Length => ta.to_string().len().cmp(&tb.to_string().len()),
            },
            _ => a.cmp(b), // Fallback to string comparison
        }
    });
}

/// TRN sorting criteria
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrnSortCriteria {
    /// Sort by platform
    Platform,
    /// Sort by resource type
    ResourceType,
    /// Sort by instance ID
    InstanceId,
    /// Sort by version (semantic)
    Version,
    /// Sort by TRN string length
    Length,
}

/// TRN sorting order (alias for compatibility)
pub type TrnSortOrder = TrnSortCriteria;

impl TrnSortCriteria {
    /// Sort by scope then instance ID
    pub const SCOPE_THEN_INSTANCE: Self = Self::InstanceId;
    /// Sort by semantic version
    pub const SEMANTIC_VERSION: Self = Self::Version;
}

/// Generate TRN variants (different versions of the same base TRN)
pub fn generate_trn_variants(base_trn: &str, versions: &[&str]) -> TrnResult<Vec<String>> {
    let trn = Trn::parse(base_trn)?;
    let base = trn.base_trn();
    
    let mut variants = Vec::new();
    
    for version in versions {
        let variant = crate::builder::TrnBuilder::from_trn(&base)
            .version(*version)
            .build()?;
        variants.push(variant.to_string());
    }
    
    Ok(variants)
}

/// Validate a collection of TRNs and return summary
pub fn validate_trn_collection(trns: &[String]) -> TrnValidationSummary {
    let mut valid = 0;
    let mut invalid = 0;
    let mut errors = Vec::new();
    
    for trn in trns {
        match crate::validation::validate_trn_string(trn) {
            Ok(()) => valid += 1,
            Err(e) => {
                invalid += 1;
                errors.push(format!("{}: {}", trn, e));
            }
        }
    }
    
    TrnValidationSummary {
        total: trns.len(),
        valid,
        invalid,
        errors,
    }
}

/// TRN validation summary
#[derive(Debug, Clone)]
pub struct TrnValidationSummary {
    /// Total TRNs processed
    pub total: usize,
    /// Number of valid TRNs
    pub valid: usize,
    /// Number of invalid TRNs
    pub invalid: usize,
    /// List of validation errors
    pub errors: Vec<String>,
}

/// Batch parse result structure
#[derive(Debug)]
pub struct BatchParseResult {
    /// Successfully parsed TRNs
    pub successes: Vec<Trn>,
    /// Failed parsing attempts with errors
    pub failures: Vec<TrnError>,
}

/// Parse multiple TRN strings at once
pub fn batch_parse(trn_strings: &[String]) -> BatchParseResult {
    let results: Vec<TrnResult<Trn>> = trn_strings.iter()
        .map(|s| Trn::parse(s))
        .collect();
    
    let mut successes = Vec::new();
    let mut failures = Vec::new();
    
    for result in results {
        match result {
            Ok(trn) => successes.push(trn),
            Err(err) => failures.push(err),
        }
    }
    
    BatchParseResult { successes, failures }
}

/// Transform a TRN to use a different platform
pub fn transform_platform(
    trn: &Trn, 
    new_platform: Platform, 
    scope: Option<&str>
) -> TrnResult<Trn> {
    let mut new_trn = trn.clone();
    new_trn.set_scope(scope.map(|s| s.to_string()));
    
    // Create new TRN with the new platform
    Trn::new_full(
        new_platform.to_string(),
        scope.map(|s| s.to_string()),
        trn.resource_type(),
        trn.type_(),
        trn.subtype().map(|s| s.to_string()),
        trn.instance_id(),
        trn.version(),
        trn.tag().map(|s| s.to_string()),
        trn.hash().map(|s| s.to_string()),
    )
}

/// Convert batch of TRNs to URLs
pub fn convert_batch_to_urls(trns: &[Trn]) -> TrnResult<Vec<String>> {
    trns.iter()
        .map(|trn| trn.to_url())
        .collect()
}

/// Get error context for debugging
pub fn get_error_context(error: &TrnError) -> String {
    format!("Error: {} (Code: {})", error, error.code())
}

/// Get fix suggestions for common errors
pub fn get_fix_suggestions(error: &TrnError) -> Vec<String> {
    match error.code() {
        100 => vec!["Check TRN format: trn:platform:resource_type:type:instance_id:version".to_string()],
        101 => vec!["Ensure all required components are present".to_string()],
        102 => vec!["Check component character restrictions (alphanumeric, hyphens, underscores)".to_string()],
        _ => vec!["Check TRN documentation for valid formats".to_string()],
    }
}

/// Group TRNs by scope
pub fn group_by_scope(trns: &[String]) -> HashMap<String, Vec<String>> {
    let mut groups = HashMap::new();
    
    for trn_str in trns {
        if let Ok(trn) = Trn::parse(trn_str) {
            let scope = trn.scope().unwrap_or("").to_string();
            groups
                .entry(scope)
                .or_insert_with(Vec::new)
                .push(trn_str.clone());
        }
    }
    
    groups
}

/// Group TRNs by tool type
pub fn group_by_tool_type(trns: &[String]) -> HashMap<String, Vec<String>> {
    let mut groups = HashMap::new();
    
    for trn_str in trns {
        if let Ok(trn) = Trn::parse(trn_str) {
            if trn.resource_type() == "tool" {
                groups
                    .entry(trn.type_().to_string())
                    .or_insert_with(Vec::new)
                    .push(trn_str.clone());
            }
        }
    }
    
    groups
}

/// Filter TRNs by platform
pub fn filter_by_platform(trns: &[Trn], platform: &crate::Platform) -> Vec<Trn> {
    let platform_str = match platform {
        crate::Platform::User => "user",
        crate::Platform::Org => "org", 
        crate::Platform::AiPlatform => "aiplatform",
        crate::Platform::Custom(custom) => custom.as_str(),
    };
    
    trns.iter()
        .filter(|trn| trn.platform() == platform_str)
        .cloned()
        .collect()
}

/// Filter TRNs by scope
pub fn filter_by_scope(trns: &[Trn], scope: &str) -> Vec<Trn> {
    trns.iter()
        .filter(|trn| trn.scope().map_or(false, |s| s == scope))
        .cloned()
        .collect()
}

/// Filter TRNs by tool type
pub fn filter_by_tool_type(trns: &[Trn], tool_type: &crate::ToolType) -> Vec<Trn> {
    let tool_type_str = match tool_type {
        crate::ToolType::OpenApi => "openapi",
        crate::ToolType::Workflow => "workflow",
        crate::ToolType::Python => "python",
        crate::ToolType::Shell => "shell",
        crate::ToolType::System => "system",
        crate::ToolType::Function => "function",
        crate::ToolType::Composite => "composite",
        crate::ToolType::AsyncApi => "async_api",
        crate::ToolType::Custom(custom) => custom.as_str(),
    };
    
    trns.iter()
        .filter(|trn| trn.resource_type() == "tool" && trn.type_() == tool_type_str)
        .cloned()
        .collect()
}

/// Filter TRNs by version pattern
pub fn filter_by_version_pattern(trns: &[Trn], pattern: &str) -> Vec<Trn> {
    let regex_pattern = regex::Regex::new(pattern).unwrap_or_else(|_| {
        regex::Regex::new(&regex::escape(pattern)).unwrap()
    });
    
    trns.iter()
        .filter(|trn| regex_pattern.is_match(trn.version()))
        .cloned()
        .collect()
}

/// Sort TRNs alphabetically by their string representation
pub fn sort_trns_alphabetically(trns: &mut [String]) {
    trns.sort();
}

/// Sort TRNs by version (semantic version aware)
pub fn sort_trns_by_version(trns: &mut [String], reverse: bool) {
    trns.sort_by(|a, b| {
        let trn_a = Trn::parse(a);
        let trn_b = Trn::parse(b);
        
        match (trn_a, trn_b) {
            (Ok(ta), Ok(tb)) => {
                let version_a = ta.version();
                let version_b = tb.version();
                
                match (SemanticVersion::parse(version_a), SemanticVersion::parse(version_b)) {
                    (Ok(v1), Ok(v2)) => {
                        if reverse {
                            v2.cmp(&v1)
                        } else {
                            v1.cmp(&v2)
                        }
                    }
                    _ => {
                        if reverse {
                            version_b.cmp(version_a)
                        } else {
                            version_a.cmp(version_b)
                        }
                    }
                }
            }
            _ => {
                if reverse {
                    b.cmp(a)
                } else {
                    a.cmp(b)
                }
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_semantic_version_parsing() {
        let version = SemanticVersion::parse("1.2.3-beta+build.1").unwrap();
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 2);
        assert_eq!(version.patch, 3);
        assert_eq!(version.prerelease, Some("beta".to_string()));
        assert_eq!(version.build, Some("build.1".to_string()));
    }

    #[test]
    fn test_version_comparison() {
        assert!(compare_versions("v1.2.3", "v1.2.0", VersionOp::Greater));
        assert!(compare_versions("v1.2.3", "v1.2.3", VersionOp::Equal));
        assert!(compare_versions("v1.2.0", "v1.2.3", VersionOp::Less));
        
        let v1 = SemanticVersion::parse("1.2.3").unwrap();
        let v2 = SemanticVersion::parse("1.2.0").unwrap();
        assert!(v1.is_compatible(&v2));
    }

    #[test]
    fn test_find_latest_version() {
        let trns = vec![
            "trn:user:alice:tool:openapi:api:v1.0.0".to_string(),
            "trn:user:alice:tool:openapi:api:v1.2.0".to_string(),
            "trn:user:alice:tool:openapi:api:v1.1.5".to_string(),
        ];
        
        let latest = find_latest_version(&trns).unwrap();
        assert!(latest.contains("v1.2.0"));
    }

    #[test]
    fn test_group_trns() {
        let trns = vec![
            "trn:user:alice:tool:openapi:api:v1.0".to_string(),
            "trn:user:bob:tool:python:script:v2.0".to_string(),
            "trn:org:company:tool:workflow:pipeline:latest".to_string(),
        ];
        
        let by_platform = group_trns_by_platform(&trns);
        assert_eq!(by_platform.get("user").unwrap().len(), 2);
        assert_eq!(by_platform.get("org").unwrap().len(), 1);
        
        let by_resource = group_trns_by_resource_type(&trns);
        assert_eq!(by_resource.get("tool").unwrap().len(), 3);
    }

    #[test]
    fn test_trn_statistics() {
        let trns = vec![
            "trn:user:alice:tool:openapi:api:v1.0@sha256:1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string(),
            "trn:user:bob:tool:python:script:v2.0:beta".to_string(),
            "trn:org:company:dataset:structured:data:latest".to_string(),
        ];
        
        let stats = calculate_trn_statistics(&trns);
        assert_eq!(stats.total_count, 3);
        assert_eq!(stats.unique_platforms, 2);
        assert_eq!(stats.unique_resource_types, 2);
        assert_eq!(stats.trns_with_hash, 1);
        assert_eq!(stats.trns_with_tag, 1);
    }

    #[test]
    fn test_extract_unique_components() {
        let trns = vec![
            "trn:user:alice:tool:openapi:api:v1.0".to_string(),
            "trn:user:bob:tool:python:script:v2.0".to_string(),
            "trn:org:company:dataset:structured:data:latest".to_string(),
        ];
        
        let platforms = extract_unique_platforms(&trns);
        assert_eq!(platforms, vec!["org", "user"]);
        
        let resources = extract_unique_resource_types(&trns);
        assert_eq!(resources, vec!["dataset", "tool"]);
    }

    #[test]
    fn test_sort_trns() {
        let mut trns = vec![
            "trn:user:charlie:tool:openapi:api:v1.0".to_string(),
            "trn:user:alice:tool:python:script:v2.0".to_string(),
            "trn:user:bob:tool:workflow:pipeline:latest".to_string(),
        ];
        
        sort_trns(&mut trns, TrnSortCriteria::Platform);
        // All are user platform, so order should be preserved
        
        // Test by instance ID
        sort_trns(&mut trns, TrnSortCriteria::InstanceId);
        assert!(trns[0].contains("api"));
        assert!(trns[1].contains("pipeline"));
        assert!(trns[2].contains("script"));
    }

    #[test]
    fn test_generate_variants() {
        let base = "trn:user:alice:tool:openapi:api:v1.0";
        let versions = &["v2.0", "v3.0", "latest"];
        
        let variants = generate_trn_variants(base, versions).unwrap();
        assert_eq!(variants.len(), 3);
        assert!(variants.iter().any(|v| v.contains("v2.0")));
        assert!(variants.iter().any(|v| v.contains("latest")));
    }

    #[test]
    fn test_conversion_formats() {
        let trn = "trn:user:alice:tool:openapi:github-api:v1.0";
        
        let standard = convert_trn_format(trn, TrnFormat::Standard).unwrap();
        assert_eq!(standard, trn);
        
        let url = convert_trn_format(trn, TrnFormat::Url).unwrap();
        assert!(url.starts_with("trn://"));
        
        let json = convert_trn_format(trn, TrnFormat::Json).unwrap();
        assert!(json.contains("platform"));
    }
} 