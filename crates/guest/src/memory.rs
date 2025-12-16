//! Memory management utilities for WASM guests

use aingle_wasm_types::{WasmSlice, WasmResult, WasmError};
use aingle_wasm_codec::{encode_with_envelope, decode_envelope};
use crate::arena::arena_alloc_copy;

/// Read input arguments from the host
///
/// Decodes the envelope and returns the payload bytes.
pub fn host_args(ptr: u32, len: u32) -> Result<&'static [u8], WasmError> {
    if len == 0 {
        return Ok(&[]);
    }

    let bytes = unsafe {
        core::slice::from_raw_parts(ptr as *const u8, len as usize)
    };

    let envelope = decode_envelope(bytes)?;

    // Return a reference to the payload (zero-copy)
    Ok(envelope.payload)
}

/// Read raw bytes from guest memory
pub fn read_bytes(ptr: u32, len: u32) -> &'static [u8] {
    if len == 0 {
        return &[];
    }
    unsafe {
        core::slice::from_raw_parts(ptr as *const u8, len as usize)
    }
}

/// Return a successful result to the host
pub fn return_ok(data: &[u8]) -> u64 {
    let mut buffer = [0u8; 4096]; // Max return size

    match encode_with_envelope(data, 0, &mut buffer) {
        Ok(len) => {
            let ptr = arena_alloc_copy(&buffer[..len]);
            WasmResult::ok(WasmSlice::new(ptr as u32, len as u32)).into_raw()
        }
        Err(_) => {
            return_err(b"encoding error")
        }
    }
}

/// Return an error result to the host
pub fn return_err(message: &[u8]) -> u64 {
    use aingle_wasm_types::EnvelopeFlags;

    let mut buffer = [0u8; 256];
    let flags = EnvelopeFlags::IsError as u8;

    match encode_with_envelope(message, flags, &mut buffer) {
        Ok(len) => {
            let ptr = arena_alloc_copy(&buffer[..len]);
            WasmResult::err(WasmSlice::new(ptr as u32, len as u32)).into_raw()
        }
        Err(_) => {
            // Last resort: return empty error
            WasmResult::err(WasmSlice::empty()).into_raw()
        }
    }
}

/// Try macro for guest functions - returns error to host on failure
#[macro_export]
macro_rules! try_result {
    ($expr:expr) => {
        match $expr {
            Ok(val) => val,
            Err(e) => {
                let msg = format!("{:?}", e);
                return $crate::return_err(msg.as_bytes());
            }
        }
    };
    ($expr:expr, $msg:literal) => {
        match $expr {
            Ok(val) => val,
            Err(_) => {
                return $crate::return_err($msg.as_bytes());
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_return_ok() {
        let data = b"test response";
        let result = return_ok(data);

        let wasm_result = WasmResult::from_raw(result);
        assert!(wasm_result.is_ok());
        assert!(!wasm_result.slice().is_empty());
    }

    #[test]
    fn test_return_err() {
        let msg = b"error message";
        let result = return_err(msg);

        let wasm_result = WasmResult::from_raw(result);
        assert!(wasm_result.is_err());
    }
}
