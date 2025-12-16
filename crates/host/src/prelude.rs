//! Prelude module - convenient re-exports
//!
//! This module provides all commonly used types and functions
//! for hosting WASM guests.

pub use crate::{
    build_guest_result,
    consume_bytes_from_guest,
    move_data_to_guest,
    EngineConfig,
    // Cache (legacy)
    // ModuleCache from cache module - using module::ModuleCache instead
    // Environment
    Env,
    // Guest utilities
    // Note: ExternIO intentionally NOT exported to avoid conflict with aingle_zome_types::ExternIO
    GuestPtr,
    // Errors
    HostError,
    Len,
    // Engine
    WasmEngine,
    // Instance
    WasmInstance,
    // Constants
    DEFAULT_METERING_LIMIT,
};

// Module cache from the new module
pub use crate::module::ModuleCache;

// Conditionally export call function when wasmer is enabled
#[cfg(any(feature = "wasmer_sys_dev", feature = "wasmer_sys_prod"))]
pub use crate::guest::call;

pub use aingle_wasmer_common::{
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

// Re-export serde for user convenience
pub use serde::{de::DeserializeOwned, Deserialize, Serialize};
