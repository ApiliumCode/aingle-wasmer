//! Module caching with PLRU eviction

use std::collections::HashMap;

#[cfg(any(feature = "wasmer_sys_dev", feature = "wasmer_sys_prod"))]
use wasmer::Module;

/// Simple LRU cache for compiled modules
pub struct ModuleCache {
    #[cfg(any(feature = "wasmer_sys_dev", feature = "wasmer_sys_prod"))]
    entries: HashMap<Vec<u8>, CacheEntry>,
    max_size: usize,
    current_size: usize,
    access_counter: u64,
}

#[cfg(any(feature = "wasmer_sys_dev", feature = "wasmer_sys_prod"))]
struct CacheEntry {
    module: Module,
    size: usize,
    last_access: u64,
}

impl ModuleCache {
    /// Create a new cache with the given maximum size in bytes
    pub fn new(max_size: usize) -> Self {
        Self {
            #[cfg(any(feature = "wasmer_sys_dev", feature = "wasmer_sys_prod"))]
            entries: HashMap::new(),
            max_size,
            current_size: 0,
            access_counter: 0,
        }
    }

    /// Get a module from the cache
    #[cfg(any(feature = "wasmer_sys_dev", feature = "wasmer_sys_prod"))]
    pub fn get(&self, key: &[u8]) -> Option<&Module> {
        self.entries.get(key).map(|e| &e.module)
    }

    /// Insert a module into the cache
    #[cfg(any(feature = "wasmer_sys_dev", feature = "wasmer_sys_prod"))]
    pub fn insert(&mut self, key: Vec<u8>, module: Module) {
        // Estimate size (rough approximation)
        let size = key.len() + 1024; // Module size is hard to determine

        // Evict if needed
        while self.current_size + size > self.max_size && !self.entries.is_empty() {
            self.evict_oldest();
        }

        self.access_counter += 1;
        self.entries.insert(key, CacheEntry {
            module,
            size,
            last_access: self.access_counter,
        });
        self.current_size += size;
    }

    /// Evict the oldest entry
    #[cfg(any(feature = "wasmer_sys_dev", feature = "wasmer_sys_prod"))]
    fn evict_oldest(&mut self) {
        if let Some((key, entry)) = self.entries
            .iter()
            .min_by_key(|(_, e)| e.last_access)
            .map(|(k, e)| (k.clone(), e.size))
        {
            self.entries.remove(&key);
            self.current_size -= entry;
        }
    }

    /// Clear the cache
    pub fn clear(&mut self) {
        #[cfg(any(feature = "wasmer_sys_dev", feature = "wasmer_sys_prod"))]
        self.entries.clear();
        self.current_size = 0;
    }

    /// Get the number of cached modules
    #[cfg(any(feature = "wasmer_sys_dev", feature = "wasmer_sys_prod"))]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if cache is empty
    #[cfg(any(feature = "wasmer_sys_dev", feature = "wasmer_sys_prod"))]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_creation() {
        let cache = ModuleCache::new(1024 * 1024);
        assert_eq!(cache.max_size, 1024 * 1024);
    }
}
