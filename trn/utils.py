"""
TRN Utility Functions

Provides utility functions and helper classes for working with TRN objects.
Includes pattern matching, version comparison, normalization, and more.
"""

import re
import fnmatch
from typing import List, Optional, Union, Tuple, Dict, Any
from functools import lru_cache

from .exceptions import TRNError, TRNValidationError
from .constants import COMMON_ALIASES, VERSION_OPERATORS


def normalize_trn(trn_input: Union[str, 'TRN']) -> str:
    """
    Normalize a TRN string or object.
    
    Args:
        trn_input: TRN string or TRN object
        
    Returns:
        str: Normalized TRN string
        
    Examples:
        >>> normalize_trn("TRN:USER:Alice:TOOL:OpenAPI:GitHub-API:V1.0")
        "trn:user:alice:tool:openapi:github-api:v1.0"
    """
    if hasattr(trn_input, '__str__'):
        # It's a TRN object
        return str(trn_input)
    
    # It's a string
    from .parser import TRNParser
    return TRNParser.normalize_string(trn_input)


def generate_trn(
    platform: str,
    resource_type: str,
    type: str,
    instance_id: str,
    version: str,
    scope: Optional[str] = None,
    subtype: Optional[str] = None,
    tag: Optional[str] = None,
    hash: Optional[str] = None
) -> str:
    """
    Generate a TRN string from components.
    
    Args:
        platform: Platform identifier
        resource_type: Resource type
        type: Specific type
        instance_id: Instance identifier
        version: Version identifier
        scope: Optional scope
        subtype: Optional subtype
        tag: Optional tag
        hash: Optional hash
        
    Returns:
        str: Generated TRN string
        
    Examples:
        >>> generate_trn("user", "tool", "openapi", "github-api", "v1.0", scope="alice")
        "trn:user:alice:tool:openapi:github-api:v1.0"
    """
    from .core import TRN
    
    trn = TRN(
        platform=platform,
        resource_type=resource_type,
        type=type,
        instance_id=instance_id,
        version=version,
        scope=scope,
        subtype=subtype,
        tag=tag,
        hash=hash
    )
    
    return str(trn)


def extract_base_trn(trn_input: Union[str, 'TRN']) -> str:
    """
    Extract base TRN (without version, tag, and hash).
    
    Args:
        trn_input: TRN string or object
        
    Returns:
        str: Base TRN string
        
    Examples:
        >>> extract_base_trn("trn:user:alice:tool:openapi:github-api:v1.0:beta@sha256:abc123")
        "trn:user:alice:tool:openapi:github-api"
    """
    if hasattr(trn_input, 'get_base_trn'):
        # It's a TRN object
        return str(trn_input.get_base_trn())
    
    # It's a string
    from .parser import TRNParser
    return TRNParser.extract_base_trn(trn_input)


def is_valid_trn(trn_string: str, strict: bool = True) -> bool:
    """
    Check if a string is a valid TRN.
    
    Args:
        trn_string: String to validate
        strict: Whether to use strict validation
        
    Returns:
        bool: True if valid
        
    Examples:
        >>> is_valid_trn("trn:user:alice:tool:openapi:github-api:v1.0")
        True
        >>> is_valid_trn("invalid-trn")
        False
    """
    try:
        from .validator import TRNValidator
        TRNValidator.validate(trn_string, strict=strict)
        return True
    except:
        return False


@lru_cache(maxsize=200)
def match_trn_pattern(trn_string: str, pattern: str) -> bool:
    """
    Check if TRN matches a pattern (supports wildcards).
    
    Args:
        trn_string: TRN string to match
        pattern: Pattern with optional wildcards (*, ?)
        
    Returns:
        bool: True if matches
        
    Examples:
        >>> match_trn_pattern("trn:user:alice:tool:openapi:github-api:v1.0", "trn:user:*:tool:*")
        True
        >>> match_trn_pattern("trn:user:alice:tool:openapi:github-api:v1.0", "trn:org:*")
        False
    """
    # Use fnmatch for basic wildcard matching
    return fnmatch.fnmatch(trn_string, pattern)


def compare_trn_versions(version1: str, version2: str, operator: str = "==") -> bool:
    """
    Compare two TRN versions.
    
    Args:
        version1: First version
        version2: Second version
        operator: Comparison operator (==, !=, >, >=, <, <=, ~, ^)
        
    Returns:
        bool: True if comparison is satisfied
        
    Examples:
        >>> compare_trn_versions("v1.2.0", "v1.1.0", ">")
        True
        >>> compare_trn_versions("v1.2.0", "v1.2.*", "~")
        True
    """
    if operator not in VERSION_OPERATORS:
        raise ValueError(f"Invalid operator: {operator}. Supported: {VERSION_OPERATORS}")
    
    # Handle alias versions
    if version1 in COMMON_ALIASES or version2 in COMMON_ALIASES:
        return _compare_alias_versions(version1, version2, operator)
    
    # Parse semantic versions
    v1_parts = _parse_version(version1)
    v2_parts = _parse_version(version2)
    
    if operator == "==":
        return v1_parts == v2_parts
    elif operator == "!=":
        return v1_parts != v2_parts
    elif operator == ">":
        return v1_parts > v2_parts
    elif operator == ">=":
        return v1_parts >= v2_parts
    elif operator == "<":
        return v1_parts < v2_parts
    elif operator == "<=":
        return v1_parts <= v2_parts
    elif operator == "~":
        # Compatible version (same major.minor)
        return v1_parts[:2] == v2_parts[:2]
    elif operator == "^":
        # Compatible within major version
        return v1_parts[0] == v2_parts[0] and v1_parts >= v2_parts
    
    return False


