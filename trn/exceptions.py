"""
TRN Exception Classes

Defines all exception types used throughout the TRN library.
Provides clear error hierarchy for different types of TRN-related errors.
"""

class TRNError(Exception):
    """Base exception for all TRN-related errors."""
    
    def __init__(self, message: str, trn: str = None, code: int = None):
        super().__init__(message)
        self.message = message
        self.trn = trn
        self.code = code
    
    def __str__(self):
        if self.trn:
            return f"TRN Error: {self.message} (TRN: {self.trn})"
        return f"TRN Error: {self.message}"


class TRNFormatError(TRNError):
    """Raised when TRN format is invalid."""
    
    def __init__(self, message: str, trn: str = None, component: str = None):
        super().__init__(message, trn, code=-32000)
        self.component = component
    
    def __str__(self):
        if self.component:
            return f"TRN Format Error in '{self.component}': {self.message}"
        return f"TRN Format Error: {self.message}"


class TRNValidationError(TRNError):
    """Raised when TRN validation fails."""
    
    def __init__(self, message: str, trn: str = None, rule: str = None):
        super().__init__(message, trn, code=-32001)
        self.rule = rule
    
    def __str__(self):
        if self.rule:
            return f"TRN Validation Error ({self.rule}): {self.message}"
        return f"TRN Validation Error: {self.message}"


class TRNComponentError(TRNError):
    """Raised when TRN component is missing or invalid."""
    
    def __init__(self, message: str, component: str, trn: str = None):
        super().__init__(message, trn, code=-32001)
        self.component = component
    
    def __str__(self):
        return f"TRN Component Error '{self.component}': {self.message}"


class TRNHashError(TRNError):
    """Raised when TRN hash verification fails."""
    
    def __init__(self, message: str, trn: str = None, expected_hash: str = None, actual_hash: str = None):
        super().__init__(message, trn, code=-32002)
        self.expected_hash = expected_hash
        self.actual_hash = actual_hash
    
    def __str__(self):
        if self.expected_hash and self.actual_hash:
            return f"TRN Hash Error: {self.message} (expected: {self.expected_hash}, got: {self.actual_hash})"
        return f"TRN Hash Error: {self.message}"


class TRNAliasError(TRNError):
    """Raised when TRN alias resolution fails."""
    
    def __init__(self, message: str, alias: str = None, trn: str = None):
        super().__init__(message, trn, code=-32003)
        self.alias = alias
    
    def __str__(self):
        if self.alias:
            return f"TRN Alias Error '{self.alias}': {self.message}"
        return f"TRN Alias Error: {self.message}"


class TRNLengthError(TRNValidationError):
    """Raised when TRN length exceeds limits."""
    
    def __init__(self, message: str, trn: str = None, length: int = None, max_length: int = 256):
        super().__init__(message, trn, rule="length_limit")
        self.length = length
        self.max_length = max_length
    
    def __str__(self):
        if self.length and self.max_length:
            return f"TRN Length Error: {self.message} ({self.length}/{self.max_length} chars)"
        return f"TRN Length Error: {self.message}"


class TRNCharacterError(TRNValidationError):
    """Raised when TRN contains invalid characters."""
    
    def __init__(self, message: str, trn: str = None, invalid_char: str = None, position: int = None):
        super().__init__(message, trn, rule="character_set")
        self.invalid_char = invalid_char
        self.position = position
    
    def __str__(self):
        if self.invalid_char and self.position is not None:
            return f"TRN Character Error: Invalid character '{self.invalid_char}' at position {self.position}"
        return f"TRN Character Error: {self.message}"


class TRNReservedWordError(TRNValidationError):
    """Raised when TRN uses reserved words."""
    
    def __init__(self, message: str, trn: str = None, reserved_word: str = None, component: str = None):
        super().__init__(message, trn, rule="reserved_words")
        self.reserved_word = reserved_word
        self.component = component
    
    def __str__(self):
        if self.reserved_word and self.component:
            return f"TRN Reserved Word Error: '{self.reserved_word}' is reserved in component '{self.component}'"
        return f"TRN Reserved Word Error: {self.message}"


class TRNPermissionError(TRNError):
    """Raised when TRN access permission is denied."""
    
    def __init__(self, message: str, trn: str = None, user: str = None, action: str = None):
        super().__init__(message, trn, code=-32020)
        self.user = user
        self.action = action
    
    def __str__(self):
        if self.user and self.action:
            return f"TRN Permission Error: User '{self.user}' denied '{self.action}' on {self.trn}"
        return f"TRN Permission Error: {self.message}"


class TRNNotFoundError(TRNError):
    """Raised when TRN resource is not found."""
    
    def __init__(self, message: str, trn: str = None):
        super().__init__(message, trn, code=-32030)
    
    def __str__(self):
        return f"TRN Not Found Error: {self.message}"


class TRNConflictError(TRNError):
    """Raised when TRN conflicts with existing resource."""
    
    def __init__(self, message: str, trn: str = None, existing_trn: str = None):
        super().__init__(message, trn, code=-32031)
        self.existing_trn = existing_trn
    
    def __str__(self):
        if self.existing_trn:
            return f"TRN Conflict Error: {self.message} (conflicts with: {self.existing_trn})"
        return f"TRN Conflict Error: {self.message}"


class TRNDeprecationWarning(UserWarning):
    """Warning for deprecated TRN features or formats."""
    
    def __init__(self, message: str, trn: str = None, replacement: str = None):
        super().__init__(message)
        self.message = message
        self.trn = trn
        self.replacement = replacement
    
    def __str__(self):
        if self.replacement:
            return f"TRN Deprecation Warning: {self.message} (use: {self.replacement})"
        return f"TRN Deprecation Warning: {self.message}"


# Error code mapping for JSON RPC compatibility
ERROR_CODE_MAP = {
    TRNFormatError: -32000,
    TRNValidationError: -32001,
    TRNComponentError: -32001,
    TRNLengthError: -32002,
    TRNCharacterError: -32002,
    TRNReservedWordError: -32002,
    TRNHashError: -32003,
    TRNAliasError: -32003,
    TRNPermissionError: -32020,
    TRNNotFoundError: -32030,
    TRNConflictError: -32031,
}


def get_error_code(exception: Exception) -> int:
    """Get JSON RPC error code for TRN exception."""
    return ERROR_CODE_MAP.get(type(exception), -32000)


def format_error_response(exception: TRNError) -> dict:
    """Format TRN exception as JSON RPC error response."""
    return {
        "code": exception.code or get_error_code(exception),
        "message": str(exception),
        "data": {
            "type": exception.__class__.__name__,
            "trn": exception.trn,
            "details": exception.message
        }
    } 