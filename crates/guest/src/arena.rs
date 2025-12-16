//! Arena-based memory allocation for WASM guests
//!
//! Uses bumpalo for fast, sequential allocation with bulk deallocation.

use bumpalo::Bump;
use core::cell::RefCell;

thread_local! {
    /// The global arena for this WASM instance
    pub static ARENA: GuestArena = GuestArena::new();
}

/// Arena allocator for WASM guest memory
pub struct GuestArena {
    bump: RefCell<Bump>,
}

impl GuestArena {
    /// Create a new arena
    pub fn new() -> Self {
        Self {
            bump: RefCell::new(Bump::new()),
        }
    }

    /// Allocate bytes from the arena
    pub fn alloc(&self, len: usize) -> *mut u8 {
        self.bump.borrow().alloc_layout(
            core::alloc::Layout::from_size_align(len, 1).unwrap()
        ).as_ptr()
    }

    /// Allocate and copy bytes
    pub fn alloc_copy(&self, data: &[u8]) -> *mut u8 {
        let ptr = self.alloc(data.len());
        unsafe {
            core::ptr::copy_nonoverlapping(data.as_ptr(), ptr, data.len());
        }
        ptr
    }

    /// Reset the arena, deallocating all memory
    pub fn reset(&self) {
        self.bump.borrow_mut().reset();
    }

    /// Get allocated bytes count
    pub fn allocated_bytes(&self) -> usize {
        self.bump.borrow().allocated_bytes()
    }
}

impl Default for GuestArena {
    fn default() -> Self {
        Self::new()
    }
}

/// Allocate from the global arena
pub fn arena_alloc(len: usize) -> *mut u8 {
    ARENA.with(|arena| arena.alloc(len))
}

/// Allocate and copy from the global arena
pub fn arena_alloc_copy(data: &[u8]) -> *mut u8 {
    ARENA.with(|arena| arena.alloc_copy(data))
}

/// Reset the global arena
pub fn arena_reset() {
    ARENA.with(|arena| arena.reset());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arena_alloc() {
        let arena = GuestArena::new();

        let ptr1 = arena.alloc(100);
        let ptr2 = arena.alloc(200);

        assert!(!ptr1.is_null());
        assert!(!ptr2.is_null());
        assert_ne!(ptr1, ptr2);
    }

    #[test]
    fn test_arena_copy() {
        let arena = GuestArena::new();
        let data = b"hello world";

        let ptr = arena.alloc_copy(data);

        let copied = unsafe { core::slice::from_raw_parts(ptr, data.len()) };
        assert_eq!(copied, data);
    }

    #[test]
    fn test_arena_reset() {
        let arena = GuestArena::new();

        arena.alloc(1000);
        let before = arena.allocated_bytes();
        assert!(before > 0);

        arena.reset();
        // After reset, new allocations start fresh
    }
}