def _parse_version(version: str) -> Tuple[int, ...]:
    """Parse version string into comparable tuple."""
    # Remove 'v' prefix if present
    if version.startswith('v'):
        version = version[1:]
    
    # Split by dots and convert to integers
    parts = []
    for part in version.split('.'):
        # Extract numeric part (ignore suffixes like '-beta')
        numeric_part = re.match(r'(\d+)', part)
        if numeric_part:
            parts.append(int(numeric_part.group(1)))
        else:
            parts.append(0)
    
    # Ensure at least 3 parts (major.minor.patch)
    while len(parts) < 3:
        parts.append(0)
    
    return tuple(parts)


def _compare_alias_versions(version1: str, version2: str, operator: str) -> bool:
    """Compare versions when one or both are aliases."""
    # Simplified alias comparison
    alias_order = {
        "dev": 0,
        "alpha": 1,
        "beta": 2,
        "rc": 3,
        "stable": 4,
        "latest": 5,
        "lts": 4  # Same as stable
    }
    
    if version1 in alias_order and version2 in alias_order:
        v1_order = alias_order[version1]
        v2_order = alias_order[version2]
        
        if operator == "==":
            return v1_order == v2_order
        elif operator == "!=":
            return v1_order != v2_order
        elif operator == ">":
            return v1_order > v2_order
        elif operator == ">=":
            return v1_order >= v2_order
        elif operator == "<":
            return v1_order < v2_order
        elif operator == "<=":
            return v1_order <= v2_order
    
    # If only one is an alias, assume they're different unless exact match
    return operator == "!=" if operator in ["==", "!="] else False


def find_matching_trns(trn_list: List[str], pattern: str) -> List[str]:
    """
    Find all TRNs in a list that match a pattern.
    
    Args:
        trn_list: List of TRN strings
        pattern: Pattern to match
        
    Returns:
        List[str]: Matching TRN strings
        
    Examples:
        >>> trns = ["trn:user:alice:tool:openapi:github-api:v1.0", 
        ...         "trn:user:bob:tool:openapi:slack-api:v2.0"]
        >>> find_matching_trns(trns, "trn:user:*:tool:openapi:*")
        ["trn:user:alice:tool:openapi:github-api:v1.0", "trn:user:bob:tool:openapi:slack-api:v2.0"]
    """
    return [trn for trn in trn_list if match_trn_pattern(trn, pattern)]


def group_trns_by_base(trn_list: List[str]) -> Dict[str, List[str]]:
    """
    Group TRNs by their base (without version/tag/hash).
    
    Args:
        trn_list: List of TRN strings
        
    Returns:
        Dict[str, List[str]]: Dictionary mapping base TRN to list of full TRNs
        
    Examples:
        >>> trns = ["trn:user:alice:tool:openapi:github-api:v1.0",
        ...         "trn:user:alice:tool:openapi:github-api:v2.0"]
        >>> group_trns_by_base(trns)
        {"trn:user:alice:tool:openapi:github-api": ["...:v1.0", "...:v2.0"]}
    """
    groups = {}
    
    for trn in trn_list:
        try:
            base = extract_base_trn(trn)
            if base not in groups:
                groups[base] = []
            groups[base].append(trn)
        except:
            # Skip invalid TRNs
            continue
    
    return groups


def get_latest_version_trn(trn_list: List[str]) -> Optional[str]:
    """
    Get the TRN with the latest version from a list.
    
    Args:
        trn_list: List of TRN strings (should be same base)
        
    Returns:
        Optional[str]: TRN with latest version, or None if list is empty
        
    Examples:
        >>> trns = ["trn:user:alice:tool:openapi:github-api:v1.0",
        ...         "trn:user:alice:tool:openapi:github-api:v2.1"]
        >>> get_latest_version_trn(trns)
        "trn:user:alice:tool:openapi:github-api:v2.1"
    """
    if not trn_list:
        return None
    
    # Parse versions and find latest
    latest_trn = None
    latest_version = None
    
    for trn in trn_list:
        try:
            from .core import TRN
            parsed_trn = TRN.parse(trn)
            version = parsed_trn.version
            
            if latest_version is None or compare_trn_versions(version, latest_version, ">"):
                latest_version = version
                latest_trn = trn
        except:
            continue
    
    return latest_trn


