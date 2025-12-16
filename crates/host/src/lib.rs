//! # AIngle WASM Host
//!
//! Host-side WASM execution engine for the AIngle conductor.
//!
//! ## Features
//! - Module caching with PLRU eviction
//! - Metering for resource limits
//! - Sandboxed execution
//! - Zero-copy data transfer where possible
//!
//! ## Example
//!
//! ```ignore
//! use aingle_wasm_host::prelude::*;
//!
//! let engine = WasmEngine::new(EngineConfig::default())?;
//! let module = engine.compile(wasm_bytes)?;
//! let instance = engine.instantiate(&module, imports)?;
//!
//! let result: Output = instance.call("my_function", input)?;
//! ```

#![warn(missing_docs)]

mod engine;
mod instance;
mod cache;
mod env;
mod guest;
mod error;

pub mod prelude;

pub use engine::*;
pub use instance::*;
pub use cache::*;
pub use env::*;
pub use guest::*;
pub use error::*;

pub use aingle_wasm_types::{
    WasmError, WasmSlice, WasmResult,
    WasmEncode, WasmDecode,
};

/// Default metering limit: 100 billion operations
pub const DEFAULT_METERING_LIMIT: u64 = 100_000_000_000;

/// Test metering limit: 10 million operations
#[cfg(test)]
pub const TEST_METERING_LIMIT: u64 = 10_000_000;
