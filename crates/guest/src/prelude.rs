//! Prelude module - convenient re-exports

pub use crate::{
    // Memory
    host_args, read_bytes, return_ok, return_err,
    // Arena
    arena_alloc, arena_alloc_copy, arena_reset, GuestArena, ARENA,
    // Host calls
    host_call_raw,
    // Macros
    try_result, host_externs, call_host,
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
    // Macros
    wasm_error,
};

pub use aingle_wasm_codec::{
    encode_with_envelope, encode_to_slice,
    decode_envelope, decode_raw, DecodedEnvelope,
    compute_checksum, verify_checksum,
    Encoder, Decoder,
};