def suggest_similar_trns(trn_string: str, available_trns: List[str], max_suggestions: int = 5) -> List[str]:
    """
    Suggest similar TRNs based on edit distance and component similarity.
    
    Args:
        trn_string: Invalid or target TRN string
        available_trns: List of available TRN strings
        max_suggestions: Maximum number of suggestions
        
    Returns:
        List[str]: List of suggested TRN strings
    """
    suggestions = []
    
    for available_trn in available_trns:
        similarity = _calculate_trn_similarity(trn_string, available_trn)
        suggestions.append((similarity, available_trn))
    
    # Sort by similarity (descending) and return top suggestions
    suggestions.sort(key=lambda x: x[0], reverse=True)
    return [trn for _, trn in suggestions[:max_suggestions]]


def _calculate_trn_similarity(trn1: str, trn2: str) -> float:
    """Calculate similarity score between two TRNs."""
    # Simple edit distance-based similarity
    import difflib
    return difflib.SequenceMatcher(None, trn1.lower(), trn2.lower()).ratio()


class TRNBuilder:
    """
    Builder pattern for constructing TRN objects.
    
    Provides a fluent interface for building TRN objects step by step.
    
    Examples:
        >>> trn = (TRNBuilder()
        ...        .platform("user")
        ...        .scope("alice")
        ...        .resource_type("tool")
        ...        .type("openapi")
        ...        .instance_id("github-api")
        ...        .version("v1.0")
        ...        .build())
    """
    
    def __init__(self):
        self._platform = None
        self._scope = None
        self._resource_type = None
        self._type = None
        self._subtype = None
        self._instance_id = None
        self._version = None
        self._tag = None
        self._hash = None
    
    def platform(self, platform: str) -> 'TRNBuilder':
        """Set platform."""
        self._platform = platform
        return self
    
    def scope(self, scope: str) -> 'TRNBuilder':
        """Set scope."""
        self._scope = scope
        return self
    
    def resource_type(self, resource_type: str) -> 'TRNBuilder':
        """Set resource type."""
        self._resource_type = resource_type
        return self
    
    def type(self, type: str) -> 'TRNBuilder':
        """Set type."""
        self._type = type
        return self
    
    def subtype(self, subtype: str) -> 'TRNBuilder':
        """Set subtype."""
        self._subtype = subtype
        return self
    
    def instance_id(self, instance_id: str) -> 'TRNBuilder':
        """Set instance ID."""
        self._instance_id = instance_id
        return self
    
    def version(self, version: str) -> 'TRNBuilder':
        """Set version."""
        self._version = version
        return self
    
    def tag(self, tag: str) -> 'TRNBuilder':
        """Set tag."""
        self._tag = tag
        return self
    
    def hash(self, hash: str) -> 'TRNBuilder':
        """Set hash."""
        self._hash = hash
        return self
    
    def build(self) -> 'TRN':
        """Build the TRN object."""
        from .core import TRN
        
        return TRN(
            platform=self._platform,
            scope=self._scope,
            resource_type=self._resource_type,
            type=self._type,
            subtype=self._subtype,
            instance_id=self._instance_id,
            version=self._version,
            tag=self._tag,
            hash=self._hash
        )
    
    def build_string(self) -> str:
        """Build TRN string directly."""
        return str(self.build())


class TRNMatcher:
    """
    Advanced TRN pattern matcher with support for complex expressions.
    
    Supports various matching patterns including wildcards, ranges, and logical operators.
    
    Examples:
        >>> matcher = TRNMatcher("trn:user:*:tool:openapi:*:v1.*")
        >>> matcher.matches("trn:user:alice:tool:openapi:github-api:v1.2")
        True
    """
    
    def __init__(self, pattern: str):
        self.pattern = pattern
        self._compiled_pattern = self._compile_pattern(pattern)
    
    def matches(self, trn_string: str) -> bool:
        """Check if TRN matches the pattern."""
        return self._compiled_pattern(trn_string)
    
    def _compile_pattern(self, pattern: str):
        """Compile pattern into a matching function."""
        # Convert TRN pattern to regex-like matching
        if "*" in pattern or "?" in pattern:
            return lambda trn: fnmatch.fnmatch(trn, pattern)
        else:
            return lambda trn: trn == pattern


def batch_validate_trns(trn_list: List[str], strict: bool = True) -> Dict[str, Any]:
    """
    Validate a batch of TRN strings and return results.
    
    Args:
        trn_list: List of TRN strings to validate
        strict: Whether to use strict validation
        
    Returns:
        Dict[str, Any]: Validation results with statistics
        
    Examples:
        >>> results = batch_validate_trns(["trn:user:alice:tool:openapi:github-api:v1.0", "invalid"])
        >>> print(results["valid_count"])  # 1
        >>> print(results["invalid_count"])  # 1
    """
    from .validator import TRNValidator
    
    valid = []
    invalid = []
    errors = []
    
    for trn in trn_list:
        try:
            TRNValidator.validate(trn, strict=strict)
            valid.append(trn)
        except Exception as e:
            invalid.append(trn)
            errors.append(str(e))
    
    return {
        "total_count": len(trn_list),
        "valid_count": len(valid),
        "invalid_count": len(invalid),
        "valid_trns": valid,
        "invalid_trns": invalid,
        "errors": errors,
        "success_rate": len(valid) / len(trn_list) if trn_list else 0.0
    } 