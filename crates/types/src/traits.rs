//! Traits for WASM serialization and guest/host communication

use crate::{WasmError, WasmSlice};

/// Trait for types that can be encoded to WASM memory
pub trait WasmEncode {
    /// Calculate the encoded size in bytes
    fn encoded_size(&self) -> usize;

    /// Encode into the provided buffer
    ///
    /// Returns the number of bytes written.
    fn encode_to(&self, buf: &mut [u8]) -> Result<usize, WasmError>;
}

/// Trait for types that can be decoded from WASM memory
pub trait WasmDecode: Sized {
    /// Decode from a byte slice
    fn decode_from(buf: &[u8]) -> Result<Self, WasmError>;
}

/// Trait for types that can be passed to/from WASM as a single value
///
/// This is for primitive types that fit in a WASM value (i32, i64, f32, f64).
pub trait WasmPrimitive: Copy {
    /// The WASM type this maps to
    type WasmType: Copy;

    /// Convert to WASM type
    fn to_wasm(self) -> Self::WasmType;

    /// Convert from WASM type
    fn from_wasm(wasm: Self::WasmType) -> Self;
}

/// Trait for host functions that can be imported into WASM
pub trait HostFunction<Args, Ret> {
    /// The name of the function
    const NAME: &'static str;

    /// Call the host function
    fn call(&self, args: Args) -> Result<Ret, WasmError>;
}

/// Trait for guest functions that can be called from the host
pub trait GuestFunction {
    /// The name of the function
    const NAME: &'static str;

    /// The input type
    type Input: WasmEncode;

    /// The output type
    type Output: WasmDecode;
}

/// Marker trait for types safe to share between host and guest
///
/// # Safety
/// Only implement this for types that:
/// - Have a stable memory layout
/// - Contain no pointers to host memory
/// - Are serializable/deserializable deterministically
pub unsafe trait WasmSafe {}

// Implement WasmPrimitive for common types
impl WasmPrimitive for u32 {
    type WasmType = u32;

    #[inline]
    fn to_wasm(self) -> u32 {
        self
    }

    #[inline]
    fn from_wasm(wasm: u32) -> Self {
        wasm
    }
}

impl WasmPrimitive for i32 {
    type WasmType = i32;

    #[inline]
    fn to_wasm(self) -> i32 {
        self
    }

    #[inline]
    fn from_wasm(wasm: i32) -> Self {
        wasm
    }
}

impl WasmPrimitive for u64 {
    type WasmType = u64;

    #[inline]
    fn to_wasm(self) -> u64 {
        self
    }

    #[inline]
    fn from_wasm(wasm: u64) -> Self {
        wasm
    }
}

impl WasmPrimitive for i64 {
    type WasmType = i64;

    #[inline]
    fn to_wasm(self) -> i64 {
        self
    }

    #[inline]
    fn from_wasm(wasm: i64) -> Self {
        wasm
    }
}

impl WasmPrimitive for f32 {
    type WasmType = f32;

    #[inline]
    fn to_wasm(self) -> f32 {
        self
    }

    #[inline]
    fn from_wasm(wasm: f32) -> Self {
        wasm
    }
}

impl WasmPrimitive for f64 {
    type WasmType = f64;

    #[inline]
    fn to_wasm(self) -> f64 {
        self
    }

    #[inline]
    fn from_wasm(wasm: f64) -> Self {
        wasm
    }
}

impl WasmPrimitive for bool {
    type WasmType = i32;

    #[inline]
    fn to_wasm(self) -> i32 {
        self as i32
    }

    #[inline]
    fn from_wasm(wasm: i32) -> Self {
        wasm != 0
    }
}

impl WasmPrimitive for WasmSlice {
    type WasmType = u64;

    #[inline]
    fn to_wasm(self) -> u64 {
        self.pack()
    }

    #[inline]
    fn from_wasm(wasm: u64) -> Self {
        WasmSlice::unpack(wasm)
    }
}

// Safety: Primitive types are safe to share
unsafe impl WasmSafe for u8 {}
unsafe impl WasmSafe for u16 {}
unsafe impl WasmSafe for u32 {}
unsafe impl WasmSafe for u64 {}
unsafe impl WasmSafe for i8 {}
unsafe impl WasmSafe for i16 {}
unsafe impl WasmSafe for i32 {}
unsafe impl WasmSafe for i64 {}
unsafe impl WasmSafe for f32 {}
unsafe impl WasmSafe for f64 {}
unsafe impl WasmSafe for bool {}
unsafe impl WasmSafe for WasmSlice {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bool_primitive() {
        assert_eq!(true.to_wasm(), 1i32);
        assert_eq!(false.to_wasm(), 0i32);
        assert_eq!(bool::from_wasm(1), true);
        assert_eq!(bool::from_wasm(0), false);
        assert_eq!(bool::from_wasm(42), true);
    }

    #[test]
    fn test_slice_primitive() {
        let slice = WasmSlice::new(100, 200);
        let wasm = slice.to_wasm();
        let back = WasmSlice::from_wasm(wasm);

        assert_eq!(back.ptr, 100);
        assert_eq!(back.len, 200);
    }
}
