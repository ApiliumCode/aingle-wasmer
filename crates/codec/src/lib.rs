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

mod checksum;
mod decode;
mod encode;

pub use checksum::*;
pub use decode::*;
pub use encode::*;

pub use aingle_wasmer_common::{
    EnvelopeFlags, EnvelopeHeader, WasmDecode, WasmEncode, WasmError, WasmResult, WasmSlice,
};
