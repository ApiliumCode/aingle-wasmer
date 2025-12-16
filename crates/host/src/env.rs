//! Environment for WASM instances

use std::sync::Arc;
use parking_lot::Mutex;

/// Environment data passed to WASM instances
#[derive(Clone)]
pub struct Env {
    /// Shared state for the instance
    state: Arc<Mutex<EnvState>>,
}

/// Internal state for the environment
struct EnvState {
    /// Allocated memory regions (ptr, len)
    allocations: Vec<(u32, u32)>,
    /// Next allocation pointer
    next_ptr: u32,
}

impl Env {
    /// Create a new environment
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(EnvState {
                allocations: Vec::new(),
                next_ptr: 4096, // Start after reserved space
            })),
        }
    }

    /// Allocate memory in the guest
    pub fn allocate(&self, len: u32) -> u32 {
        let mut state = self.state.lock();
        let ptr = state.next_ptr;
        state.next_ptr += len;
        state.next_ptr = (state.next_ptr + 7) & !7; // Align to 8 bytes
        state.allocations.push((ptr, len));
        ptr
    }

    /// Deallocate memory (no-op with arena model)
    pub fn deallocate(&self, _ptr: u32, _len: u32) {
        // No-op - memory reclaimed on instance reset
    }

    /// Reset allocations (for instance reuse)
    pub fn reset(&self) {
        let mut state = self.state.lock();
        state.allocations.clear();
        state.next_ptr = 4096;
    }

    /// Get total allocated bytes
    pub fn allocated_bytes(&self) -> u32 {
        let state = self.state.lock();
        state.allocations.iter().map(|(_, len)| len).sum()
    }
}

impl Default for Env {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_env_allocation() {
        let env = Env::new();

        let ptr1 = env.allocate(100);
        let ptr2 = env.allocate(200);

        assert!(ptr1 < ptr2);
        assert_eq!(env.allocated_bytes(), 300);
    }

    #[test]
    fn test_env_reset() {
        let env = Env::new();

        env.allocate(100);
        assert_eq!(env.allocated_bytes(), 100);

        env.reset();
        assert_eq!(env.allocated_bytes(), 0);
    }
}
