//! WASM Module caching
//!
//! Provides efficient caching of compiled WASM modules with optional
//! filesystem persistence.

use crate::HostError;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

#[cfg(any(feature = "wasmer_sys_dev", feature = "wasmer_sys_prod"))]
use wasmer::{Engine, Module, Store};

/// Cache for compiled WASM modules
///
/// Stores compiled modules in memory and optionally on disk for
/// faster subsequent loads. Thread-safe for concurrent access.
pub struct ModuleCache {
    /// In-memory cache of compiled modules
    #[cfg(any(feature = "wasmer_sys_dev", feature = "wasmer_sys_prod"))]
    modules: RwLock<HashMap<[u8; 32], Arc<Module>>>,

    /// Optional filesystem cache directory
    cache_path: Option<PathBuf>,

    /// Wasmer engine for compilation
    #[cfg(any(feature = "wasmer_sys_dev", feature = "wasmer_sys_prod"))]
    engine: Engine,
}

impl ModuleCache {
    /// Create a new module cache
    ///
    /// # Arguments
    /// * `cache_path` - Optional filesystem path for persistent caching
    pub fn new(cache_path: Option<PathBuf>) -> Self {
        #[cfg(any(feature = "wasmer_sys_dev", feature = "wasmer_sys_prod"))]
        {
            Self {
                modules: RwLock::new(HashMap::new()),
                cache_path,
                engine: Engine::default(),
            }
        }

        #[cfg(not(any(feature = "wasmer_sys_dev", feature = "wasmer_sys_prod")))]
        {
            Self { cache_path }
        }
    }

    /// Get or compile a module
    ///
    /// If the module is cached (in memory or on disk), returns the cached version.
    /// Otherwise, compiles the WASM bytes and caches the result.
    ///
    /// # Arguments
    /// * `key` - 32-byte unique identifier for the module (typically a hash)
    /// * `wasm_bytes` - The raw WASM bytecode
    ///
    /// # Returns
    /// * `Ok(Arc<Module>)` - The compiled module
    /// * `Err(HostError)` - If compilation fails
    #[cfg(any(feature = "wasmer_sys_dev", feature = "wasmer_sys_prod"))]
    pub fn get(&self, key: [u8; 32], wasm_bytes: &[u8]) -> Result<Arc<Module>, HostError> {
        // Check in-memory cache first
        {
            let cache = self.modules.read();
            if let Some(module) = cache.get(&key) {
                return Ok(Arc::clone(module));
            }
        }

        // Try to load from filesystem cache
        if let Some(module) = self.load_from_disk(&key) {
            let arc_module = Arc::new(module);
            let mut cache = self.modules.write();
            cache.insert(key, Arc::clone(&arc_module));
            return Ok(arc_module);
        }

        // Compile the module
        let module = Module::new(&self.engine, wasm_bytes)
            .map_err(|e| HostError::Compilation(format!("Failed to compile WASM: {}", e)))?;

        // Save to disk if path is configured
        self.save_to_disk(&key, &module);

        // Cache in memory
        let arc_module = Arc::new(module);
        {
            let mut cache = self.modules.write();
            cache.insert(key, Arc::clone(&arc_module));
        }

        Ok(arc_module)
    }

    /// Load a module from the filesystem cache
    #[cfg(any(feature = "wasmer_sys_dev", feature = "wasmer_sys_prod"))]
    fn load_from_disk(&self, key: &[u8; 32]) -> Option<Module> {
        let path = self.cache_path.as_ref()?;
        let file_path = path.join(hex::encode(key));

        if !file_path.exists() {
            return None;
        }

        // Try to load the serialized module
        let bytes = std::fs::read(&file_path).ok()?;

        // Deserialize the module
        // Note: This is unsafe as it loads pre-compiled code
        unsafe { Module::deserialize(&self.engine, &bytes).ok() }
    }

    /// Save a module to the filesystem cache
    #[cfg(any(feature = "wasmer_sys_dev", feature = "wasmer_sys_prod"))]
    fn save_to_disk(&self, key: &[u8; 32], module: &Module) {
        let Some(path) = self.cache_path.as_ref() else {
            return;
        };

        // Create cache directory if needed
        if let Err(e) = std::fs::create_dir_all(path) {
            tracing::warn!("Failed to create cache directory: {}", e);
            return;
        }

        let file_path = path.join(hex::encode(key));

        // Serialize and save
        match module.serialize() {
            Ok(bytes) => {
                if let Err(e) = std::fs::write(&file_path, bytes) {
                    tracing::warn!("Failed to write module to cache: {}", e);
                }
            }
            Err(e) => {
                tracing::warn!("Failed to serialize module: {}", e);
            }
        }
    }

    /// Clear the in-memory cache
    #[cfg(any(feature = "wasmer_sys_dev", feature = "wasmer_sys_prod"))]
    pub fn clear(&self) {
        let mut cache = self.modules.write();
        cache.clear();
    }

    /// Get the number of cached modules
    #[cfg(any(feature = "wasmer_sys_dev", feature = "wasmer_sys_prod"))]
    pub fn len(&self) -> usize {
        self.modules.read().len()
    }

    /// Check if cache is empty
    #[cfg(any(feature = "wasmer_sys_dev", feature = "wasmer_sys_prod"))]
    pub fn is_empty(&self) -> bool {
        self.modules.read().is_empty()
    }

    /// Get the cache path
    pub fn cache_path(&self) -> Option<&PathBuf> {
        self.cache_path.as_ref()
    }
}

impl Default for ModuleCache {
    fn default() -> Self {
        Self::new(None)
    }
}

/// Helper to convert bytes to hex string
mod hex {
    pub fn encode(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{:02x}", b)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_creation() {
        let cache = ModuleCache::new(None);
        assert!(cache.cache_path().is_none());
    }

    #[test]
    fn test_cache_with_path() {
        let path = PathBuf::from("/tmp/test_cache");
        let cache = ModuleCache::new(Some(path.clone()));
        assert_eq!(cache.cache_path(), Some(&path));
    }

    #[test]
    fn test_hex_encode() {
        assert_eq!(hex::encode(&[0xde, 0xad, 0xbe, 0xef]), "deadbeef");
    }
}
