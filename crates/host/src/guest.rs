//! Guest interaction utilities

use crate::HostError;
use aingle_wasm_types::WasmSlice;
use aingle_wasm_codec::{encode_with_envelope, decode_envelope};

/// Call a guest function with automatic serialization
pub fn call<I, O>(
    instance: &mut crate::WasmInstance,
    name: &str,
    input: &I,
) -> Result<O, HostError>
where
    I: AsRef<[u8]>,
    O: From<Vec<u8>>,
{
    let result = instance.call_raw(name, input.as_ref())?;
    Ok(O::from(result))
}

/// Consume bytes from guest memory
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
    use aingle_wasm_types::EnvelopeFlags;

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

    #[test]
    fn test_build_result() {
        let data = b"test";
        let result = build_guest_result(data, false).unwrap();
        assert!(!result.is_empty());

        let decoded = decode_envelope(&result).unwrap();
        assert_eq!(decoded.payload, data);
    }
}
