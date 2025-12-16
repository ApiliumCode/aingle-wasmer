//! Host function calling utilities

use crate::arena::arena_alloc_copy;
use aingle_wasmer_codec::{decode_envelope, encode_with_envelope};
use aingle_wasmer_common::{HostCallError, WasmError, WasmResult};

/// Call a host function with encoded arguments
///
/// # Arguments
/// * `host_fn` - The extern host function to call
/// * `args` - The serialized arguments
///
/// # Returns
/// The response payload from the host
pub fn host_call_raw(
    host_fn: unsafe extern "C" fn(u32, u32) -> u64,
    args: &[u8],
) -> Result<&'static [u8], WasmError> {
    // Encode args with envelope
    let mut buffer = [0u8; 4096];
    let len = encode_with_envelope(args, 0, &mut buffer)?;

    // Copy to arena so host can read
    let ptr = arena_alloc_copy(&buffer[..len]);

    // Call the host
    let result = unsafe { host_fn(ptr as u32, len as u32) };

    // Parse result
    let wasm_result = WasmResult::from_raw(result);
    let slice = wasm_result.slice();

    if slice.is_empty() {
        if wasm_result.is_err() {
            return Err(WasmError::HostCall(HostCallError::HostError(0)));
        }
        return Ok(&[]);
    }

    // Read response from host
    let response_bytes =
        unsafe { core::slice::from_raw_parts(slice.ptr as *const u8, slice.len as usize) };

    // Decode envelope
    let envelope = decode_envelope(response_bytes)?;

    if wasm_result.is_err() || envelope.header.is_error() {
        // Parse error code from payload if available
        let code = if envelope.payload.len() >= 4 {
            u32::from_le_bytes([
                envelope.payload[0],
                envelope.payload[1],
                envelope.payload[2],
                envelope.payload[3],
            ])
        } else {
            0
        };
        return Err(WasmError::HostCall(HostCallError::HostError(code)));
    }

    Ok(envelope.payload)
}

/// Macro for defining host extern functions
#[macro_export]
macro_rules! host_externs {
    ($($name:ident),* $(,)?) => {
        $(
            extern "C" {
                pub fn $name(ptr: u32, len: u32) -> u64;
            }
        )*
    };
}

/// Macro for calling a host function with automatic serialization
#[macro_export]
macro_rules! call_host {
    ($fn:ident, $args:expr) => {{
        // Serialize args
        let args_bytes: &[u8] = $args;
        $crate::host_call_raw($fn, args_bytes)
    }};
}

#[cfg(test)]
mod tests {
    // Host call tests require actual WASM environment
}
