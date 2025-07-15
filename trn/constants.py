"""
TRN Constants and Configuration

Defines all constants, patterns, and configuration values used throughout the TRN library.
"""

import re
from typing import Set, Dict, List

# TRN Format Constants
TRN_PREFIX = "trn"
TRN_SEPARATOR = ":"
TRN_HASH_SEPARATOR = "@"
TRN_MAX_LENGTH = 256
TRN_MIN_LENGTH = 10

# Component Length Limits
PLATFORM_MAX_LENGTH = 32
SCOPE_MAX_LENGTH = 32
RESOURCE_TYPE_MAX_LENGTH = 16
TYPE_MAX_LENGTH = 32
SUBTYPE_MAX_LENGTH = 32
INSTANCE_ID_MAX_LENGTH = 64
VERSION_MAX_LENGTH = 32
TAG_MAX_LENGTH = 16
HASH_MAX_LENGTH = 71

# Character Set Patterns (RFC3986 compatible)
PLATFORM_PATTERN = r"[a-z][a-z0-9-]{1,31}"
SCOPE_PATTERN = r"[a-z0-9][a-z0-9-]{0,31}"
RESOURCE_TYPE_PATTERN = r"[a-z][a-z0-9-]{1,15}"
TYPE_PATTERN = r"[a-z][a-z0-9-]{1,31}"
SUBTYPE_PATTERN = r"[a-z][a-z0-9-]{1,31}"
INSTANCE_ID_PATTERN = r"[a-z0-9][a-z0-9-]{0,63}"
VERSION_PATTERN = r"[a-z0-9][a-z0-9.-]{0,31}"
TAG_PATTERN = r"[a-z0-9][a-z0-9-]{0,15}"
HASH_PATTERN = r"[a-z0-9:]{8,71}"

# Complete TRN Regex Pattern
TRN_REGEX_PATTERN = (
    r"^trn:"
    r"(" + PLATFORM_PATTERN + r")"
    r"(?::(" + SCOPE_PATTERN + r"))?"
    r":(" + RESOURCE_TYPE_PATTERN + r")"
    r":(" + TYPE_PATTERN + r")"
    r"(?::(" + SUBTYPE_PATTERN + r"))?"
    r":(" + INSTANCE_ID_PATTERN + r")"
    r":(" + VERSION_PATTERN + r")"
    r"(?::(" + TAG_PATTERN + r"))?"
    r"(?:@(" + HASH_PATTERN + r"))?"
    r"$"
)

# Compiled regex for performance
TRN_REGEX = re.compile(TRN_REGEX_PATTERN)

# URL Pattern for TRN URLs
TRN_URL_PATTERN = (
    r"^trn://"
    r"(" + PLATFORM_PATTERN + r")"
    r"(?:/(" + SCOPE_PATTERN + r"))?"
    r"/(" + RESOURCE_TYPE_PATTERN + r")"
    r"/(" + TYPE_PATTERN + r")"
    r"(?:/(" + SUBTYPE_PATTERN + r"))?"
    r"/(" + INSTANCE_ID_PATTERN + r")"
    r"/(" + VERSION_PATTERN + r")"
    r"(?:/(" + TAG_PATTERN + r"))?"
    r"(?:\?hash=(" + HASH_PATTERN + r"))?"
    r"$"
)

TRN_URL_REGEX = re.compile(TRN_URL_PATTERN)

# Supported Platforms
SUPPORTED_PLATFORMS: Set[str] = {
    "aiplatform",  # System platform
    "user",        # User platform
    "org",         # Organization platform
}

# Supported Resource Types
SUPPORTED_RESOURCE_TYPES: Set[str] = {
    "tool",        # Executable tools
    "dataset",     # Data resources
    "pipeline",    # Workflow templates
    "model",       # AI model resources
}

# Supported Tool Types
SUPPORTED_TOOL_TYPES: Set[str] = {
    "openapi",     # RESTful API tools
    "workflow",    # Business process tools
    "python",      # Python execution tools
    "shell",       # Shell command tools
    "system",      # System operation tools
    "async_api",   # Async/Event-driven API tools
}

# Supported Tool Subtypes
SUPPORTED_TOOL_SUBTYPES: Set[str] = {
    "async",       # Asynchronous execution
    "streaming",   # Streaming data processing
    "batch",       # Batch processing
    "sync",        # Synchronous execution
    "realtime",    # Real-time processing
}

