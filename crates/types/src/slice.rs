//! WASM memory slice types for zero-copy operations

use core::marker::PhantomData;

/// A slice of WASM memory represented as pointer + length
///
/// This is the fundamental type for passing data across the
/// host↔guest boundary without copying.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WasmSlice {
    /// Pointer to the start of the data in WASM linear memory
    pub ptr: u32,
    /// Length of the data in bytes
    pub len: u32,
}

impl WasmSlice {
    /// Create a new WASM slice
    #[inline]
    pub const fn new(ptr: u32, len: u32) -> Self {
        Self { ptr, len }
    }

    /// Create an empty slice
    #[inline]
    pub const fn empty() -> Self {
        Self { ptr: 0, len: 0 }
    }

    /// Check if the slice is empty
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Pack into a single u64 for return values
    ///
    /// Format: high 32 bits = ptr, low 32 bits = len
    #[inline]
    pub const fn pack(&self) -> u64 {
        ((self.ptr as u64) << 32) | (self.len as u64)
    }

    /// Unpack from a u64 return value
    #[inline]
    pub const fn unpack(packed: u64) -> Self {
        Self {
            ptr: (packed >> 32) as u32,
            len: packed as u32,
        }
    }

    /// Get the end offset (ptr + len)
    #[inline]
    pub const fn end(&self) -> u32 {
        self.ptr.saturating_add(self.len)
    }

    /// Check if this slice overlaps with another
    #[inline]
    pub const fn overlaps(&self, other: &WasmSlice) -> bool {
        self.ptr < other.end() && other.ptr < self.end()
    }

    /// Check if an offset is within this slice
    #[inline]
    pub const fn contains(&self, offset: u32) -> bool {
        offset >= self.ptr && offset < self.end()
    }
}

/// A typed reference to data in WASM memory
///
/// Provides type safety for WASM memory access without
/// actually containing the data (zero-copy).
#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct WasmRef<T> {
    slice: WasmSlice,
    _phantom: PhantomData<T>,
}

impl<T> WasmRef<T> {
    /// Create a new typed WASM reference
    #[inline]
    pub const fn new(slice: WasmSlice) -> Self {
        Self {
            slice,
            _phantom: PhantomData,
        }
    }

    /// Get the underlying slice
    #[inline]
    pub const fn slice(&self) -> WasmSlice {
        self.slice
    }

    /// Get the pointer
    #[inline]
    pub const fn ptr(&self) -> u32 {
        self.slice.ptr
    }

    /// Get the length
    #[inline]
    pub const fn len(&self) -> u32 {
        self.slice.len
    }

    /// Check if empty
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.slice.is_empty()
    }
}

/// Result type for WASM operations, packed for return
///
/// Uses a single u64 where:
/// - Bit 63 (high bit): 0 = Ok, 1 = Err
/// - Bits 0-62: payload (WasmSlice packed)
#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct WasmResult(u64);

impl WasmResult {
    const ERROR_BIT: u64 = 1 << 63;

    /// Create a successful result
    #[inline]
    pub const fn ok(slice: WasmSlice) -> Self {
        Self(slice.pack())
    }

    /// Create an error result
    #[inline]
    pub const fn err(slice: WasmSlice) -> Self {
        Self(slice.pack() | Self::ERROR_BIT)
    }

    /// Check if this is an error
    #[inline]
    pub const fn is_err(&self) -> bool {
        self.0 & Self::ERROR_BIT != 0
    }

    /// Check if this is ok
    #[inline]
    pub const fn is_ok(&self) -> bool {
        !self.is_err()
    }

    /// Get the underlying slice
    #[inline]
    pub const fn slice(&self) -> WasmSlice {
        WasmSlice::unpack(self.0 & !Self::ERROR_BIT)
    }

    /// Convert to raw u64
    #[inline]
    pub const fn into_raw(self) -> u64 {
        self.0
    }

    /// Create from raw u64
    #[inline]
    pub const fn from_raw(raw: u64) -> Self {
        Self(raw)
    }
}

/// Double usize for guest function returns (compatibility type)
pub type DoubleUSize = u64;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slice_pack_unpack() {
        let slice = WasmSlice::new(0x12345678, 0x1000);
        let packed = slice.pack();
        let unpacked = WasmSlice::unpack(packed);

        assert_eq!(unpacked.ptr, 0x12345678);
        assert_eq!(unpacked.len, 0x1000);
    }

    #[test]
    fn test_slice_overlap() {
        let a = WasmSlice::new(100, 50); // 100-150
        let b = WasmSlice::new(120, 50); // 120-170
        let c = WasmSlice::new(200, 50); // 200-250

        assert!(a.overlaps(&b));
        assert!(b.overlaps(&a));
        assert!(!a.overlaps(&c));
        assert!(!c.overlaps(&a));
    }

    #[test]
    fn test_wasm_result() {
        let ok = WasmResult::ok(WasmSlice::new(100, 50));
        assert!(ok.is_ok());
        assert!(!ok.is_err());
        assert_eq!(ok.slice().ptr, 100);

        let err = WasmResult::err(WasmSlice::new(200, 10));
        assert!(err.is_err());
        assert!(!err.is_ok());
        assert_eq!(err.slice().ptr, 200);
    }
}
