//! Environment for WASM instances
//!
//! Provides the execution environment for WASM guest code, including
//! memory management and data transfer between host and guest.

use crate::HostError;
use aingle_wasmer_common::{WasmError, WasmSlice, WasmResult};
use serde::{Serialize, de::DeserializeOwned};

#[cfg(feature = "wasmer_sys_dev")]
use wasmer::{Memory, MemoryView, StoreMut, TypedFunction};

#[cfg(feature = "wasmer_sys_prod")]
use wasmer::{Memory, MemoryView, StoreMut, TypedFunction};

/// Guest pointer type
pub type GuestPtr = u32;

/// Length type
pub type Len = u32;

/// Environment data passed to WASM instances
///
/// This struct holds references to the WASM memory and allocation functions,
/// which are set after the instance is created.
#[derive(Clone, Default)]
pub struct Env {
    /// The WASM linear memory
    pub memory: Option<Memory>,
    /// Function to allocate memory in the guest
    pub allocate: Option<TypedFunction<i32, i32>>,
    /// Function to deallocate memory in the guest
    pub deallocate: Option<TypedFunction<(i32, i32), ()>>,
}

impl Env {
    /// Create a new empty environment
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if the environment is initialized
    pub fn is_initialized(&self) -> bool {
        self.memory.is_some() && self.allocate.is_some() && self.deallocate.is_some()
    }

    /// Consume bytes from guest memory
    ///
    /// Reads `len` bytes starting at `guest_ptr` from the guest's linear memory.
    /// The memory is read directly without any envelope/protocol processing.
    ///
    /// # Arguments
    /// * `store` - Mutable reference to the Wasmer store
    /// * `guest_ptr` - Pointer to the start of the data in guest memory
    /// * `len` - Number of bytes to read
    ///
    /// # Returns
    /// * `Ok(Vec<u8>)` - The bytes read from guest memory
    /// * `Err(HostError)` - If memory access fails
    pub fn consume_bytes_from_guest<'a>(
        &self,
        store: &'a mut StoreMut<'_>,
        guest_ptr: GuestPtr,
        len: Len,
    ) -> Result<Vec<u8>, HostError> {
        let memory = self.memory.as_ref()
            .ok_or_else(|| HostError::MemoryAccess("Memory not initialized".to_string()))?;

        let view = memory.view(store);
        let start = guest_ptr as u64;
        let end = start + len as u64;

        if end > view.data_size() {
            return Err(HostError::MemoryAccess(format!(
                "Out of bounds: {}..{} > {}",
                start, end, view.data_size()
            )));
        }

        let mut buffer = vec![0u8; len as usize];
        view.read(start, &mut buffer)
            .map_err(|e| HostError::MemoryAccess(format!("Failed to read memory: {}", e)))?;

        Ok(buffer)
    }

    /// Move data to guest memory
    ///
    /// Serializes the data and writes it to guest memory, returning the pointer/length.
    /// This handles allocation in the guest and returns a combined u64 value.
    ///
    /// # Type Parameters
    /// * `T` - The type to serialize (must implement Serialize)
    ///
    /// # Arguments
    /// * `store` - Mutable reference to the Wasmer store
    /// * `data` - The data to serialize and move to guest memory
    ///
    /// # Returns
    /// * `Ok(u64)` - Combined pointer/length value (ptr << 32 | len)
    /// * `Err(HostError)` - If allocation or memory write fails
    pub fn move_data_to_guest<T: Serialize>(
        &self,
        store: &mut StoreMut<'_>,
        data: T,
    ) -> Result<u64, HostError> {
        // Serialize the data
        let bytes = rmp_serde::to_vec_named(&data)
            .map_err(|e| HostError::Serialization(format!("Failed to serialize: {}", e)))?;

        self.move_bytes_to_guest(store, &bytes)
    }

    /// Move raw bytes to guest memory
    ///
    /// Allocates memory in the guest and copies the bytes there.
    ///
    /// # Arguments
    /// * `store` - Mutable reference to the Wasmer store
    /// * `bytes` - The bytes to copy to guest memory
    ///
    /// # Returns
    /// * `Ok(u64)` - Combined pointer/length value (ptr << 32 | len)
    /// * `Err(HostError)` - If allocation or memory write fails
    pub fn move_bytes_to_guest(
        &self,
        store: &mut StoreMut<'_>,
        bytes: &[u8],
    ) -> Result<u64, HostError> {
        let memory = self.memory.as_ref()
            .ok_or_else(|| HostError::MemoryAccess("Memory not initialized".to_string()))?;
        let allocate = self.allocate.as_ref()
            .ok_or_else(|| HostError::MemoryAccess("Allocate function not initialized".to_string()))?;

        let len = bytes.len() as i32;

        // Allocate memory in the guest
        let ptr = allocate.call(store, len)
            .map_err(|e| HostError::MemoryAccess(format!("Failed to allocate: {}", e)))?;

        // Write bytes to guest memory
        let view = memory.view(store);
        view.write(ptr as u64, bytes)
            .map_err(|e| HostError::MemoryAccess(format!("Failed to write to memory: {}", e)))?;

        // Return combined pointer/length
        let slice = WasmSlice::new(ptr as u32, len as u32);
        Ok(slice.pack())
    }

    /// Deallocate memory in the guest
    ///
    /// # Arguments
    /// * `store` - Mutable reference to the Wasmer store
    /// * `ptr` - Pointer to the memory to deallocate
    /// * `len` - Length of the memory to deallocate
    pub fn deallocate_in_guest(
        &self,
        store: &mut StoreMut<'_>,
        ptr: GuestPtr,
        len: Len,
    ) -> Result<(), HostError> {
        if let Some(deallocate) = self.deallocate.as_ref() {
            deallocate.call(store, ptr as i32, len as i32)
                .map_err(|e| HostError::MemoryAccess(format!("Failed to deallocate: {}", e)))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_env_default() {
        let env = Env::new();
        assert!(!env.is_initialized());
        assert!(env.memory.is_none());
        assert!(env.allocate.is_none());
        assert!(env.deallocate.is_none());
    }
}
