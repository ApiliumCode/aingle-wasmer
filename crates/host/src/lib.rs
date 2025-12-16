//! # AIngle WASM Host
//!
//! Host-side WASM execution engine for the AIngle conductor.
//!
//! ## Features
//! - Module caching with filesystem persistence
//! - Metering for resource limits
//! - Sandboxed execution
//! - Zero-copy data transfer where possible
//!
//! ## Example
//!
//! ```ignore
//! use aingle_wasmer_host::prelude::*;
//!
//! let cache = ModuleCache::new(None);
//! let module = cache.get(key, wasm_bytes)?;
//! ```

#![warn(missing_docs)]

mod engine;
mod env;
mod error;
/// Guest interaction utilities
pub mod guest;
mod instance;

/// Module caching with filesystem support
pub mod module;

pub mod prelude;

pub use engine::*;
pub use env::*;
pub use error::*;
pub use guest::*;
pub use instance::*;
pub use module::ModuleCache;

pub use aingle_wasmer_common::{
    DeserializeError, DoubleUSize, GuestCallError, HostCallError, SerializeError, WasmDecode,
    WasmEncode, WasmError, WasmErrorInner, WasmResult, WasmSlice,
};

/// Default metering limit: 100 billion operations
pub const DEFAULT_METERING_LIMIT: u64 = 100_000_000_000;

/// Test metering limit: 10 million operations
#[cfg(test)]
pub const TEST_METERING_LIMIT: u64 = 10_000_000;
