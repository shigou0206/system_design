"""
TRN Parser

Provides parsing functionality to convert TRN strings into TRN objects.
Handles normalization, component extraction, and error recovery.
"""

import re
from typing import Dict, Optional, Tuple
from functools import lru_cache

from .exceptions import TRNFormatError, TRNValidationError
from .constants import TRN_REGEX, TRN_HASH_SEPARATOR, DEFAULT_CONFIG


class TRNParser:
    """
    TRN string parser.
    
    Provides static methods for parsing TRN strings into component objects
    with support for normalization and error recovery.
    """

    @classmethod
    def parse(cls, trn_string: str, validate: bool = True, normalize: bool = None) -> 'TRN':
        """
        Parse a TRN string into a TRN object.
        
        Args:
            trn_string: TRN string to parse
            validate: Whether to validate the parsed components
            normalize: Whether to normalize the TRN (defaults to config setting)
            
        Returns:
            TRN: Parsed TRN object
            
        Raises:
            TRNFormatError: If parsing fails
            TRNValidationError: If validation fails
        """
        from .core import TRN
        
        if not trn_string:
            raise TRNFormatError("TRN string cannot be empty")

        if not isinstance(trn_string, str):
            raise TRNFormatError(f"TRN must be a string, got {type(trn_string)}")

        # Normalize if requested
        normalize = normalize if normalize is not None else DEFAULT_CONFIG["normalize_on_parse"]
        if normalize:
            trn_string = cls.normalize_string(trn_string)

        # Parse components
        components = cls.parse_components(trn_string)
        
        # Create TRN object
        return TRN(
            platform=components["platform"],
            scope=components["scope"],
            resource_type=components["resource_type"],
            type=components["type"],
            subtype=components["subtype"],
            instance_id=components["instance_id"],
            version=components["version"],
            tag=components["tag"],
            hash=components["hash"],
            validate=validate
        )

    @classmethod
    def parse_components(cls, trn_string: str) -> Dict[str, Optional[str]]:
        """
        Parse TRN string into component dictionary.
        
        Args:
            trn_string: TRN string to parse
            
        Returns:
            Dict[str, Optional[str]]: Dictionary of TRN components
            
        Raises:
            TRNFormatError: If parsing fails
        """
        # Basic validation
        if not trn_string.startswith("trn:"):
            raise TRNFormatError("TRN must start with 'trn:'", trn=trn_string)

        # Try regex parsing first
        match = TRN_REGEX.match(trn_string)
        if match:
            return cls._extract_components_from_match(match)

        # If regex fails, try recovery parsing
        return cls._recovery_parse(trn_string)

    @classmethod
    def _extract_components_from_match(cls, match: re.Match) -> Dict[str, Optional[str]]:
        """Extract components from regex match."""
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
    def _recovery_parse(cls, trn_string: str) -> Dict[str, Optional[str]]:
        """
        Attempt to parse malformed TRN strings with error recovery.
        
        Args:
            trn_string: TRN string to parse
            
        Returns:
            Dict[str, Optional[str]]: Dictionary of TRN components
            
        Raises:
            TRNFormatError: If recovery parsing also fails
        """
        # Split by hash first if present
        hash_value = None
        if TRN_HASH_SEPARATOR in trn_string:
            main_part, hash_part = trn_string.rsplit(TRN_HASH_SEPARATOR, 1)
            hash_value = hash_part
        else:
            main_part = trn_string

        # Split main part by colons
        parts = main_part.split(":")
        
        if len(parts) < 6:  # Minimum: trn:platform:resource_type:type:instance_id:version
            raise TRNFormatError(
                f"TRN requires at least 6 components, got {len(parts)}. "
                f"Format: trn:platform:resource_type:type:instance_id:version",
                trn=trn_string
            )

        # Remove 'trn' prefix
        if parts[0] != "trn":
            raise TRNFormatError("TRN must start with 'trn:'", trn=trn_string)
        
        parts = parts[1:]  # Remove 'trn'

        try:
            # Map components based on number of parts
            return cls._map_recovery_components(parts, hash_value, trn_string)
        except Exception as e:
            raise TRNFormatError(f"Failed to parse TRN components: {str(e)}", trn=trn_string)

    @classmethod
    def _map_recovery_components(cls, parts: list, hash_value: Optional[str], original_trn: str) -> Dict[str, Optional[str]]:
        """Map parsed parts to component dictionary."""
        components = {
            "platform": None,
            "scope": None,
            "resource_type": None,
            "type": None,
            "subtype": None,
            "instance_id": None,
            "version": None,
            "tag": None,
            "hash": hash_value
        }

        if len(parts) == 5:
            # trn:platform:resource_type:type:instance_id:version
            components.update({
                "platform": parts[0],
                "resource_type": parts[1],
                "type": parts[2],
                "instance_id": parts[3],
                "version": parts[4]
            })
        elif len(parts) == 6:
            # Could be: platform:scope:resource_type:type:instance_id:version
            # Or: platform:resource_type:type:subtype:instance_id:version
            # Or: platform:resource_type:type:instance_id:version:tag
            
            # Try to determine the structure by checking if second part looks like scope
            if cls._looks_like_scope(parts[1], parts[0]):
                # Assume: platform:scope:resource_type:type:instance_id:version
                components.update({
                    "platform": parts[0],
                    "scope": parts[1],
                    "resource_type": parts[2],
                    "type": parts[3],
                    "instance_id": parts[4],
                    "version": parts[5]
                })
            elif cls._looks_like_version(parts[5]):
                # Assume: platform:resource_type:type:instance_id:version:tag
                components.update({
                    "platform": parts[0],
                    "resource_type": parts[1],
                    "type": parts[2],
                    "instance_id": parts[3],
                    "version": parts[4],
                    "tag": parts[5]
                })
            else:
                # Assume: platform:resource_type:type:subtype:instance_id:version
                components.update({
                    "platform": parts[0],
                    "resource_type": parts[1],
                    "type": parts[2],
                    "subtype": parts[3],
                    "instance_id": parts[4],
                    "version": parts[5]
                })
        elif len(parts) == 7:
            # Could be several combinations, use heuristics
            if cls._looks_like_scope(parts[1], parts[0]):
                # platform:scope:resource_type:type:subtype:instance_id:version
                components.update({
                    "platform": parts[0],
                    "scope": parts[1],
                    "resource_type": parts[2],
                    "type": parts[3],
                    "subtype": parts[4],
                    "instance_id": parts[5],
                    "version": parts[6]
                })
            elif cls._looks_like_version(parts[6]):
                # platform:resource_type:type:subtype:instance_id:version:tag
                components.update({
                    "platform": parts[0],
                    "resource_type": parts[1],
                    "type": parts[2],
                    "subtype": parts[3],
                    "instance_id": parts[4],
                    "version": parts[5],
                    "tag": parts[6]
                })
            else:
                raise TRNFormatError(f"Ambiguous TRN format with 7 components", trn=original_trn)
        elif len(parts) == 8:
            # Full format: platform:scope:resource_type:type:subtype:instance_id:version:tag
            components.update({
                "platform": parts[0],
                "scope": parts[1],
                "resource_type": parts[2],
                "type": parts[3],
                "subtype": parts[4],
                "instance_id": parts[5],
                "version": parts[6],
                "tag": parts[7]
            })
        else:
            raise TRNFormatError(f"Too many components: {len(parts)} (maximum: 8)", trn=original_trn)

        return components

    @classmethod
    def _looks_like_scope(cls, value: str, platform: str) -> bool:
        """Heuristic to determine if a value looks like a scope."""
        # For user platform, scope is typically a username
        if platform == "user":
            return len(value) >= 2 and value.isalnum()
        
        # For org platform, scope is required and looks like org name
        if platform == "org":
            return len(value) >= 2
        
        # For system platform, scope is not expected
        if platform == "aiplatform":
            return False
        
        # Default heuristic: short alphanumeric strings
        return len(value) <= 32 and value.replace("-", "").isalnum()

    @classmethod
    def _looks_like_version(cls, value: str) -> bool:
        """Heuristic to determine if a value looks like a version."""
        version_patterns = [
            r"^v?\d+\.\d+\.\d+",  # Semantic versioning
            r"^v?\d+\.\d+",       # Major.minor
            r"^v?\d+",            # Major only
            r"^(latest|stable|beta|alpha|dev|rc)$",  # Common aliases
        ]
        
        return any(re.match(pattern, value, re.IGNORECASE) for pattern in version_patterns)

    @classmethod
    def normalize_string(cls, trn_string: str) -> str:
        """
        Normalize a TRN string.
        
        Args:
            trn_string: TRN string to normalize
            
        Returns:
            str: Normalized TRN string
        """
        if not trn_string:
            return trn_string

        # Convert to lowercase
        normalized = trn_string.lower()
        
        # Remove extra whitespace
        normalized = normalized.strip()
        
        # Remove multiple consecutive colons (except in hash)
        parts = normalized.split(TRN_HASH_SEPARATOR)
        main_part = parts[0]
        hash_part = parts[1] if len(parts) > 1 else None
        
        # Fix multiple colons in main part
        main_part = re.sub(r":+", ":", main_part)
        
        # Reconstruct
        if hash_part:
            normalized = main_part + TRN_HASH_SEPARATOR + hash_part
        else:
            normalized = main_part
        
        return normalized

    @classmethod
    @lru_cache(maxsize=100)
    def extract_base_trn(cls, trn_string: str) -> str:
        """
        Extract base TRN (without version, tag, and hash).
        
        Args:
            trn_string: Full TRN string
            
        Returns:
            str: Base TRN string
        """
        try:
            components = cls.parse_components(trn_string)
            
            # Build base TRN
            parts = ["trn", components["platform"]]
            
            if components["scope"]:
                parts.append(components["scope"])
            
            parts.extend([
                components["resource_type"],
                components["type"]
            ])
            
            if components["subtype"]:
                parts.append(components["subtype"])
            
            parts.append(components["instance_id"])
            
            return ":".join(parts)
        except:
            # Fallback: try to extract manually
            return cls._extract_base_trn_fallback(trn_string)

    @classmethod
    def _extract_base_trn_fallback(cls, trn_string: str) -> str:
        """Fallback method to extract base TRN."""
        # Remove hash if present
        if TRN_HASH_SEPARATOR in trn_string:
            trn_string = trn_string.split(TRN_HASH_SEPARATOR)[0]
        
        # Split by colons and remove last 1-2 parts (version and possibly tag)
        parts = trn_string.split(":")
        if len(parts) >= 6:
            # Remove version (and tag if present)
            if len(parts) >= 7 and not cls._looks_like_version(parts[-2]):
                # Last part might be tag, second-to-last is version
                parts = parts[:-2]
            else:
                # Last part is version
                parts = parts[:-1]
        
        return ":".join(parts)

    @classmethod
    def validate_and_parse(cls, trn_string: str, strict: bool = True) -> Tuple['TRN', bool]:
        """
        Validate and parse TRN string, returning success status.
        
        Args:
            trn_string: TRN string to parse
            strict: Whether to use strict validation
            
        Returns:
            Tuple[TRN, bool]: (TRN object or None, success flag)
        """
        try:
            trn = cls.parse(trn_string, validate=strict)
            return trn, True
        except Exception:
            return None, False

    @classmethod
    def get_component_at_position(cls, trn_string: str, position: int) -> Optional[Tuple[str, str]]:
        """
        Get the component name and value at a specific character position.
        
        Args:
            trn_string: TRN string
            position: Character position
            
        Returns:
            Optional[Tuple[str, str]]: (component_name, component_value) or None
        """
        try:
            # Split by colons to find component boundaries
            parts = trn_string.split(":")
            current_pos = 0
            
            for i, part in enumerate(parts):
                start_pos = current_pos
                end_pos = current_pos + len(part)
                
                if start_pos <= position < end_pos:
                    # Map index to component name
                    component_map = {
                        0: "prefix",  # "trn"
                        1: "platform",
                        2: "scope_or_resource_type",
                        3: "resource_type_or_type",
                        4: "type_or_subtype_or_instance",
                        5: "subtype_or_instance_or_version",
                        6: "instance_or_version_or_tag",
                        7: "version_or_tag",
                        8: "tag"
                    }
                    
                    component_name = component_map.get(i, "unknown")
                    return component_name, part
                
                current_pos = end_pos + 1  # +1 for the colon
            
            # Check if position is in hash part
            if TRN_HASH_SEPARATOR in trn_string:
                hash_pos = trn_string.find(TRN_HASH_SEPARATOR)
                if position >= hash_pos:
                    hash_value = trn_string[hash_pos + 1:]
                    return "hash", hash_value
            
            return None
        except:
            return None 