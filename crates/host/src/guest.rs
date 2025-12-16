//! Guest interaction utilities
//!
//! Functions for calling guest WASM functions and transferring data.

use crate::HostError;
use aingle_wasmer_common::{WasmSlice, WasmResult};
use serde::{Serialize, de::DeserializeOwned};
use std::sync::Arc;

#[cfg(any(feature = "wasmer_sys_dev", feature = "wasmer_sys_prod"))]
use wasmer::{Instance, StoreMut, Value};

/// ExternIO compatible type for host-guest communication
///
/// This wraps serialized bytes and provides encode/decode methods
/// compatible with aingle's ExternIO.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
#[repr(transparent)]
pub struct ExternIO(#[serde(with = "serde_bytes")] pub Vec<u8>);

impl ExternIO {
    /// Create a new ExternIO from bytes
    pub fn new(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }

    /// Encode a value to ExternIO
    pub fn encode<T: Serialize>(value: T) -> Result<Self, HostError> {
        let bytes = rmp_serde::to_vec_named(&value)
            .map_err(|e| HostError::Serialization(format!("Failed to encode: {}", e)))?;
        Ok(Self(bytes))
    }

    /// Decode from ExternIO
    pub fn decode<T: DeserializeOwned>(&self) -> Result<T, HostError> {
        rmp_serde::from_slice(&self.0)
            .map_err(|e| HostError::Serialization(format!("Failed to decode: {}", e)))
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

impl AsRef<[u8]> for ExternIO {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl From<Vec<u8>> for ExternIO {
    fn from(v: Vec<u8>) -> Self {
        Self(v)
    }
}

impl From<ExternIO> for Vec<u8> {
    fn from(io: ExternIO) -> Self {
        io.0
    }
}

/// Call a guest function
///
/// This function:
/// 1. Allocates memory in the guest for the input
/// 2. Copies the input bytes to guest memory
/// 3. Calls the guest function
/// 4. Reads the result from guest memory
///
/// # Arguments
/// * `store` - Mutable reference to the Wasmer store
/// * `instance` - Arc reference to the WASM instance
/// * `name` - Name of the function to call
/// * `input` - Input data as ExternIO
///
/// # Returns
/// * `Ok(ExternIO)` - The result from the guest function
/// * `Err(wasmer::RuntimeError)` - If the call fails
#[cfg(any(feature = "wasmer_sys_dev", feature = "wasmer_sys_prod"))]
pub fn call(
    store: &mut StoreMut<'_>,
    instance: Arc<Instance>,
    name: &str,
    input: ExternIO,
) -> Result<ExternIO, wasmer::RuntimeError> {
    // Get the memory and allocate function from the instance
    let memory = instance.exports.get_memory("memory")
        .map_err(|e| wasmer::RuntimeError::new(format!("Failed to get memory: {}", e)))?;

    let allocate = instance.exports.get_typed_function::<i32, i32>(store, "__hc__allocate_1")
        .map_err(|e| wasmer::RuntimeError::new(format!("Failed to get allocate: {}", e)))?;

    let input_bytes = input.0;
    let input_len = input_bytes.len() as i32;

    // Allocate memory for input in guest
    let input_ptr = allocate.call(store, input_len)?;

    // Write input to guest memory
    let view = memory.view(store);
    view.write(input_ptr as u64, &input_bytes)
        .map_err(|e| wasmer::RuntimeError::new(format!("Failed to write input: {}", e)))?;

    // Get the target function
    let func = instance.exports.get_function(name)
        .map_err(|e| wasmer::RuntimeError::new(format!("Function '{}' not found: {}", name, e)))?;

    // Call the function
    let results = func.call(store, &[Value::I32(input_ptr), Value::I32(input_len)])?;

    // Parse the result (returns i64 containing pointer and length)
    let result_packed = results.first()
        .and_then(|v| v.i64())
        .ok_or_else(|| wasmer::RuntimeError::new("Invalid return type from guest"))?;

    let wasm_result = WasmResult::from_raw(result_packed as u64);
    let slice = wasm_result.slice();

    if slice.is_empty() {
        return Ok(ExternIO::new(Vec::new()));
    }

    // Read the result from guest memory
    let view = memory.view(store);
    let mut result_bytes = vec![0u8; slice.len as usize];
    view.read(slice.ptr as u64, &mut result_bytes)
        .map_err(|e| wasmer::RuntimeError::new(format!("Failed to read result: {}", e)))?;

    Ok(ExternIO::new(result_bytes))
}

/// Call a guest function with raw bytes (no ExternIO wrapper)
#[cfg(any(feature = "wasmer_sys_dev", feature = "wasmer_sys_prod"))]
pub fn call_raw(
    store: &mut StoreMut<'_>,
    instance: Arc<Instance>,
    name: &str,
    input: &[u8],
) -> Result<Vec<u8>, wasmer::RuntimeError> {
    let result = call(store, instance, name, ExternIO::new(input.to_vec()))?;
    Ok(result.into_vec())
}

/// Consume bytes from guest memory
///
/// This is a helper function that reads bytes directly from guest memory.
pub fn consume_bytes_from_guest(
    memory: &[u8],
    ptr: u32,
    len: u32,
) -> Result<Vec<u8>, HostError> {
    let start = ptr as usize;
    let end = start + len as usize;

    if end > memory.len() {
        return Err(HostError::MemoryAccess(format!(
            "out of bounds: {}..{} > {}",
            start, end, memory.len()
        )));
    }

    Ok(memory[start..end].to_vec())
}

/// Move data to guest memory
///
/// This is a helper function that writes bytes to a memory buffer.
pub fn move_data_to_guest(
    memory: &mut [u8],
    ptr: u32,
    data: &[u8],
) -> Result<WasmSlice, HostError> {
    let start = ptr as usize;
    let end = start + data.len();

    if end > memory.len() {
        return Err(HostError::MemoryAccess(format!(
            "out of bounds: {}..{} > {}",
            start, end, memory.len()
        )));
    }

    memory[start..end].copy_from_slice(data);
    Ok(WasmSlice::new(ptr, data.len() as u32))
}

/// Build a result for returning to guest
pub fn build_guest_result(
    data: &[u8],
    is_error: bool,
) -> Result<Vec<u8>, HostError> {
    use aingle_wasmer_codec::encode_with_envelope;
    use aingle_wasmer_common::EnvelopeFlags;

    let flags = if is_error {
        EnvelopeFlags::IsError as u8
    } else {
        0
    };

    let mut buffer = vec![0u8; data.len() + 64];
    let len = encode_with_envelope(data, flags, &mut buffer)
        .map_err(|e| HostError::Serialization(format!("{:?}", e)))?;

    buffer.truncate(len);
    Ok(buffer)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extern_io_roundtrip() {
        #[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
        struct TestData {
            value: i32,
            name: String,
        }

        let original = TestData {
            value: 42,
            name: "test".to_string(),
        };

        let io = ExternIO::encode(&original).unwrap();
        let decoded: TestData = io.decode().unwrap();

        assert_eq!(original, decoded);
    }

    #[test]
    fn test_consume_bytes() {
        let memory = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        let bytes = consume_bytes_from_guest(&memory, 2, 4).unwrap();
        assert_eq!(bytes, vec![2, 3, 4, 5]);
    }

    #[test]
    fn test_move_data() {
        let mut memory = vec![0u8; 100];
        let data = b"hello";

        let slice = move_data_to_guest(&mut memory, 10, data).unwrap();
        assert_eq!(slice.ptr, 10);
        assert_eq!(slice.len, 5);
        assert_eq!(&memory[10..15], data);
    }
}
