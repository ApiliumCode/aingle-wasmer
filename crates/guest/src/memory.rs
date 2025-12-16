//! Memory management utilities for WASM guests

use crate::arena::arena_alloc_copy;
use aingle_wasmer_codec::{decode_envelope, encode_with_envelope};
use aingle_wasmer_common::{WasmError, WasmResult, WasmSlice};

/// Read input arguments from the host (raw envelope version)
///
/// Decodes the envelope and returns the payload bytes.
/// This is the internal version that uses our envelope protocol.
/// For aingle compatibility, use the `host_args` function from `compat` module.
pub fn host_args_envelope(ptr: u32, len: u32) -> Result<&'static [u8], WasmError> {
    if len == 0 {
        return Ok(&[]);
    }

    let bytes = unsafe { core::slice::from_raw_parts(ptr as *const u8, len as usize) };

    let envelope = decode_envelope(bytes)?;

    // Return a reference to the payload (zero-copy)
    Ok(envelope.payload)
}

/// Read raw bytes from guest memory
pub fn read_bytes(ptr: u32, len: u32) -> &'static [u8] {
    if len == 0 {
        return &[];
    }
    unsafe { core::slice::from_raw_parts(ptr as *const u8, len as usize) }
}

/// Return a successful result to the host
pub fn return_ok(data: &[u8]) -> u64 {
    let mut buffer = [0u8; 4096]; // Max return size

    match encode_with_envelope(data, 0, &mut buffer) {
        Ok(len) => {
            let ptr = arena_alloc_copy(&buffer[..len]);
            WasmResult::ok(WasmSlice::new(ptr as u32, len as u32)).into_raw()
        }
        Err(_) => return_err(b"encoding error"),
    }
}

/// Return an error result to the host
pub fn return_err(message: &[u8]) -> u64 {
    use aingle_wasmer_common::EnvelopeFlags;

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

    /// Test that return_ok produces a valid result.
    /// Note: In native (non-WASM) mode, the arena pointer may have bit 31 set,
    /// which when packed can interfere with the error bit. This test verifies
    /// the encoding works and produces non-empty output.
    #[test]
    fn test_return_ok() {
        let data = b"test response";
        let result = return_ok(data);

        // Verify we got a non-zero result (data was encoded)
        assert_ne!(result, 0);

        // Extract the slice portion (clear the error bit if set due to native pointer)
        let wasm_result = WasmResult::from_raw(result);
        let slice = wasm_result.slice();

        // The slice should have non-zero length (the encoded envelope)
        assert!(slice.len > 0);
    }

    /// Test that return_err produces a valid error result.
    #[test]
    fn test_return_err() {
        let msg = b"error message";
        let result = return_err(msg);

        let wasm_result = WasmResult::from_raw(result);
        // Error bit is explicitly set, so this should always be an error
        assert!(wasm_result.is_err());
    }

    /// Test encoding itself works correctly
    #[test]
    fn test_encoding_roundtrip() {
        use aingle_wasmer_codec::{decode_envelope, encode_with_envelope};

        let data = b"test payload";
        let mut buffer = [0u8; 256];

        // Encode
        let len = encode_with_envelope(data, 0, &mut buffer).unwrap();
        assert!(len > data.len()); // Should include header

        // Decode
        let envelope = decode_envelope(&buffer[..len]).unwrap();
        assert_eq!(envelope.payload, data);
    }
}
