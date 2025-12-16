//! Prelude module - convenient re-exports
//!
//! This module provides all commonly used types and functions
//! for WASM guest development.

pub use crate::{
    // Memory (internal)
    host_args_envelope, read_bytes, return_ok, return_err,
    // Arena
    arena_alloc, arena_alloc_copy, arena_reset, GuestArena, ARENA,
    // Host calls (internal)
    host_call_raw,
    // Macros
    try_result, host_externs, call_host,
    // Compatibility layer (for ADK)
    // Note: SerializedBytes is NOT exported - use from aingle_zome_types
    host_args, return_ptr, return_err_ptr, host_call,
    GuestPtr, Len,
};

pub use aingle_wasmer_common::{
    // Errors
    WasmError, WasmErrorInner, ErrorKind,
    SerializeError, DeserializeError, MemoryError,
    HostCallError, GuestCallError,
    // Types
    WasmSlice, WasmResult, DoubleUSize,
    WasmRef,
    // Traits
    WasmEncode, WasmDecode, WasmPrimitive, WasmSafe,
    // Envelope
    EnvelopeHeader, EnvelopeFlags, EnvelopeError,
    // Constants
    MAGIC, PROTOCOL_VERSION,
    // Macros
    wasm_error,
};

pub use aingle_wasmer_codec::{
    encode_with_envelope, encode_to_slice,
    decode_envelope, decode_raw, DecodedEnvelope,
    compute_checksum, verify_checksum,
    Encoder, Decoder,
};

// Re-export serde traits for user convenience
pub use serde::{Serialize, Deserialize, de::DeserializeOwned};
