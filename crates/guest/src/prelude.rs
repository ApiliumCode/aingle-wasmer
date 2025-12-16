//! Prelude module - convenient re-exports
//!
//! This module provides all commonly used types and functions
//! for WASM guest development.

pub use crate::{
    // Arena
    arena_alloc,
    arena_alloc_copy,
    arena_reset,
    call_host,
    // Compatibility layer (for ADK)
    // Note: SerializedBytes is NOT exported - use from aingle_zome_types
    host_args,
    // Memory (internal)
    host_args_envelope,
    host_call,
    // Host calls (internal)
    host_call_raw,
    host_externs,
    read_bytes,
    return_err,
    return_err_ptr,
    return_ok,
    return_ptr,
    // Macros
    try_result,
    GuestArena,
    GuestPtr,
    Len,
    ARENA,
};

pub use aingle_wasmer_common::{
    // Macros
    wasm_error,
    DeserializeError,
    DoubleUSize,
    EnvelopeError,
    EnvelopeFlags,
    // Envelope
    EnvelopeHeader,
    ErrorKind,
    GuestCallError,
    HostCallError,
    MemoryError,
    SerializeError,
    WasmDecode,
    // Traits
    WasmEncode,
    // Errors
    WasmError,
    WasmErrorInner,
    WasmPrimitive,
    WasmRef,
    WasmResult,
    WasmSafe,
    // Types
    WasmSlice,
    // Constants
    MAGIC,
    PROTOCOL_VERSION,
};

pub use aingle_wasmer_codec::{
    compute_checksum, decode_envelope, decode_raw, encode_to_slice, encode_with_envelope,
    verify_checksum, DecodedEnvelope, Decoder, Encoder,
};

// Re-export serde traits for user convenience
pub use serde::{de::DeserializeOwned, Deserialize, Serialize};
