//! Compatibility layer for aingle integration
//!
//! Provides the API expected by the ADK:
//! - `host_args` - Read input arguments from the host
//! - `return_ptr` - Return a serialized success value
//! - `return_err_ptr` - Return a serialized error
//! - `host_call` - Call a host function with typed serialization

use crate::arena::arena_alloc_copy;
use aingle_wasmer_common::{
    DeserializeError, DoubleUSize, ErrorKind, HostCallError, SerializeError, WasmError,
    WasmErrorInner, WasmResult, WasmSlice,
};
use serde::{de::DeserializeOwned, Serialize};

/// Guest pointer type (memory offset)
pub type GuestPtr = u32;

/// Length type
pub type Len = u32;

/// Wrapper for serialized bytes (compatible with ExternIO)
#[derive(Clone, Debug, PartialEq, Eq, Serialize, serde::Deserialize)]
#[serde(transparent)]
#[repr(transparent)]
pub struct SerializedBytes(#[serde(with = "serde_bytes")] pub Vec<u8>);

impl SerializedBytes {
    /// Create from raw bytes
    pub fn new(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }

    /// Encode a value to serialized bytes
    pub fn encode<T: Serialize>(value: &T) -> Result<Self, WasmError> {
        let bytes = rmp_serde::to_vec_named(value)
            .map_err(|_| WasmError::Serialize(SerializeError::UnsupportedType))?;
        Ok(Self(bytes))
    }

    /// Decode from serialized bytes
    pub fn decode<T: DeserializeOwned>(&self) -> Result<T, WasmError> {
        rmp_serde::from_slice(&self.0)
            .map_err(|_| WasmError::Deserialize(DeserializeError::InvalidFormat))
    }

    /// Get inner bytes
    pub fn into_vec(self) -> Vec<u8> {
        self.0
    }

    /// Get bytes as slice
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl AsRef<[u8]> for SerializedBytes {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl From<Vec<u8>> for SerializedBytes {
    fn from(v: Vec<u8>) -> Self {
        Self(v)
    }
}

impl From<SerializedBytes> for Vec<u8> {
    fn from(sb: SerializedBytes) -> Self {
        sb.0
    }
}

/// Read input arguments from the host
///
/// This function reads bytes from guest memory at the given pointer/length
/// and returns them as a Vec<u8>. The caller should wrap this in ExternIO or
/// another container type for decoding.
///
/// # Arguments
/// * `guest_ptr` - Pointer to the start of the input data
/// * `len` - Length of the input data in bytes
///
/// # Returns
/// * `Ok(Vec<u8>)` - The raw input bytes
/// * `Err(DoubleUSize)` - Error pointer if reading fails
pub fn host_args(guest_ptr: GuestPtr, len: Len) -> Result<Vec<u8>, DoubleUSize> {
    if len == 0 {
        return Ok(Vec::new());
    }

    let bytes = unsafe { core::slice::from_raw_parts(guest_ptr as *const u8, len as usize) };

    Ok(bytes.to_vec())
}

/// Return a serialized success value to the host
///
/// Serializes the value and copies it to the arena, returning a pointer
/// that the host can use to read the result.
///
/// # Type Parameters
/// * `T` - The type to serialize (must implement Serialize)
///
/// # Arguments
/// * `value` - The value to return
///
/// # Returns
/// A DoubleUSize encoding the pointer and length
pub fn return_ptr<T: Serialize>(value: T) -> DoubleUSize {
    match SerializedBytes::encode(&value) {
        Ok(sb) => {
            let bytes = sb.0;
            let len = bytes.len() as u32;
            let ptr = arena_alloc_copy(&bytes) as u32;
            WasmResult::ok(WasmSlice::new(ptr, len)).into_raw()
        }
        Err(_) => {
            // Return empty error on serialization failure
            WasmResult::err(WasmSlice::empty()).into_raw()
        }
    }
}

/// Return a serialized error to the host
///
/// Converts the WasmError to a serializable format and copies it to the arena,
/// returning an error pointer that the host can use to read the error.
///
/// # Arguments
/// * `error` - The WasmError to return
///
/// # Returns
/// A DoubleUSize encoding the error pointer and length
pub fn return_err_ptr(error: WasmError) -> DoubleUSize {
    // Convert WasmError to a serializable error struct
    #[derive(Serialize)]
    struct SerializableError {
        error_type: String,
        message: String,
    }

    let serializable = SerializableError {
        error_type: format!("{:?}", core::mem::discriminant(&error)),
        message: format!("{}", error),
    };

    match SerializedBytes::encode(&serializable) {
        Ok(sb) => {
            let bytes = sb.0;
            let len = bytes.len() as u32;
            let ptr = arena_alloc_copy(&bytes) as u32;
            WasmResult::err(WasmSlice::new(ptr, len)).into_raw()
        }
        Err(_) => {
            // Return empty error if we can't serialize
            WasmResult::err(WasmSlice::empty()).into_raw()
        }
    }
}

/// Call a host function with typed serialization
///
/// This function:
/// 1. Serializes the input using MessagePack
/// 2. Copies it to the arena
/// 3. Calls the host function
/// 4. Deserializes and returns the result
///
/// # Type Parameters
/// * `I` - Input type (must implement Serialize)
/// * `O` - Output type (must implement DeserializeOwned)
///
/// # Arguments
/// * `host_fn` - The extern host function to call
/// * `input` - The input value to serialize and send
///
/// # Returns
/// * `Ok(O)` - The deserialized output
/// * `Err(WasmError)` - If serialization, the call, or deserialization fails
pub fn host_call<I, O>(
    host_fn: unsafe extern "C" fn(GuestPtr, Len) -> u64,
    input: I,
) -> Result<O, WasmError>
where
    I: Serialize,
    O: DeserializeOwned,
{
    // Serialize input
    let input_bytes = SerializedBytes::encode(&input)?;
    let bytes = input_bytes.0;
    let len = bytes.len() as u32;

    // Copy to arena for host access
    let ptr = arena_alloc_copy(&bytes) as u32;

    // Call the host
    let result = unsafe { host_fn(ptr, len) };

    // Parse result
    let wasm_result = WasmResult::from_raw(result);
    let slice = wasm_result.slice();

    if wasm_result.is_err() {
        // Return host call error - we can't deserialize WasmError directly
        return Err(WasmError::HostCall(HostCallError::HostError(0)));
    }

    // Deserialize success response
    if slice.is_empty() {
        // Try to decode empty/unit type
        return rmp_serde::from_slice(&[])
            .map_err(|_| WasmError::Deserialize(DeserializeError::InvalidFormat));
    }

    let response_bytes =
        unsafe { core::slice::from_raw_parts(slice.ptr as *const u8, slice.len as usize) };

    rmp_serde::from_slice(response_bytes)
        .map_err(|_| WasmError::Deserialize(DeserializeError::InvalidFormat))
}

// Note: host_externs! macro is defined in host_call.rs

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialized_bytes_roundtrip() {
        #[derive(Debug, PartialEq, Serialize, serde::Deserialize)]
        struct TestData {
            value: u32,
            name: String,
        }

        let original = TestData {
            value: 42,
            name: "test".to_string(),
        };

        let sb = SerializedBytes::encode(&original).unwrap();
        let decoded: TestData = sb.decode().unwrap();

        assert_eq!(original, decoded);
    }

    #[test]
    fn test_host_args_empty() {
        let result = host_args(0, 0).unwrap();
        assert!(result.is_empty());
    }
}
