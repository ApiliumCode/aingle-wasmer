//! Host-side error types

use thiserror::Error;

/// Errors that can occur on the host side
#[derive(Error, Debug)]
pub enum HostError {
    /// WASM compilation failed
    #[error("compilation error: {0}")]
    Compilation(String),

    /// Instance creation failed
    #[error("instantiation error: {0}")]
    Instantiation(String),

    /// Function not found in module
    #[error("function not found: {0}")]
    FunctionNotFound(String),

    /// Memory export not found
    #[error("memory not found in exports")]
    MemoryNotFound,

    /// Memory access error
    #[error("memory access error: {0}")]
    MemoryAccess(String),

    /// Runtime error during execution
    #[error("runtime error: {0}")]
    Runtime(String),

    /// Invalid return value from guest
    #[error("invalid return value from guest")]
    InvalidReturn,

    /// Guest returned an error
    #[error("guest error: {0}")]
    GuestError(String),

    /// Serialization error
    #[error("serialization error: {0}")]
    Serialization(String),

    /// Deserialization error
    #[error("deserialization error: {0}")]
    Deserialization(String),

    /// Metering limit exceeded
    #[error("metering limit exceeded")]
    MeteringExceeded,

    /// Cache error
    #[error("cache error: {0}")]
    Cache(String),
}

impl From<HostError> for aingle_wasmer_common::WasmError {
    fn from(err: HostError) -> Self {
        use aingle_wasmer_common::{GuestCallError, HostCallError};

        match err {
            HostError::FunctionNotFound(_) => {
                aingle_wasmer_common::WasmError::GuestCall(GuestCallError::FunctionNotExported)
            }
            HostError::InvalidReturn => {
                aingle_wasmer_common::WasmError::GuestCall(GuestCallError::InvalidReturn)
            }
            HostError::MeteringExceeded => {
                aingle_wasmer_common::WasmError::GuestCall(GuestCallError::MeteringExceeded)
            }
            HostError::GuestError(_) => {
                aingle_wasmer_common::WasmError::GuestCall(GuestCallError::Panic)
            }
            _ => {
                aingle_wasmer_common::WasmError::HostCall(HostCallError::HostError(0))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = HostError::FunctionNotFound("test_fn".to_string());
        assert!(err.to_string().contains("test_fn"));
    }
}
