//! Prelude module - convenient re-exports

pub use crate::{
    // Engine
    WasmEngine, EngineConfig,
    // Instance
    WasmInstance,
    // Cache
    ModuleCache,
    // Environment
    Env,
    // Guest utilities
    call, consume_bytes_from_guest, move_data_to_guest, build_guest_result,
    // Errors
    HostError,
    // Constants
    DEFAULT_METERING_LIMIT,
};

pub use aingle_wasm_types::{
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

pub use aingle_wasm_codec::{
    encode_with_envelope, encode_to_slice,
    decode_envelope, decode_raw, DecodedEnvelope,
    compute_checksum, verify_checksum,
    Encoder, Decoder,
};
