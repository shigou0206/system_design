"""
TRN Validator

Provides comprehensive validation for TRN format and components.
Includes regex validation, length checks, character set validation, and semantic validation.
"""

import re
from typing import Dict, List, Optional, Set, Tuple
from functools import lru_cache

from .exceptions import (
    TRNFormatError, TRNValidationError, TRNComponentError,
    TRNLengthError, TRNCharacterError, TRNReservedWordError
)
from .constants import (
    TRN_REGEX, TRN_MAX_LENGTH, TRN_MIN_LENGTH,
    PLATFORM_MAX_LENGTH, SCOPE_MAX_LENGTH, RESOURCE_TYPE_MAX_LENGTH,
    TYPE_MAX_LENGTH, SUBTYPE_MAX_LENGTH, INSTANCE_ID_MAX_LENGTH,
    VERSION_MAX_LENGTH, TAG_MAX_LENGTH, HASH_MAX_LENGTH,
    SUPPORTED_PLATFORMS, SUPPORTED_RESOURCE_TYPES, SUPPORTED_TOOL_TYPES,
    SUPPORTED_TOOL_SUBTYPES, RESERVED_WORDS, HASH_FORMAT_REGEX,
    COMPONENT_NAMES, DEFAULT_CONFIG
)


class TRNValidator:
    """
    TRN format and component validator.
    
    Provides static methods for validating TRN strings and components
    with comprehensive error reporting and caching for performance.
    """

    # Validation cache for performance
    _validation_cache: Dict[str, bool] = {}
    _cache_size = 0
    _max_cache_size = DEFAULT_CONFIG["max_cache_size"]

    @classmethod
    def validate(cls, trn_string: str, strict: bool = True) -> bool:
        """
        Validate a TRN string.
        
        Args:
            trn_string: TRN string to validate
            strict: Whether to use strict validation rules
            
        Returns:
            bool: True if valid
            
        Raises:
            TRNValidationError: If validation fails
        """
        # Check cache first
        cache_key = f"{trn_string}:{strict}"
        if DEFAULT_CONFIG["cache_validation_results"] and cache_key in cls._validation_cache:
            return cls._validation_cache[cache_key]

        try:
            # Basic format validation
            cls._validate_basic_format(trn_string)
            
            # Parse and validate components
            components = cls._parse_components(trn_string)
            cls._validate_components_dict(components, strict=strict)
            
            # Cache successful validation
            cls._cache_result(cache_key, True)
            return True
            
        except Exception as e:
            # Cache failure (but don't cache exception details)
            cls._cache_result(cache_key, False)
            raise

    @classmethod
    def validate_components(cls, components, strict: bool = True) -> bool:
        """
        Validate TRN components object.
        
        Args:
            components: TRNComponents object to validate
            strict: Whether to use strict validation rules
            
        Returns:
            bool: True if valid
            
        Raises:
            TRNValidationError: If validation fails
        """
        components_dict = components.to_dict()
        return cls._validate_components_dict(components_dict, strict=strict)

    @classmethod
    def _validate_basic_format(cls, trn_string: str) -> None:
        """Validate basic TRN format."""
        if not trn_string:
            raise TRNFormatError("TRN string cannot be empty")

        if not isinstance(trn_string, str):
            raise TRNFormatError(f"TRN must be a string, got {type(trn_string)}")

        # Length validation
        if len(trn_string) < TRN_MIN_LENGTH:
            raise TRNLengthError(
                f"TRN too short: {len(trn_string)} chars (minimum: {TRN_MIN_LENGTH})",
                trn=trn_string,
                length=len(trn_string),
                max_length=TRN_MIN_LENGTH
            )

        if len(trn_string) > TRN_MAX_LENGTH:
            raise TRNLengthError(
                f"TRN too long: {len(trn_string)} chars (maximum: {TRN_MAX_LENGTH})",
                trn=trn_string,
                length=len(trn_string),
                max_length=TRN_MAX_LENGTH
            )

        # Must start with "trn:"
        if not trn_string.startswith("trn:"):
            raise TRNFormatError("TRN must start with 'trn:'", trn=trn_string)

        # Regex validation
        if not TRN_REGEX.match(trn_string):
            raise TRNFormatError("TRN format does not match required pattern", trn=trn_string)

    @classmethod
    def _parse_components(cls, trn_string: str) -> Dict[str, Optional[str]]:
        """Parse TRN string into components."""
        match = TRN_REGEX.match(trn_string)
        if not match:
            raise TRNFormatError("Failed to parse TRN components", trn=trn_string)

        groups = match.groups()
        
        return {
            "platform": groups[0],
            "scope": groups[1],
            "resource_type": groups[2],
            "type": groups[3],
            "subtype": groups[4],
            "instance_id": groups[5],
            "version": groups[6],
            "tag": groups[7],
            "hash": groups[8]
        }

    @classmethod
    def _validate_components_dict(cls, components: Dict[str, Optional[str]], strict: bool = True) -> bool:
        """Validate components dictionary."""
        # Validate required components
        cls._validate_required_components(components)
        
        # Validate individual components
        cls._validate_individual_components(components, strict=strict)
        
        # Validate business rules
        if strict:
            cls._validate_business_rules(components)
        
        return True

    @classmethod
    def _validate_required_components(cls, components: Dict[str, Optional[str]]) -> None:
        """Validate that all required components are present."""
        required_fields = ["platform", "resource_type", "type", "instance_id", "version"]
        
        for field in required_fields:
            if not components.get(field):
                raise TRNComponentError(
                    f"{COMPONENT_NAMES[field]} is required but missing",
                    component=field
                )

    @classmethod
    def _validate_individual_components(cls, components: Dict[str, Optional[str]], strict: bool = True) -> None:
        """Validate each component individually."""
        validators = {
            "platform": (cls._validate_platform, PLATFORM_MAX_LENGTH),
            "scope": (cls._validate_scope, SCOPE_MAX_LENGTH),
            "resource_type": (cls._validate_resource_type, RESOURCE_TYPE_MAX_LENGTH),
            "type": (cls._validate_type, TYPE_MAX_LENGTH),
            "subtype": (cls._validate_subtype, SUBTYPE_MAX_LENGTH),
            "instance_id": (cls._validate_instance_id, INSTANCE_ID_MAX_LENGTH),
            "version": (cls._validate_version, VERSION_MAX_LENGTH),
            "tag": (cls._validate_tag, TAG_MAX_LENGTH),
            "hash": (cls._validate_hash, HASH_MAX_LENGTH)
        }

        for component_name, (validator_func, max_length) in validators.items():
            value = components.get(component_name)
            if value is not None:
                # Length check
                if len(value) > max_length:
                    raise TRNLengthError(
                        f"{COMPONENT_NAMES[component_name]} too long: {len(value)} chars (max: {max_length})",
                        length=len(value),
                        max_length=max_length
                    )
                
                # Component-specific validation
                validator_func(value, strict=strict)

    @classmethod
    def _validate_platform(cls, platform: str, strict: bool = True) -> None:
        """Validate platform component."""
        if strict and platform not in SUPPORTED_PLATFORMS:
            raise TRNValidationError(
                f"Unsupported platform '{platform}'. Supported: {sorted(SUPPORTED_PLATFORMS)}",
                rule="supported_platforms"
            )
        
        cls._check_reserved_words(platform, "platform")

    @classmethod
    def _validate_scope(cls, scope: str, strict: bool = True) -> None:
        """Validate scope component."""
        cls._check_reserved_words(scope, "scope")

    @classmethod
    def _validate_resource_type(cls, resource_type: str, strict: bool = True) -> None:
        """Validate resource type component."""
        if strict and resource_type not in SUPPORTED_RESOURCE_TYPES:
            raise TRNValidationError(
                f"Unsupported resource type '{resource_type}'. Supported: {sorted(SUPPORTED_RESOURCE_TYPES)}",
                rule="supported_resource_types"
            )
        
        cls._check_reserved_words(resource_type, "resource_type")

    @classmethod
    def _validate_type(cls, type_value: str, strict: bool = True) -> None:
        """Validate type component."""
        # For tool resources, validate against supported tool types
        if strict:
            # This would need context about resource_type, but for now we'll be lenient
            pass
        
        cls._check_reserved_words(type_value, "type")

    @classmethod
    def _validate_subtype(cls, subtype: str, strict: bool = True) -> None:
        """Validate subtype component."""
        if strict and subtype not in SUPPORTED_TOOL_SUBTYPES:
            # Only validate if it's a known subtype (allow custom subtypes)
            if subtype in {"async", "sync", "streaming", "batch", "realtime"}:
                if subtype not in SUPPORTED_TOOL_SUBTYPES:
                    raise TRNValidationError(
                        f"Unsupported subtype '{subtype}'. Supported: {sorted(SUPPORTED_TOOL_SUBTYPES)}",
                        rule="supported_subtypes"
                    )
        
        cls._check_reserved_words(subtype, "subtype")

    @classmethod
    def _validate_instance_id(cls, instance_id: str, strict: bool = True) -> None:
        """Validate instance ID component."""
        # Check for path-like characters that might cause security issues
        forbidden_chars = ["../", "./", "\\", "/"]
        for char in forbidden_chars:
            if char in instance_id:
                raise TRNCharacterError(
                    f"Instance ID cannot contain '{char}' for security reasons",
                    invalid_char=char
                )
        
        cls._check_reserved_words(instance_id, "instance_id")

    @classmethod
    def _validate_version(cls, version: str, strict: bool = True) -> None:
        """Validate version component."""
        # Allow common version formats and aliases
        version_patterns = [
            r"^v?\d+\.\d+\.\d+.*$",  # Semantic versioning
            r"^v?\d+\.\d+.*$",       # Major.minor
            r"^v?\d+.*$",            # Major only
            r"^(latest|stable|beta|alpha|dev|rc)$",  # Common aliases
        ]
        
        if strict:
            if not any(re.match(pattern, version, re.IGNORECASE) for pattern in version_patterns):
                raise TRNValidationError(
                    f"Version '{version}' does not match expected format (semantic versioning or common aliases)",
                    rule="version_format"
                )
        
        cls._check_reserved_words(version, "version")

    @classmethod
    def _validate_tag(cls, tag: str, strict: bool = True) -> None:
        """Validate tag component."""
        cls._check_reserved_words(tag, "tag")

    @classmethod
    def _validate_hash(cls, hash_value: str, strict: bool = True) -> None:
        """Validate hash component."""
        if strict and not HASH_FORMAT_REGEX.match(hash_value):
            raise TRNValidationError(
                f"Hash '{hash_value}' does not match format 'algorithm:hexvalue'",
                rule="hash_format"
            )

    @classmethod
    def _validate_business_rules(cls, components: Dict[str, Optional[str]]) -> None:
        """Validate business rules and component relationships."""
        # Rule: If scope is provided for 'user' platform, it should be a valid user identifier
        if components["platform"] == "user" and components["scope"]:
            scope = components["scope"]
            if len(scope) < 2:
                raise TRNValidationError(
                    "User scope must be at least 2 characters long",
                    rule="user_scope_length"
                )

        # Rule: If scope is provided for 'org' platform, it should be a valid org identifier
        if components["platform"] == "org" and not components["scope"]:
            raise TRNValidationError(
                "Organization platform requires a scope (organization name)",
                rule="org_scope_required"
            )

        # Rule: System platform should not have user-specific scopes
        if components["platform"] == "aiplatform" and components["scope"]:
            raise TRNValidationError(
                "System platform (aiplatform) should not have scope",
                rule="system_no_scope"
            )

    @classmethod
    def _check_reserved_words(cls, value: str, component: str) -> None:
        """Check if value contains reserved words."""
        if value.lower() in RESERVED_WORDS:
            raise TRNReservedWordError(
                f"'{value}' is a reserved word",
                reserved_word=value,
                component=component
            )

        # Check for words that start with double underscore
        if value.startswith("__"):
            raise TRNReservedWordError(
                f"Names starting with '__' are reserved for system use",
                reserved_word=value,
                component=component
            )

    @classmethod
    def _cache_result(cls, key: str, result: bool) -> None:
        """Cache validation result."""
        if not DEFAULT_CONFIG["cache_validation_results"]:
            return

        if cls._cache_size >= cls._max_cache_size:
            # Simple cache eviction: clear half the cache
            keys_to_remove = list(cls._validation_cache.keys())[:cls._cache_size // 2]
            for k in keys_to_remove:
                del cls._validation_cache[k]
            cls._cache_size = len(cls._validation_cache)

        cls._validation_cache[key] = result
        cls._cache_size += 1

    @classmethod
    def clear_cache(cls) -> None:
        """Clear the validation cache."""
        cls._validation_cache.clear()
        cls._cache_size = 0

    @classmethod
    def get_validation_errors(cls, trn_string: str, strict: bool = True) -> List[str]:
        """
        Get all validation errors for a TRN string without raising exceptions.
        
        Args:
            trn_string: TRN string to validate
            strict: Whether to use strict validation
            
        Returns:
            List[str]: List of error messages, empty if valid
        """
        errors = []
        
        try:
            cls.validate(trn_string, strict=strict)
        except TRNValidationError as e:
            errors.append(str(e))
        except Exception as e:
            errors.append(f"Validation error: {str(e)}")
        
        return errors

    @classmethod
    def suggest_fixes(cls, trn_string: str) -> List[str]:
        """
        Suggest fixes for common TRN validation errors.
        
        Args:
            trn_string: Invalid TRN string
            
        Returns:
            List[str]: List of suggested fixes
        """
        suggestions = []
        
        if not trn_string:
            suggestions.append("Provide a non-empty TRN string")
            return suggestions

        if not trn_string.startswith("trn:"):
            suggestions.append("TRN must start with 'trn:'")

        # Check for common format issues
        parts = trn_string.split(":")
        if len(parts) < 6:  # Minimum required parts
            suggestions.append("TRN requires at least: trn:platform:resource_type:type:instance_id:version")

        # Check for uppercase (should be lowercase)
        if trn_string != trn_string.lower():
            suggestions.append("Convert TRN to lowercase")

        # Check for spaces
        if " " in trn_string:
            suggestions.append("Remove spaces from TRN")

        # Check for invalid characters
        invalid_chars = set(trn_string) - set("abcdefghijklmnopqrstuvwxyz0123456789:-@.")
        if invalid_chars:
            suggestions.append(f"Remove invalid characters: {', '.join(sorted(invalid_chars))}")

        return suggestions

    @classmethod
    @lru_cache(maxsize=100)
    def is_valid_component_value(cls, value: str, component_type: str) -> bool:
        """
        Check if a value is valid for a specific component type.
        
        Args:
            value: Value to check
            component_type: Type of component (platform, scope, etc.)
            
        Returns:
            bool: True if valid
        """
        if not value:
            return False

        try:
            # Create a mock components dict for validation
            components = {
                "platform": "aiplatform",
                "resource_type": "tool",
                "type": "openapi",
                "instance_id": "test",
                "version": "v1.0",
                component_type: value
            }
            
            cls._validate_individual_components(components, strict=True)
            return True
        except:
            return False 