# Reserved Words (cannot be used in any component)
RESERVED_WORDS: Set[str] = {
    "__internal__",
    "__system__",
    "__admin__",
    "__test__",
    "system",
    "internal",
    "admin",
    "root",
    "super",
    "null",
    "undefined",
    "reserved",
}

# Common Version Aliases
COMMON_ALIASES: Set[str] = {
    "latest",
    "stable", 
    "beta",
    "alpha",
    "dev",
    "experimental",
    "lts",  # Long Term Support
    "rc",   # Release Candidate
}

# Hash Algorithm Mapping
HASH_ALGORITHMS: Dict[str, int] = {
    "md5": 32,       # MD5 hex length
    "sha1": 40,      # SHA1 hex length
    "sha256": 64,    # SHA256 hex length
    "sha512": 128,   # SHA512 hex length
    "crc32": 8,      # CRC32 hex length
}

# Supported Hash Formats
HASH_FORMAT_PATTERN = r"^(md5|sha1|sha256|sha512|crc32):[a-f0-9]+$"
HASH_FORMAT_REGEX = re.compile(HASH_FORMAT_PATTERN)

# URL Encoding Map for TRN components
URL_ENCODING_MAP: Dict[str, str] = {
    ":": "%3A",
    "/": "%2F",
    ".": "%2E",
    "@": "%40",
    "#": "%23",
    "?": "%3F",
    "&": "%26",
    "=": "%3D",
    "+": "%2B",
    " ": "%20",
}

# Default Configuration
DEFAULT_CONFIG: Dict[str, any] = {
    "validate_on_create": True,
    "strict_validation": True,
    "allow_uppercase": False,
    "normalize_on_parse": True,
    "cache_validation_results": True,
    "max_cache_size": 1000,
    "enable_deprecation_warnings": True,
    "hash_verification": True,
    "alias_resolution": True,
    "permission_check": False,  # Disabled by default
}

# Component Display Names (for error messages)
COMPONENT_NAMES: Dict[str, str] = {
    "platform": "Platform",
    "scope": "Scope", 
    "resource_type": "Resource Type",
    "type": "Type",
    "subtype": "Subtype",
    "instance_id": "Instance ID",
    "version": "Version",
    "tag": "Tag",
    "hash": "Hash",
}

# TRN Component Groups for wildcard matching
WILDCARD_GROUPS: Dict[str, List[str]] = {
    "platform_level": ["platform"],
    "tenant_level": ["platform", "scope"],
    "resource_level": ["platform", "scope", "resource_type"],
    "type_level": ["platform", "scope", "resource_type", "type"],
    "subtype_level": ["platform", "scope", "resource_type", "type", "subtype"],
    "instance_level": ["platform", "scope", "resource_type", "type", "subtype", "instance_id"],
    "version_level": ["platform", "scope", "resource_type", "type", "subtype", "instance_id", "version"],
    "tag_level": ["platform", "scope", "resource_type", "type", "subtype", "instance_id", "version", "tag"],
}

# Permission Actions
PERMISSION_ACTIONS: Set[str] = {
    "READ",
    "WRITE", 
    "EXECUTE",
    "DELETE",
    "ADMIN",
    "ALL",
}

# Version Comparison Operators
VERSION_OPERATORS: Set[str] = {
    "==",  # Equal
    "!=",  # Not equal
    ">",   # Greater than
    ">=",  # Greater than or equal
    "<",   # Less than
    "<=",  # Less than or equal
    "~",   # Compatible version
    "^",   # Compatible within major version
}

# HTTP Status Code Mapping for TRN Errors
HTTP_STATUS_MAP: Dict[int, int] = {
    -32000: 400,  # Bad Request - Format Error
    -32001: 400,  # Bad Request - Validation Error
    -32002: 400,  # Bad Request - Length/Character Error
    -32003: 400,  # Bad Request - Hash/Alias Error
    -32020: 403,  # Forbidden - Permission Error
    -32030: 404,  # Not Found - Resource Not Found
    -32031: 409,  # Conflict - Resource Conflict
}

# Default TTL values (in seconds)
TTL_VALUES: Dict[str, int] = {
    "validation_cache": 300,      # 5 minutes
    "alias_resolution": 600,      # 10 minutes
    "permission_cache": 1800,     # 30 minutes
    "hash_verification": 3600,    # 1 hour
} 