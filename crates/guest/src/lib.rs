//! # AIngle WASM Guest
//!
//! Guest-side utilities for WASM modules running in AIngle.
//!
//! ## Features
//! - Arena-based memory allocation (bumpalo)
//! - Ergonomic macros for defining entry points
//! - Automatic serialization/deserialization
//! - Zero-copy data passing where possible
//!
//! ## Example
//!
//! ```ignore
//! use aingle_wasmer_guest::prelude::*;
//!
//! #[aingle_entry]
//! fn my_function(input: MyInput) -> Result<MyOutput, WasmError> {
//!     // Your logic here
//!     Ok(output)
//! }
//! ```

#![warn(missing_docs)]

mod arena;
mod compat;
mod host_call;
mod memory;

pub mod prelude;

pub use arena::*;
pub use host_call::*;
pub use memory::{host_args_envelope, read_bytes, return_err, return_ok};
// Export compat functions but NOT SerializedBytes (conflicts with aingle_zome_types)
pub use compat::{host_args, host_call, return_err_ptr, return_ptr, GuestPtr, Len};

pub use aingle_wasmer_common::{
    DeserializeError, DoubleUSize, GuestCallError, HostCallError, SerializeError, WasmDecode,
    WasmEncode, WasmError, WasmErrorInner, WasmPrimitive, WasmResult, WasmSlice,
};

pub use aingle_wasmer_codec::{decode_envelope, encode_with_envelope};

// Re-export serde for convenience
pub use serde;

/// Memory extern functions that the host will provide
#[cfg(target_arch = "wasm32")]
extern "C" {
    /// Allocate memory in the guest
    fn __aingle_allocate(len: u32) -> u32;
    /// Deallocate memory in the guest
    fn __aingle_deallocate(ptr: u32, len: u32);
}

/// Allocate memory for use by the host (new naming)
#[no_mangle]
pub extern "C" fn __aingle_guest_allocate(len: u32) -> u32 {
    ARENA.with(|arena| arena.alloc(len as usize) as u32)
}

/// Allocate memory for use by the host (holochain-compatible naming)
#[no_mangle]
pub extern "C" fn __hc__allocate_1(len: i32) -> i32 {
    ARENA.with(|arena| arena.alloc(len as usize) as i32)
}

/// Deallocate memory (no-op with arena, cleared on call end)
#[no_mangle]
pub extern "C" fn __aingle_guest_deallocate(_ptr: u32, _len: u32) {
    // Arena allocation - no individual deallocation
    // Memory is reclaimed when arena is reset
}

/// Deallocate memory (holochain-compatible naming)
#[no_mangle]
pub extern "C" fn __hc__deallocate_1(_ptr: i32, _len: i32) {
    // Arena allocation - no individual deallocation
    // Memory is reclaimed when arena is reset
}

/// Reset the arena (called by host at end of each call)
#[no_mangle]
pub extern "C" fn __aingle_guest_reset_arena() {
    ARENA.with(|arena| arena.reset());
}

// Re-export middleware_bytes types for aingle compatibility
pub use aingle_middleware_bytes;
