//! WASM instance management

use crate::{WasmEngine, HostError, Env};
#[allow(unused_imports)]
use aingle_wasm_types::WasmSlice;
use aingle_wasm_types::WasmResult;
use aingle_wasm_codec::{encode_with_envelope, decode_envelope};

#[cfg(any(feature = "wasmer_sys_dev", feature = "wasmer_sys_prod"))]
use wasmer::{Module, Instance, Store, imports, Memory, MemoryType};

/// A WASM instance ready for execution
pub struct WasmInstance {
    #[cfg(any(feature = "wasmer_sys_dev", feature = "wasmer_sys_prod"))]
    instance: Instance,
    #[cfg(any(feature = "wasmer_sys_dev", feature = "wasmer_sys_prod"))]
    store: Store,
    #[allow(dead_code)]
    env: Env,
}

impl WasmInstance {
    /// Create a new instance from a module
    #[cfg(any(feature = "wasmer_sys_dev", feature = "wasmer_sys_prod"))]
    pub fn new(engine: &WasmEngine, module: &Module) -> Result<Self, HostError> {
        let mut store = Store::new(engine.inner().clone());
        let env = Env::new();

        // Create memory
        let memory = Memory::new(&mut store, MemoryType::new(1, None, false))
            .map_err(|e| HostError::Instantiation(e.to_string()))?;

        // Build minimal imports
        let import_object = imports! {
            "env" => {
                "memory" => memory,
            },
        };

        let instance = Instance::new(&mut store, module, &import_object)
            .map_err(|e| HostError::Instantiation(e.to_string()))?;

        Ok(Self { instance, store, env })
    }

    /// Call a function on the instance
    #[cfg(any(feature = "wasmer_sys_dev", feature = "wasmer_sys_prod"))]
    pub fn call_raw(&mut self, name: &str, args: &[u8]) -> Result<Vec<u8>, HostError> {
        // Get the function
        let func = self.instance
            .exports
            .get_function(name)
            .map_err(|_| HostError::FunctionNotFound(name.to_string()))?;

        // Encode args with envelope
        let mut buffer = vec![0u8; args.len() + 64];
        let len = encode_with_envelope(args, 0, &mut buffer)
            .map_err(|e| HostError::Serialization(format!("{:?}", e)))?;

        // Get memory for writing
        let memory = self.instance
            .exports
            .get_memory("memory")
            .map_err(|_| HostError::MemoryNotFound)?;

        // Write to guest memory at fixed offset
        let ptr: u32 = 1024;
        {
            let view = memory.view(&self.store);
            view.write(ptr as u64, &buffer[..len])
                .map_err(|e| HostError::MemoryAccess(e.to_string()))?;
        }

        // Call the function
        let result = func.call(&mut self.store, &[
            wasmer::Value::I32(ptr as i32),
            wasmer::Value::I32(len as i32),
        ]).map_err(|e| HostError::Runtime(e.to_string()))?;

        // Parse result
        let result_packed = match result.first() {
            Some(wasmer::Value::I64(v)) => *v as u64,
            _ => return Err(HostError::InvalidReturn),
        };

        let wasm_result = WasmResult::from_raw(result_packed);
        let slice = wasm_result.slice();

        if slice.is_empty() {
            if wasm_result.is_err() {
                return Err(HostError::GuestError("empty error".to_string()));
            }
            return Ok(vec![]);
        }

        // Read response from guest memory
        let mut response = vec![0u8; slice.len as usize];
        {
            let view = memory.view(&self.store);
            view.read(slice.ptr as u64, &mut response)
                .map_err(|e| HostError::MemoryAccess(e.to_string()))?;
        }

        // Decode envelope
        let envelope = decode_envelope(&response)
            .map_err(|e| HostError::Deserialization(format!("{:?}", e)))?;

        if wasm_result.is_err() || envelope.header.is_error() {
            return Err(HostError::GuestError(
                String::from_utf8_lossy(envelope.payload).to_string()
            ));
        }

        Ok(envelope.payload.to_vec())
    }

    /// Get reference to the store
    #[cfg(any(feature = "wasmer_sys_dev", feature = "wasmer_sys_prod"))]
    pub fn store(&self) -> &Store {
        &self.store
    }

    /// Get mutable reference to the store
    #[cfg(any(feature = "wasmer_sys_dev", feature = "wasmer_sys_prod"))]
    pub fn store_mut(&mut self) -> &mut Store {
        &mut self.store
    }
}

#[cfg(test)]
mod tests {
    // Instance tests require actual WASM modules
}
