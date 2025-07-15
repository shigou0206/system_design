"""
TRN (Tool Resource Name) Python Library

A comprehensive library for working with Tool Resource Names in AI Agent platforms.
Provides parsing, validation, URL conversion, alias management, and more.

Example:
    >>> from trn import TRN
    >>> trn = TRN.parse("trn:user:alice:tool:openapi:github-api:v1.0")
    >>> print(trn.platform)  # "user"
    >>> print(trn.instance_id)  # "github-api"
"""

from .core import TRN, TRNComponents
from .exceptions import (
    TRNError,
    TRNFormatError,
    TRNValidationError,
    TRNComponentError,
    TRNHashError,
    TRNAliasError
)
from .validator import TRNValidator
from .parser import TRNParser
from .url_converter import TRNURLConverter
from .alias_manager import TRNAliasManager
from .hash_verifier import TRNHashVerifier
from .utils import (
    normalize_trn,
    generate_trn,
    extract_base_trn,
    is_valid_trn,
    compare_trn_versions
)

__version__ = "1.0.0"
__author__ = "AI Platform Team"
__email__ = "team@aiplatform.com"

# Convenience functions for common operations
def parse(trn_string: str) -> TRN:
    """Parse a TRN string into a TRN object."""
    return TRN.parse(trn_string)

def validate(trn_string: str) -> bool:
    """Validate a TRN string format."""
    return TRNValidator.validate(trn_string)

def from_url(url: str) -> TRN:
    """Convert a TRN URL to TRN object."""
    return TRNURLConverter.from_url(url)

def to_url(trn: TRN) -> str:
    """Convert a TRN object to URL format."""
    return TRNURLConverter.to_url(trn)

__all__ = [
    # Core classes
    'TRN',
    'TRNComponents',
    
    # Exceptions
    'TRNError',
    'TRNFormatError', 
    'TRNValidationError',
    'TRNComponentError',
    'TRNHashError',
    'TRNAliasError',
    
    # Service classes
    'TRNValidator',
    'TRNParser',
    'TRNURLConverter',
    'TRNAliasManager',
    'TRNHashVerifier',
    
    # Utility functions
    'normalize_trn',
    'generate_trn',
    'extract_base_trn',
    'is_valid_trn',
    'compare_trn_versions',
    
    # Convenience functions
    'parse',
    'validate',
    'from_url',
    'to_url',
] 