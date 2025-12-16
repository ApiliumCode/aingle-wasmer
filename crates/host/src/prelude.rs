//! Prelude module - convenient re-exports
//!
//! This module provides all commonly used types and functions
//! for hosting WASM guests.

pub use crate::{
    // Engine
    WasmEngine, EngineConfig,
    // Instance
    WasmInstance,
    // Cache (legacy)
    // ModuleCache from cache module - using module::ModuleCache instead
    // Environment
    Env, GuestPtr, Len,
    // Guest utilities
    ExternIO,
    consume_bytes_from_guest, move_data_to_guest, build_guest_result,
    // Errors
    HostError,
    // Constants
    DEFAULT_METERING_LIMIT,
};

// Module cache from the new module
pub use crate::module::ModuleCache;

// Conditionally export call function when wasmer is enabled
#[cfg(any(feature = "wasmer_sys_dev", feature = "wasmer_sys_prod"))]
pub use crate::guest::call;

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
};

pub use aingle_wasmer_codec::{
    encode_with_envelope, encode_to_slice,
    decode_envelope, decode_raw, DecodedEnvelope,
    compute_checksum, verify_checksum,
    Encoder, Decoder,
};

// Re-export serde for user convenience
pub use serde::{Serialize, Deserialize, de::DeserializeOwned};
