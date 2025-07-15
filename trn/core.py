"""
TRN Core Classes

Defines the main TRN class and related data structures.
Provides the primary interface for working with TRN objects.
"""

from typing import Optional, Dict, Any, Union
from dataclasses import dataclass, field
import warnings

from .exceptions import (
    TRNError, TRNFormatError, TRNValidationError, 
    TRNComponentError, TRNLengthError, TRNDeprecationWarning
)
from .constants import (
    TRN_MAX_LENGTH, TRN_MIN_LENGTH, TRN_SEPARATOR, TRN_HASH_SEPARATOR,
    SUPPORTED_PLATFORMS, SUPPORTED_RESOURCE_TYPES, RESERVED_WORDS,
    DEFAULT_CONFIG, COMPONENT_NAMES
)


@dataclass(frozen=True)
class TRNComponents:
    """
    Immutable data structure representing TRN components.
    
    Attributes:
        platform: Platform identifier (required)
        scope: Tenant/group identifier (optional)
        resource_type: Resource type (required, defaults to 'tool')
        type: Specific type (required)
        subtype: Sub-type (optional)
        instance_id: Instance identifier (required)
        version: Version identifier (required)
        tag: Version tag (optional)
        hash: Content hash (optional)
    """
    platform: str
    resource_type: str
    type: str
    instance_id: str
    version: str
    scope: Optional[str] = None
    subtype: Optional[str] = None
    tag: Optional[str] = None
    hash: Optional[str] = None

    def __post_init__(self):
        """Validate components after initialization."""
        # Check required fields
        if not self.platform:
            raise TRNComponentError("Platform is required", "platform")
        if not self.resource_type:
            raise TRNComponentError("Resource type is required", "resource_type")
        if not self.type:
            raise TRNComponentError("Type is required", "type")
        if not self.instance_id:
            raise TRNComponentError("Instance ID is required", "instance_id")
        if not self.version:
            raise TRNComponentError("Version is required", "version")

        # Validate platform
        if self.platform not in SUPPORTED_PLATFORMS:
            raise TRNValidationError(
                f"Unsupported platform '{self.platform}'. Supported: {SUPPORTED_PLATFORMS}",
                rule="supported_platforms"
            )

        # Validate resource type
        if self.resource_type not in SUPPORTED_RESOURCE_TYPES:
            raise TRNValidationError(
                f"Unsupported resource type '{self.resource_type}'. Supported: {SUPPORTED_RESOURCE_TYPES}",
                rule="supported_resource_types"
            )

        # Check for reserved words
        for field_name, value in self.__dict__.items():
            if value and value.lower() in RESERVED_WORDS:
                raise TRNValidationError(
                    f"'{value}' is a reserved word and cannot be used in {COMPONENT_NAMES.get(field_name, field_name)}",
                    rule="reserved_words"
                )

    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary representation."""
        return {
            "platform": self.platform,
            "scope": self.scope,
            "resource_type": self.resource_type,
            "type": self.type,
            "subtype": self.subtype,
            "instance_id": self.instance_id,
            "version": self.version,
            "tag": self.tag,
            "hash": self.hash,
        }

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> 'TRNComponents':
        """Create from dictionary representation."""
        return cls(**data)


class TRN:
    """
    Main TRN (Tool Resource Name) class.
    
    Provides parsing, validation, and manipulation of TRN identifiers.
    Supports the format: trn:platform[:scope]:resource_type:type[:subtype]:instance_id:version[:tag][@hash]
    
    Examples:
        >>> trn = TRN.parse("trn:user:alice:tool:openapi:github-api:v1.0")
        >>> print(trn.platform)  # "user"
        >>> print(trn.scope)     # "alice"
        >>> print(trn.instance_id)  # "github-api"
        
        >>> trn = TRN(
        ...     platform="aiplatform",
        ...     resource_type="tool",
        ...     type="openapi",
        ...     instance_id="github-api",
        ...     version="v1.0"
        ... )
        >>> print(str(trn))  # "trn:aiplatform:tool:openapi:github-api:v1.0"
    """

    def __init__(
        self,
        platform: str,
        resource_type: str,
        type: str,
        instance_id: str,
        version: str,
        scope: Optional[str] = None,
        subtype: Optional[str] = None,
        tag: Optional[str] = None,
        hash: Optional[str] = None,
        validate: bool = None
    ):
        """
        Initialize a TRN object.
        
        Args:
            platform: Platform identifier
            resource_type: Resource type
            type: Specific type
            instance_id: Instance identifier
            version: Version identifier
            scope: Optional tenant/group identifier
            subtype: Optional sub-type
            tag: Optional version tag
            hash: Optional content hash
            validate: Whether to validate on creation (defaults to config setting)
        """
        self._components = TRNComponents(
            platform=platform,
            scope=scope,
            resource_type=resource_type,
            type=type,
            subtype=subtype,
            instance_id=instance_id,
            version=version,
            tag=tag,
            hash=hash
        )
        
        self._string_cache: Optional[str] = None
        self._config = DEFAULT_CONFIG.copy()
        
        # Validate if requested
        if validate if validate is not None else self._config["validate_on_create"]:
            self.validate()

    @property
    def platform(self) -> str:
        """Platform identifier."""
        return self._components.platform

    @property
    def scope(self) -> Optional[str]:
        """Scope/tenant identifier."""
        return self._components.scope

    @property
    def resource_type(self) -> str:
        """Resource type."""
        return self._components.resource_type

    @property
    def type(self) -> str:
        """Specific type."""
        return self._components.type

    @property
    def subtype(self) -> Optional[str]:
        """Sub-type."""
        return self._components.subtype

    @property
    def instance_id(self) -> str:
        """Instance identifier."""
        return self._components.instance_id

    @property
    def version(self) -> str:
        """Version identifier."""
        return self._components.version

    @property
    def tag(self) -> Optional[str]:
        """Version tag."""
        return self._components.tag

    @property
    def hash(self) -> Optional[str]:
        """Content hash."""
        return self._components.hash

    @property
    def components(self) -> TRNComponents:
        """Get TRN components."""
        return self._components

    def __str__(self) -> str:
        """String representation of TRN."""
        if self._string_cache is None:
            self._string_cache = self._build_string()
        return self._string_cache

    def __repr__(self) -> str:
        """Developer representation of TRN."""
        return f"TRN('{str(self)}')"

    def __eq__(self, other) -> bool:
        """Equality comparison."""
        if not isinstance(other, TRN):
            return False
        return self._components == other._components

    def __hash__(self) -> int:
        """Hash for use in sets and dictionaries."""
        return hash(self._components)

    def _build_string(self) -> str:
        """Build the TRN string representation."""
        parts = ["trn", self.platform]
        
        # Add scope if present
        if self.scope:
            parts.append(self.scope)
        
        # Add required parts
        parts.extend([self.resource_type, self.type])
        
        # Add subtype if present
        if self.subtype:
            parts.append(self.subtype)
        
        # Add required parts
        parts.extend([self.instance_id, self.version])
        
        # Add tag if present
        if self.tag:
            parts.append(self.tag)
        
        # Build main TRN string
        trn_string = TRN_SEPARATOR.join(parts)
        
        # Add hash if present
        if self.hash:
            trn_string += TRN_HASH_SEPARATOR + self.hash
        
        return trn_string

    def validate(self, strict: bool = None) -> bool:
        """
        Validate the TRN format and components.
        
        Args:
            strict: Whether to use strict validation rules
            
        Returns:
            bool: True if valid
            
        Raises:
            TRNValidationError: If validation fails
        """
        from .validator import TRNValidator
        
        strict = strict if strict is not None else self._config["strict_validation"]
        return TRNValidator.validate_components(self._components, strict=strict)

    def normalize(self) -> 'TRN':
        """
        Create a normalized version of this TRN.
        
        Returns:
            TRN: Normalized TRN object
        """
        # For now, just return a copy since we already store in normalized format
        return TRN(
            platform=self.platform,
            scope=self.scope,
            resource_type=self.resource_type,
            type=self.type,
            subtype=self.subtype,
            instance_id=self.instance_id,
            version=self.version,
            tag=self.tag,
            hash=self.hash,
            validate=False  # Already validated
        )

    def without_hash(self) -> 'TRN':
        """
        Create a copy of this TRN without the hash component.
        
        Returns:
            TRN: New TRN object without hash
        """
        return TRN(
            platform=self.platform,
            scope=self.scope,
            resource_type=self.resource_type,
            type=self.type,
            subtype=self.subtype,
            instance_id=self.instance_id,
            version=self.version,
            tag=self.tag,
            hash=None,
            validate=False
        )

    def without_tag(self) -> 'TRN':
        """
        Create a copy of this TRN without the tag component.
        
        Returns:
            TRN: New TRN object without tag
        """
        return TRN(
            platform=self.platform,
            scope=self.scope,
            resource_type=self.resource_type,
            type=self.type,
            subtype=self.subtype,
            instance_id=self.instance_id,
            version=self.version,
            tag=None,
            hash=self.hash,
            validate=False
        )

    def get_base_trn(self) -> 'TRN':
        """
        Get the base TRN (without version, tag, and hash).
        
        Returns:
            TRN: Base TRN object
        """
        return TRN(
            platform=self.platform,
            scope=self.scope,
            resource_type=self.resource_type,
            type=self.type,
            subtype=self.subtype,
            instance_id=self.instance_id,
            version="*",  # Wildcard for base
            tag=None,
            hash=None,
            validate=False
        )

    def matches_pattern(self, pattern: str) -> bool:
        """
        Check if this TRN matches a pattern (supports wildcards).
        
        Args:
            pattern: Pattern string with optional wildcards (*)
            
        Returns:
            bool: True if matches pattern
        """
        from .utils import match_trn_pattern
        return match_trn_pattern(str(self), pattern)

    def is_compatible_with(self, other: Union['TRN', str]) -> bool:
        """
        Check if this TRN is compatible with another TRN.
        
        Args:
            other: Another TRN object or string
            
        Returns:
            bool: True if compatible
        """
        if isinstance(other, str):
            other = TRN.parse(other)
        
        # Same base components must match
        return (
            self.platform == other.platform and
            self.scope == other.scope and
            self.resource_type == other.resource_type and
            self.type == other.type and
            self.subtype == other.subtype and
            self.instance_id == other.instance_id
        )

    def to_dict(self) -> Dict[str, Any]:
        """Convert TRN to dictionary representation."""
        return self._components.to_dict()

    def to_url(self) -> str:
        """Convert TRN to URL format."""
        from .url_converter import TRNURLConverter
        return TRNURLConverter.to_url(self)

    @classmethod
    def parse(cls, trn_string: str, validate: bool = True) -> 'TRN':
        """
        Parse a TRN string into a TRN object.
        
        Args:
            trn_string: TRN string to parse
            validate: Whether to validate the parsed TRN
            
        Returns:
            TRN: Parsed TRN object
            
        Raises:
            TRNFormatError: If parsing fails
        """
        from .parser import TRNParser
        return TRNParser.parse(trn_string, validate=validate)

    @classmethod
    def from_components(cls, components: TRNComponents, validate: bool = True) -> 'TRN':
        """
        Create TRN from TRNComponents.
        
        Args:
            components: TRN components
            validate: Whether to validate
            
        Returns:
            TRN: New TRN object
        """
        return cls(
            platform=components.platform,
            scope=components.scope,
            resource_type=components.resource_type,
            type=components.type,
            subtype=components.subtype,
            instance_id=components.instance_id,
            version=components.version,
            tag=components.tag,
            hash=components.hash,
            validate=validate
        )

    @classmethod
    def from_dict(cls, data: Dict[str, Any], validate: bool = True) -> 'TRN':
        """
        Create TRN from dictionary.
        
        Args:
            data: Dictionary with TRN component data
            validate: Whether to validate
            
        Returns:
            TRN: New TRN object
        """
        components = TRNComponents.from_dict(data)
        return cls.from_components(components, validate=validate)

    @classmethod
    def from_url(cls, url: str, validate: bool = True) -> 'TRN':
        """
        Create TRN from URL format.
        
        Args:
            url: TRN URL string
            validate: Whether to validate
            
        Returns:
            TRN: New TRN object
        """
        from .url_converter import TRNURLConverter
        return TRNURLConverter.from_url(url, validate=validate) 