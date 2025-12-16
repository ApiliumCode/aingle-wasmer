//! WASM engine configuration and management

use crate::{DEFAULT_METERING_LIMIT, HostError};
use crate::module::ModuleCache;
use std::sync::Arc;

#[cfg(feature = "wasmer_sys_dev")]
use wasmer::sys::Cranelift;

#[cfg(feature = "wasmer_sys_prod")]
use wasmer::sys::LLVM;

#[cfg(any(feature = "wasmer_sys_dev", feature = "wasmer_sys_prod"))]
use wasmer::{Engine, Module};

#[cfg(any(feature = "wasmer_sys_dev", feature = "wasmer_sys_prod"))]
use wasmer_middlewares::Metering;

/// Configuration for the WASM engine
#[derive(Clone, Debug)]
pub struct EngineConfig {
    /// Maximum operations before timeout
    pub metering_limit: u64,
    /// Enable NaN canonicalization for determinism
    pub canonicalize_nans: bool,
    /// Optional cache directory path
    pub cache_path: Option<std::path::PathBuf>,
    /// Static memory bound (for iOS compatibility)
    pub static_memory_bound: u32,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            metering_limit: DEFAULT_METERING_LIMIT,
            canonicalize_nans: true,
            cache_path: None,
            static_memory_bound: 0x4000,
        }
    }
}

/// WASM execution engine
pub struct WasmEngine {
    #[cfg(any(feature = "wasmer_sys_dev", feature = "wasmer_sys_prod"))]
    inner: Engine,
    config: EngineConfig,
    cache: Arc<ModuleCache>,
}

impl WasmEngine {
    /// Create a new WASM engine with the given configuration
    #[cfg(any(feature = "wasmer_sys_dev", feature = "wasmer_sys_prod"))]
    pub fn new(config: EngineConfig) -> Result<Self, HostError> {
        use wasmer::sys::{BaseTunables, CompilerConfig, NativeEngineExt};
        use std::sync::Arc as StdArc;

        let cost_function = |_: &wasmer::wasmparser::Operator| -> u64 { 1 };
        let metering = StdArc::new(Metering::new(config.metering_limit, cost_function));

        #[cfg(feature = "wasmer_sys_dev")]
        let mut compiler = Cranelift::default();

        #[cfg(feature = "wasmer_sys_prod")]
        let mut compiler = LLVM::default();

        if config.canonicalize_nans {
            compiler.canonicalize_nans(true);
        }
        compiler.push_middleware(metering);

        let mut engine = Engine::from(compiler);

        // iOS compatibility tunables
        engine.set_tunables(BaseTunables {
            static_memory_bound: config.static_memory_bound.into(),
            static_memory_offset_guard_size: 0x1_0000,
            dynamic_memory_offset_guard_size: 0x1_0000,
        });

        Ok(Self {
            inner: engine,
            config: config.clone(),
            cache: Arc::new(ModuleCache::new(config.cache_path.clone())),
        })
    }

    /// Compile WASM bytes into a module
    #[cfg(any(feature = "wasmer_sys_dev", feature = "wasmer_sys_prod"))]
    pub fn compile(&self, wasm: &[u8]) -> Result<Module, HostError> {
        Module::new(&self.inner, wasm)
            .map_err(|e| HostError::Compilation(e.to_string()))
    }

    /// Compile with caching using a 32-byte key
    #[cfg(any(feature = "wasmer_sys_dev", feature = "wasmer_sys_prod"))]
    pub fn compile_cached(&self, key: [u8; 32], wasm: &[u8]) -> Result<Arc<Module>, HostError> {
        self.cache.get(key, wasm)
    }

    /// Get a reference to the inner Wasmer engine
    #[cfg(any(feature = "wasmer_sys_dev", feature = "wasmer_sys_prod"))]
    pub fn inner(&self) -> &Engine {
        &self.inner
    }

    /// Get the configuration
    pub fn config(&self) -> &EngineConfig {
        &self.config
    }

    /// Clear the module cache
    #[cfg(any(feature = "wasmer_sys_dev", feature = "wasmer_sys_prod"))]
    pub fn clear_cache(&self) {
        self.cache.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(any(feature = "wasmer_sys_dev", feature = "wasmer_sys_prod"))]
    fn test_engine_creation() {
        let config = EngineConfig::default();
        let engine = WasmEngine::new(config).unwrap();
        assert!(engine.config().canonicalize_nans);
    }
}
