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
mod instance;
mod env;
mod guest;
mod error;

/// Module caching with filesystem support
pub mod module;

pub mod prelude;

pub use engine::*;
pub use instance::*;
pub use env::*;
pub use guest::*;
pub use error::*;
pub use module::ModuleCache;

pub use aingle_wasmer_common::{
    WasmError, WasmErrorInner, WasmSlice, WasmResult,
    WasmEncode, WasmDecode, DoubleUSize,
    SerializeError, DeserializeError, HostCallError, GuestCallError,
};

/// Default metering limit: 100 billion operations
pub const DEFAULT_METERING_LIMIT: u64 = 100_000_000_000;

/// Test metering limit: 10 million operations
#[cfg(test)]
pub const TEST_METERING_LIMIT: u64 = 10_000_000;
