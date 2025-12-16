//! # AIngle WASM Types
//!
//! Core types for the AIngle WASM runtime with zero external dependencies.
//! Designed for IoT and AI workloads with minimal memory footprint.
//!
//! ## Features
//! - Zero-copy envelope protocol
//! - Versioned wire format
//! - no_std compatible

#![no_std]
#![warn(missing_docs)]

mod envelope;
mod error;
mod slice;
mod traits;

pub use envelope::*;
pub use error::*;
pub use slice::*;
pub use traits::*;

/// Protocol version for the AIngle WASM envelope format
pub const PROTOCOL_VERSION: u8 = 1;

/// Magic bytes identifying AIngle WASM messages: "AI" (0x4149)
pub const MAGIC: u16 = 0x4149;
