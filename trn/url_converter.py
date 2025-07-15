"""
TRN URL Converter

Provides conversion between TRN objects/strings and URL format.
Supports both trn:// URLs and standard HTTP URLs for frontend integration.
"""

import urllib.parse
from typing import Dict, Optional
from functools import lru_cache

from .exceptions import TRNFormatError, TRNValidationError
from .constants import TRN_URL_REGEX, URL_ENCODING_MAP


class TRNURLConverter:
    """
    TRN to URL converter.
    
    Provides static methods for converting between TRN format and URL format
    for frontend integration and direct navigation.
    """

    @classmethod
    def to_url(cls, trn: 'TRN', scheme: str = "trn") -> str:
        """
        Convert TRN object to URL format.
        
        Args:
            trn: TRN object to convert
            scheme: URL scheme to use ('trn' or 'https')
            
        Returns:
            str: URL representation of TRN
            
        Examples:
            >>> trn = TRN.parse("trn:user:alice:tool:openapi:github-api:v1.0")
            >>> TRNURLConverter.to_url(trn)
            "trn://user/alice/tool/openapi/github-api/v1.0"
        """
        if scheme == "trn":
            return cls._to_trn_url(trn)
        elif scheme in ["http", "https"]:
            return cls._to_http_url(trn, scheme)
        else:
            raise ValueError(f"Unsupported URL scheme: {scheme}")

    @classmethod
    def from_url(cls, url: str, validate: bool = True) -> 'TRN':
        """
        Convert URL to TRN object.
        
        Args:
            url: URL string to convert
            validate: Whether to validate the resulting TRN
            
        Returns:
            TRN: TRN object parsed from URL
            
        Raises:
            TRNFormatError: If URL format is invalid
            
        Examples:
            >>> url = "trn://user/alice/tool/openapi/github-api/v1.0"
            >>> trn = TRNURLConverter.from_url(url)
        """
        if url.startswith("trn://"):
            return cls._from_trn_url(url, validate=validate)
        elif url.startswith(("http://", "https://")):
            return cls._from_http_url(url, validate=validate)
        else:
            raise TRNFormatError(f"Unsupported URL format: {url}")

    @classmethod
    def _to_trn_url(cls, trn: 'TRN') -> str:
        """Convert TRN to trn:// URL format."""
        path_parts = ["trn://", trn.platform]
        
        # Add scope if present
        if trn.scope:
            path_parts.extend(["/", cls._encode_component(trn.scope)])
        
        # Add required parts
        path_parts.extend([
            "/", trn.resource_type,
            "/", trn.type
        ])
        
        # Add subtype if present
        if trn.subtype:
            path_parts.extend(["/", cls._encode_component(trn.subtype)])
        
        # Add required parts
        path_parts.extend([
            "/", cls._encode_component(trn.instance_id),
            "/", cls._encode_component(trn.version)
        ])
        
        # Add tag if present
        if trn.tag:
            path_parts.extend(["/", cls._encode_component(trn.tag)])
        
        # Build URL
        url = "".join(path_parts)
        
        # Add hash as query parameter if present
        if trn.hash:
            url += f"?hash={urllib.parse.quote(trn.hash)}"
        
        return url

    @classmethod
    def _from_trn_url(cls, url: str, validate: bool = True) -> 'TRN':
        """Parse trn:// URL into TRN object."""
        from .core import TRN
        
        # Parse URL components
        parsed = urllib.parse.urlparse(url)
        
        if parsed.scheme != "trn":
            raise TRNFormatError(f"Expected 'trn://' scheme, got '{parsed.scheme}://'")
        
        # Extract hash from query parameters
        hash_value = None
        if parsed.query:
            query_params = urllib.parse.parse_qs(parsed.query)
            if "hash" in query_params:
                hash_value = query_params["hash"][0]
        
        # Parse path components
        path = parsed.netloc + parsed.path
        if path.startswith("/"):
            path = path[1:]
        if path.endswith("/"):
            path = path[:-1]
        
        parts = [cls._decode_component(part) for part in path.split("/") if part]
        
        if len(parts) < 5:  # Minimum: platform/resource_type/type/instance_id/version
            raise TRNFormatError(
                f"TRN URL requires at least 5 path components, got {len(parts)}. "
                f"Format: trn://platform[/scope]/resource_type/type[/subtype]/instance_id/version[/tag]"
            )
        
        # Map components based on number of parts
        components = cls._map_url_components(parts, hash_value)
        
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
    def _map_url_components(cls, parts: list, hash_value: Optional[str]) -> Dict[str, Optional[str]]:
        """Map URL path components to TRN components."""
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
            # platform/resource_type/type/instance_id/version
            components.update({
                "platform": parts[0],
                "resource_type": parts[1],
                "type": parts[2],
                "instance_id": parts[3],
                "version": parts[4]
            })
        elif len(parts) == 6:
            # Could be: platform/scope/resource_type/type/instance_id/version
            # Or: platform/resource_type/type/subtype/instance_id/version
            # Or: platform/resource_type/type/instance_id/version/tag
            
            # Use heuristics similar to parser
            if cls._looks_like_scope(parts[1], parts[0]):
                # platform/scope/resource_type/type/instance_id/version
                components.update({
                    "platform": parts[0],
                    "scope": parts[1],
                    "resource_type": parts[2],
                    "type": parts[3],
                    "instance_id": parts[4],
                    "version": parts[5]
                })
            elif cls._looks_like_version(parts[4]):
                # platform/resource_type/type/instance_id/version/tag
                components.update({
                    "platform": parts[0],
                    "resource_type": parts[1],
                    "type": parts[2],
                    "instance_id": parts[3],
                    "version": parts[4],
                    "tag": parts[5]
                })
            else:
                # platform/resource_type/type/subtype/instance_id/version
                components.update({
                    "platform": parts[0],
                    "resource_type": parts[1],
                    "type": parts[2],
                    "subtype": parts[3],
                    "instance_id": parts[4],
                    "version": parts[5]
                })
        elif len(parts) == 7:
            # Could be several combinations
            if cls._looks_like_scope(parts[1], parts[0]):
                if cls._looks_like_version(parts[5]):
                    # platform/scope/resource_type/type/instance_id/version/tag
                    components.update({
                        "platform": parts[0],
                        "scope": parts[1],
                        "resource_type": parts[2],
                        "type": parts[3],
                        "instance_id": parts[4],
                        "version": parts[5],
                        "tag": parts[6]
                    })
                else:
                    # platform/scope/resource_type/type/subtype/instance_id/version
                    components.update({
                        "platform": parts[0],
                        "scope": parts[1],
                        "resource_type": parts[2],
                        "type": parts[3],
                        "subtype": parts[4],
                        "instance_id": parts[5],
                        "version": parts[6]
                    })
            else:
                # platform/resource_type/type/subtype/instance_id/version/tag
                components.update({
                    "platform": parts[0],
                    "resource_type": parts[1],
                    "type": parts[2],
                    "subtype": parts[3],
                    "instance_id": parts[4],
                    "version": parts[5],
                    "tag": parts[6]
                })
        elif len(parts) == 8:
            # Full format: platform/scope/resource_type/type/subtype/instance_id/version/tag
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
            raise TRNFormatError(f"Too many URL path components: {len(parts)} (maximum: 8)")

        return components

    @classmethod
    def _to_http_url(cls, trn: 'TRN', scheme: str = "https") -> str:
        """Convert TRN to HTTP URL format for frontend."""
        # Build path based on TRN components
        path_parts = ["/tools", trn.platform]
        
        # Add scope if present
        if trn.scope:
            path_parts.append(trn.scope)
        
        # Add type components
        path_parts.extend([trn.type])
        
        # Add subtype if present
        if trn.subtype:
            path_parts.append(trn.subtype)
        
        # Add instance and version
        path_parts.extend([trn.instance_id, trn.version])
        
        # Add tag if present
        if trn.tag:
            path_parts.append(trn.tag)
        
        # Build URL
        path = "/".join(cls._encode_component(part) for part in path_parts)
        url = f"{scheme}://your-platform.com{path}"
        
        # Add hash as query parameter if present
        if trn.hash:
            url += f"?hash={urllib.parse.quote(trn.hash)}"
        
        return url

    @classmethod
    def _from_http_url(cls, url: str, validate: bool = True) -> 'TRN':
        """Parse HTTP URL into TRN object."""
        from .core import TRN
        
        # Parse URL
        parsed = urllib.parse.urlparse(url)
        
        # Extract hash from query parameters
        hash_value = None
        if parsed.query:
            query_params = urllib.parse.parse_qs(parsed.query)
            if "hash" in query_params:
                hash_value = query_params["hash"][0]
        
        # Parse path - should start with /tools
        path_parts = [part for part in parsed.path.split("/") if part]
        
        if not path_parts or path_parts[0] != "tools":
            raise TRNFormatError("HTTP URL path must start with '/tools'")
        
        # Remove 'tools' prefix
        parts = [cls._decode_component(part) for part in path_parts[1:]]
        
        if len(parts) < 4:  # Minimum: platform/type/instance_id/version
            raise TRNFormatError(
                f"HTTP URL requires at least 4 path components after /tools, got {len(parts)}"
            )
        
        # Map components (assuming resource_type is 'tool' for HTTP URLs)
        components = cls._map_http_url_components(parts, hash_value)
        
        return TRN(
            platform=components["platform"],
            scope=components["scope"],
            resource_type="tool",  # Default for HTTP URLs
            type=components["type"],
            subtype=components["subtype"],
            instance_id=components["instance_id"],
            version=components["version"],
            tag=components["tag"],
            hash=components["hash"],
            validate=validate
        )

    @classmethod
    def _map_http_url_components(cls, parts: list, hash_value: Optional[str]) -> Dict[str, Optional[str]]:
        """Map HTTP URL path components to TRN components."""
        components = {
            "platform": None,
            "scope": None,
            "type": None,
            "subtype": None,
            "instance_id": None,
            "version": None,
            "tag": None,
            "hash": hash_value
        }

        if len(parts) == 4:
            # platform/type/instance_id/version
            components.update({
                "platform": parts[0],
                "type": parts[1],
                "instance_id": parts[2],
                "version": parts[3]
            })
        elif len(parts) == 5:
            # Could be: platform/scope/type/instance_id/version
            # Or: platform/type/subtype/instance_id/version
            # Or: platform/type/instance_id/version/tag
            
            if cls._looks_like_scope(parts[1], parts[0]):
                # platform/scope/type/instance_id/version
                components.update({
                    "platform": parts[0],
                    "scope": parts[1],
                    "type": parts[2],
                    "instance_id": parts[3],
                    "version": parts[4]
                })
            elif cls._looks_like_version(parts[3]):
                # platform/type/instance_id/version/tag
                components.update({
                    "platform": parts[0],
                    "type": parts[1],
                    "instance_id": parts[2],
                    "version": parts[3],
                    "tag": parts[4]
                })
            else:
                # platform/type/subtype/instance_id/version
                components.update({
                    "platform": parts[0],
                    "type": parts[1],
                    "subtype": parts[2],
                    "instance_id": parts[3],
                    "version": parts[4]
                })
        elif len(parts) == 6:
            # platform/scope/type/subtype/instance_id/version
            # Or: platform/type/subtype/instance_id/version/tag
            if cls._looks_like_scope(parts[1], parts[0]):
                # platform/scope/type/subtype/instance_id/version
                components.update({
                    "platform": parts[0],
                    "scope": parts[1],
                    "type": parts[2],
                    "subtype": parts[3],
                    "instance_id": parts[4],
                    "version": parts[5]
                })
            else:
                # platform/type/subtype/instance_id/version/tag
                components.update({
                    "platform": parts[0],
                    "type": parts[1],
                    "subtype": parts[2],
                    "instance_id": parts[3],
                    "version": parts[4],
                    "tag": parts[5]
                })
        elif len(parts) == 7:
            # platform/scope/type/subtype/instance_id/version/tag
            components.update({
                "platform": parts[0],
                "scope": parts[1],
                "type": parts[2],
                "subtype": parts[3],
                "instance_id": parts[4],
                "version": parts[5],
                "tag": parts[6]
            })
        else:
            raise TRNFormatError(f"Too many HTTP URL path components: {len(parts)} (maximum: 7)")

        return components

    @classmethod
    def _encode_component(cls, component: str) -> str:
        """Encode TRN component for URL."""
        if not component:
            return component
        
        # Apply URL encoding for special characters
        encoded = component
        for char, encoded_char in URL_ENCODING_MAP.items():
            encoded = encoded.replace(char, encoded_char)
        
        return encoded

    @classmethod
    def _decode_component(cls, component: str) -> str:
        """Decode URL component back to TRN format."""
        if not component:
            return component
        
        # Apply URL decoding
        return urllib.parse.unquote(component)

    @classmethod
    def _looks_like_scope(cls, value: str, platform: str) -> bool:
        """Heuristic to determine if a value looks like a scope."""
        # Reuse logic from parser
        from .parser import TRNParser
        return TRNParser._looks_like_scope(value, platform)

    @classmethod
    def _looks_like_version(cls, value: str) -> bool:
        """Heuristic to determine if a value looks like a version."""
        # Reuse logic from parser
        from .parser import TRNParser
        return TRNParser._looks_like_version(value)

    @classmethod
    @lru_cache(maxsize=100)
    def get_frontend_url(cls, trn: 'TRN', base_url: str = "https://your-platform.com", 
                        action: str = "view") -> str:
        """
        Get frontend URL for a TRN with specific action.
        
        Args:
            trn: TRN object
            base_url: Base URL of the frontend
            action: Action to perform (view, test, edit, docs, etc.)
            
        Returns:
            str: Frontend URL for the TRN
        """
        # Build base path
        path_parts = ["/tools", trn.platform]
        
        if trn.scope:
            path_parts.append(trn.scope)
        
        path_parts.extend([trn.type])
        
        if trn.subtype:
            path_parts.append(trn.subtype)
        
        path_parts.extend([trn.instance_id, trn.version])
        
        if trn.tag:
            path_parts.append(trn.tag)
        
        # Add action if not default
        if action != "view":
            path_parts.append(action)
        
        path = "/".join(cls._encode_component(part) for part in path_parts)
        url = f"{base_url.rstrip('/')}{path}"
        
        # Add hash as query parameter if present
        query_params = []
        if trn.hash:
            query_params.append(f"hash={urllib.parse.quote(trn.hash)}")
        
        if query_params:
            url += "?" + "&".join(query_params)
        
        return url

    @classmethod
    def validate_url(cls, url: str) -> bool:
        """
        Validate if URL is a valid TRN URL.
        
        Args:
            url: URL to validate
            
        Returns:
            bool: True if valid TRN URL
        """
        try:
            cls.from_url(url, validate=True)
            return True
        except:
            return False 