//! # AIngle WASM Codec
//!
//! Zero-copy serialization codec optimized for WASM host↔guest communication.
//!
//! ## Design Goals
//! - Minimal allocations
//! - Fast encode/decode
//! - Checksum validation
//! - Streaming support

#![warn(missing_docs)]

mod encode;
mod decode;
mod checksum;

pub use encode::*;
pub use decode::*;
pub use checksum::*;

pub use aingle_wasm_types::{
    EnvelopeHeader, EnvelopeFlags, WasmSlice, WasmResult,
    WasmEncode, WasmDecode, WasmError,
};
