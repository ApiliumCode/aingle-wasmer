//! Error types for AIngle WASM runtime

use alloc::string::{String, ToString};
use core::fmt;
use serde::{Deserialize, Serialize};

/// Primary error type for WASM operations
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
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
    /// Generic guest error with string message (for aingle compatibility)
    Guest(String),
    /// Generic host error with string message (for aingle compatibility)
    Host(String),
    /// Structured guest error with location info
    GuestStructured(WasmErrorInner),
}

impl WasmError {
    /// Create a new guest error from a string
    pub fn guest<S: Into<String>>(msg: S) -> Self {
        WasmError::Guest(msg.into())
    }
}

impl From<String> for WasmError {
    fn from(s: String) -> Self {
        WasmError::Guest(s)
    }
}

impl From<&str> for WasmError {
    fn from(s: &str) -> Self {
        WasmError::Guest(s.to_string())
    }
}

impl From<core::convert::Infallible> for WasmError {
    fn from(_: core::convert::Infallible) -> Self {
        // Infallible can never be instantiated, so this is unreachable
        unreachable!()
    }
}

/// Convert SerializedBytesError to WasmError when middleware_bytes feature is enabled
#[cfg(feature = "middleware_bytes")]
impl From<aingle_middleware_bytes::SerializedBytesError> for WasmError {
    fn from(e: aingle_middleware_bytes::SerializedBytesError) -> Self {
        WasmError::Guest(alloc::format!("SerializedBytesError: {}", e))
    }
}

/// Inner error with optional context
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WasmErrorInner {
    /// Error kind
    pub kind: ErrorKind,
    /// Optional file location
    pub file: Option<String>,
    /// Optional line number
    pub line: Option<u32>,
    /// Error message
    message: String,
}

impl WasmErrorInner {
    /// Create a new error with message
    pub fn new(kind: ErrorKind, message: &str) -> Self {
        Self {
            kind,
            file: None,
            line: None,
            message: message.to_string(),
        }
    }

    /// Create a new error with message (const-compatible, truncates at 128 chars)
    pub const fn new_const(kind: ErrorKind, message: &'static str) -> WasmErrorInnerBuilder {
        WasmErrorInnerBuilder { kind, message }
    }

    /// Get the message as a string slice
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Add file location
    pub fn with_location(mut self, file: &str, line: u32) -> Self {
        self.file = Some(file.to_string());
        self.line = Some(line);
        self
    }
}

/// Builder for const context - converts to WasmErrorInner at runtime
pub struct WasmErrorInnerBuilder {
    kind: ErrorKind,
    message: &'static str,
}

impl WasmErrorInnerBuilder {
    /// Add file location
    pub const fn with_location(self, _file: &'static str, _line: u32) -> WasmErrorInnerBuilderWithLocation {
        WasmErrorInnerBuilderWithLocation {
            kind: self.kind,
            message: self.message,
            file: _file,
            line: _line,
        }
    }

    /// Build into WasmErrorInner
    pub fn build(self) -> WasmErrorInner {
        WasmErrorInner {
            kind: self.kind,
            file: None,
            line: None,
            message: self.message.to_string(),
        }
    }
}

/// Builder with location info
pub struct WasmErrorInnerBuilderWithLocation {
    kind: ErrorKind,
    message: &'static str,
    file: &'static str,
    line: u32,
}

impl WasmErrorInnerBuilderWithLocation {
    /// Build into WasmErrorInner
    pub fn build(self) -> WasmErrorInner {
        WasmErrorInner {
            kind: self.kind,
            file: Some(self.file.to_string()),
            line: Some(self.line),
            message: self.message.to_string(),
        }
    }
}

impl From<WasmErrorInnerBuilder> for WasmErrorInner {
    fn from(b: WasmErrorInnerBuilder) -> Self {
        b.build()
    }
}

impl From<WasmErrorInnerBuilderWithLocation> for WasmErrorInner {
    fn from(b: WasmErrorInnerBuilderWithLocation) -> Self {
        b.build()
    }
}

/// Categories of errors
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
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
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SerializeError {
    /// Buffer too small for serialization
    BufferTooSmall { needed: usize, available: usize },
    /// Type cannot be serialized
    UnsupportedType,
    /// Nesting too deep
    NestingTooDeep,
}

/// Deserialization errors
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
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
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemoryError {
    /// Allocation failed
    AllocationFailed { requested: usize },
    /// Out of bounds access
    OutOfBounds {
        offset: usize,
        len: usize,
        max: usize,
    },
    /// Alignment error
    Alignment { addr: usize, required: usize },
    /// Arena exhausted
    ArenaExhausted,
}

/// Host call errors
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
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
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
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
            WasmError::Guest(msg) => write!(f, "guest error: {}", msg),
            WasmError::Host(msg) => write!(f, "host error: {}", msg),
            WasmError::GuestStructured(inner) => {
                write!(f, "[{:?}] {}", inner.kind, inner.message())?;
                if let (Some(ref file), Some(line)) = (&inner.file, inner.line) {
                    write!(f, " at {}:{}", file, line)?;
                }
                Ok(())
            }
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for WasmError {}

/// Convenience macro for creating errors with location
#[macro_export]
macro_rules! wasm_error {
    ($kind:expr, $msg:literal) => {
        $crate::WasmError::GuestStructured(
            $crate::WasmErrorInner::new($kind, $msg).with_location(file!(), line!()),
        )
    };
    ($msg:literal) => {
        $crate::wasm_error!($crate::ErrorKind::Unknown, $msg)
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wasm_error_inner() {
        let err = WasmErrorInner::new(ErrorKind::Validation, "invalid input")
            .with_location("test.rs", 42);

        assert_eq!(err.kind, ErrorKind::Validation);
        assert_eq!(err.file, Some("test.rs".to_string()));
        assert_eq!(err.line, Some(42));
        assert_eq!(err.message(), "invalid input");
    }

    #[test]
    fn test_wasm_error_display() {
        let err = WasmError::Guest("test error".to_string());
        assert_eq!(format!("{}", err), "guest error: test error");

        let err = WasmError::Host("host error".to_string());
        assert_eq!(format!("{}", err), "host error: host error");
    }

    #[test]
    fn test_wasm_error_from_string() {
        let err: WasmError = "test".into();
        assert!(matches!(err, WasmError::Guest(_)));
    }
}
