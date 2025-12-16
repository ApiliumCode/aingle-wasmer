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
//! use aingle_wasm_guest::prelude::*;
//!
//! #[aingle_entry]
//! fn my_function(input: MyInput) -> Result<MyOutput, WasmError> {
//!     // Your logic here
//!     Ok(output)
//! }
//! ```

#![warn(missing_docs)]

mod arena;
mod memory;
mod host_call;

pub mod prelude;

pub use arena::*;
pub use memory::*;
pub use host_call::*;

pub use aingle_wasm_types::{
    WasmError, WasmSlice, WasmResult, DoubleUSize,
    WasmEncode, WasmDecode, WasmPrimitive,
};

pub use aingle_wasm_codec::{
    encode_with_envelope, decode_envelope,
};

/// Memory extern functions that the host will provide
#[cfg(target_arch = "wasm32")]
extern "C" {
    /// Allocate memory in the guest
    fn __aingle_allocate(len: u32) -> u32;
    /// Deallocate memory in the guest
    fn __aingle_deallocate(ptr: u32, len: u32);
}

/// Allocate memory for use by the host
#[no_mangle]
pub extern "C" fn __aingle_guest_allocate(len: u32) -> u32 {
    ARENA.with(|arena| arena.alloc(len as usize) as u32)
}

/// Deallocate memory (no-op with arena, cleared on call end)
#[no_mangle]
pub extern "C" fn __aingle_guest_deallocate(_ptr: u32, _len: u32) {
    // Arena allocation - no individual deallocation
    // Memory is reclaimed when arena is reset
}

/// Reset the arena (called by host at end of each call)
#[no_mangle]
pub extern "C" fn __aingle_guest_reset_arena() {
    ARENA.with(|arena| arena.reset());
}
