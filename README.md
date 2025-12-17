<p align="center">
  <img src="https://raw.githubusercontent.com/ApiliumCode/aingle/main/assets/aingle.svg" alt="AIngle Logo" width="200"/>
</p>

<h1 align="center">aingle-wasmer</h1>

<p align="center">
  <strong>WASM host/guest integration library for AIngle - optimized for IoT & AI</strong>
</p>

<p align="center">
  <a href="https://crates.io/crates/aingle_wasm_host"><img src="https://img.shields.io/crates/v/aingle_wasm_host.svg" alt="Crates.io"/></a>
  <a href="https://docs.rs/aingle_wasm_host"><img src="https://docs.rs/aingle_wasm_host/badge.svg" alt="Documentation"/></a>
  <a href="https://github.com/ApiliumCode/aingle-wasmer/blob/main/LICENSE"><img src="https://img.shields.io/badge/license-Apache--2.0-blue.svg" alt="License"/></a>
  <a href="https://github.com/ApiliumCode/aingle-wasmer/actions"><img src="https://github.com/ApiliumCode/aingle-wasmer/workflows/CI/badge.svg" alt="CI Status"/></a>
</p>

---

## Overview

High-performance, memory-efficient WASM runtime for executing WebAssembly code within the AIngle platform. Designed specifically for IoT devices and AI agents with:

- **Zero-copy envelope protocol** with versioning and checksums
- **Arena-based memory allocation** for minimal fragmentation
- **Metering middleware** for resource control
- **no_std compatible** types for embedded targets

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    AIngle WASM Stack                        │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────┐  │
│  │    types     │  │    codec     │  │  guest / host    │  │
│  │  (no_std)    │──│  (checksum)  │──│  (wasmer 6.0)    │  │
│  └──────────────┘  └──────────────┘  └──────────────────┘  │
│                                                              │
│  Envelope Protocol:                                         │
│  ┌─────┬─────┬─────┬─────────────┬──────────┬───────────┐  │
│  │ AI  │ ver │flags│ payload_len │ checksum │ payload   │  │
│  │(2B) │(1B) │(1B) │    (4B)     │   (4B)   │ (var)     │  │
│  └─────┴─────┴─────┴─────────────┴──────────┴───────────┘  │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

## Crates

| Crate | Description | no_std |
|-------|-------------|--------|
| `aingle_wasm_types` | Core types, envelope, traits | Yes |
| `aingle_wasm_codec` | Encode/decode with CRC32 | No |
| `aingle_wasm_guest` | Guest utilities + arena allocator | No |
| `aingle_wasm_host` | Wasmer 6.0 execution engine | No |

## Quick Start

### Host Side (Conductor)

```rust
use aingle_wasm_host::prelude::*;

fn main() -> Result<(), HostError> {
    // Create engine with metering
    let config = EngineConfig {
        metering_limit: 1_000_000_000,
        ..Default::default()
    };
    let engine = WasmEngine::new(config)?;

    // Compile and cache module
    let wasm_bytes = include_bytes!("my_module.wasm");
    let module = engine.compile_cached(b"my_module", wasm_bytes)?;

    // Create instance and call function
    let mut instance = WasmInstance::new(&engine, &module)?;
    let result = instance.call_raw("my_function", b"input data")?;

    Ok(())
}
```

### Guest Side (WASM Module)

```rust
use aingle_wasm_guest::prelude::*;

// Define host functions to import
host_externs!(__my_host_fn);

// Entry point function
#[no_mangle]
pub extern "C" fn my_function(ptr: u32, len: u32) -> u64 {
    let input = match host_args(ptr, len) {
        Ok(bytes) => bytes,
        Err(e) => return return_err(b"failed to decode input"),
    };

    let output = process(input);
    return_ok(&output)
}
```

## Protocol Features

### Envelope Header (12 bytes)

| Field | Size | Description |
|-------|------|-------------|
| magic | 2B | `0x4149` ("AI") |
| version | 1B | Protocol version (currently 1) |
| flags | 1B | Compressed, encrypted, error, etc. |
| payload_len | 4B | Payload size in bytes |
| checksum | 4B | CRC32 of payload |

### Flags

| Value | Meaning |
|-------|---------|
| `0x01` | Compressed (LZ4) |
| `0x02` | Encrypted |
| `0x04` | Expects response |
| `0x08` | Is error response |

## Configuration

```rust
EngineConfig {
    metering_limit: 100_000_000_000,  // Max operations
    canonicalize_nans: true,          // Deterministic NaN
    cache_size: 256 * 1024 * 1024,    // 256MB cache
    static_memory_bound: 0x4000,      // iOS compatibility
}
```

## Features

```toml
[features]
default = ["wasmer_sys_dev"]
wasmer_sys_dev = ["wasmer/cranelift"]  # Fast compile
wasmer_sys_prod = ["wasmer/llvm"]      # Optimized runtime
```

## Testing

```bash
cargo test --workspace
cargo bench
```

## Part of AIngle

This crate is part of the [AIngle](https://github.com/ApiliumCode/aingle) ecosystem - a Semantic DAG framework for IoT and distributed AI applications.

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for details.

---

<p align="center">
  <sub>Maintained by <a href="https://apilium.com">Apilium Technologies</a> - Tallinn, Estonia</sub>
</p>
