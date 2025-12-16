//! Error types for AIngle WASM runtime

use core::fmt;

/// Primary error type for WASM operations
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WasmError {
    /// Error during serialization
    Serialize(SerializeError),
    /// Error during deserialization
    Deserialize(DeserializeError),
    /// Memory allocation failed
    Memory(MemoryError),
    /// Host function call failed
    HostCall(HostCallError),
    /// Guest function call failed
    GuestCall(GuestCallError),
    /// Generic error with message
    Guest(WasmErrorInner),
}

/// Inner error with optional context
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WasmErrorInner {
    /// Error kind
    pub kind: ErrorKind,
    /// Optional file location
    pub file: Option<&'static str>,
    /// Optional line number
    pub line: Option<u32>,
    /// Error message (stored inline for no_std)
    message: ErrorMessage,
}

/// Fixed-size error message for no_std compatibility
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ErrorMessage {
    bytes: [u8; 128],
    len: u8,
}

impl ErrorMessage {
    /// Create from a static string
    pub const fn from_static(s: &'static str) -> Self {
        let bytes_slice = s.as_bytes();
        let len = if bytes_slice.len() > 128 { 128 } else { bytes_slice.len() };

        let mut bytes = [0u8; 128];
        let mut i = 0;
        while i < len {
            bytes[i] = bytes_slice[i];
            i += 1;
        }

        Self { bytes, len: len as u8 }
    }

    /// Get the message as a string slice
    pub fn as_str(&self) -> &str {
        // Safety: we only store valid UTF-8
        unsafe { core::str::from_utf8_unchecked(&self.bytes[..self.len as usize]) }
    }
}

impl WasmErrorInner {
    /// Create a new error with message
    pub const fn new(kind: ErrorKind, message: &'static str) -> Self {
        Self {
            kind,
            file: None,
            line: None,
            message: ErrorMessage::from_static(message),
        }
    }

    /// Add file location
    pub const fn with_location(mut self, file: &'static str, line: u32) -> Self {
        self.file = Some(file);
        self.line = Some(line);
        self
    }
}

/// Categories of errors
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum ErrorKind {
    /// Unknown error
    Unknown = 0,
    /// Serialization error
    Serialization = 1,
    /// Deserialization error
    Deserialization = 2,
    /// Memory error
    Memory = 3,
    /// Host call error
    HostCall = 4,
    /// Guest call error
    GuestCall = 5,
    /// Validation error
    Validation = 6,
    /// Timeout error
    Timeout = 7,
    /// Permission denied
    PermissionDenied = 8,
}

/// Serialization errors
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SerializeError {
    /// Buffer too small for serialization
    BufferTooSmall { needed: usize, available: usize },
    /// Type cannot be serialized
    UnsupportedType,
    /// Nesting too deep
    NestingTooDeep,
}

/// Deserialization errors
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeserializeError {
    /// Unexpected end of input
    UnexpectedEof,
    /// Invalid data format
    InvalidFormat,
    /// Type mismatch
    TypeMismatch,
    /// Unknown variant
    UnknownVariant(u32),
}

/// Memory errors
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MemoryError {
    /// Allocation failed
    AllocationFailed { requested: usize },
    /// Out of bounds access
    OutOfBounds { offset: usize, len: usize, max: usize },
    /// Alignment error
    Alignment { addr: usize, required: usize },
    /// Arena exhausted
    ArenaExhausted,
}

/// Host call errors
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HostCallError {
    /// Function not found
    FunctionNotFound,
    /// Invalid arguments
    InvalidArguments,
    /// Host returned error
    HostError(u32),
    /// Call timed out
    Timeout,
}

/// Guest call errors
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GuestCallError {
    /// Function not exported
    FunctionNotExported,
    /// Invalid return value
    InvalidReturn,
    /// Guest panicked
    Panic,
    /// Metering limit exceeded
    MeteringExceeded,
}

impl fmt::Display for WasmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WasmError::Serialize(e) => write!(f, "serialization error: {:?}", e),
            WasmError::Deserialize(e) => write!(f, "deserialization error: {:?}", e),
            WasmError::Memory(e) => write!(f, "memory error: {:?}", e),
            WasmError::HostCall(e) => write!(f, "host call error: {:?}", e),
            WasmError::GuestCall(e) => write!(f, "guest call error: {:?}", e),
            WasmError::Guest(inner) => {
                write!(f, "[{:?}] {}", inner.kind, inner.message.as_str())?;
                if let (Some(file), Some(line)) = (inner.file, inner.line) {
                    write!(f, " at {}:{}", file, line)?;
                }
                Ok(())
            }
        }
    }
}

/// Convenience macro for creating errors with location
#[macro_export]
macro_rules! wasm_error {
    ($kind:expr, $msg:literal) => {
        $crate::WasmError::Guest($crate::WasmErrorInner::new($kind, $msg)
            .with_location(file!(), line!()))
    };
    ($msg:literal) => {
        $crate::wasm_error!($crate::ErrorKind::Unknown, $msg)
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_message() {
        let msg = ErrorMessage::from_static("test error");
        assert_eq!(msg.as_str(), "test error");
    }

    #[test]
    fn test_wasm_error_inner() {
        let err = WasmErrorInner::new(ErrorKind::Validation, "invalid input")
            .with_location("test.rs", 42);

        assert_eq!(err.kind, ErrorKind::Validation);
        assert_eq!(err.file, Some("test.rs"));
        assert_eq!(err.line, Some(42));
    }
}
