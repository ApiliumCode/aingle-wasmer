# aingle-wasmer

WASM host/guest integration library for AIngle - optimized for IoT & AI workloads.

[![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)

## Overview

AIngle WASM provides a high-performance, memory-efficient runtime for executing WebAssembly code within the AIngle platform. Designed specifically for IoT devices and AI agents with:

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
| `aingle_wasm_types` | Core types, envelope, traits | ✅ |
| `aingle_wasm_codec` | Encode/decode with CRC32 | ❌ |
| `aingle_wasm_guest` | Guest utilities + arena allocator | ❌ |
| `aingle_wasm_host` | Wasmer 6.0 execution engine | ❌ |

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
    // Decode input from host
    let input = match host_args(ptr, len) {
        Ok(bytes) => bytes,
        Err(e) => return return_err(b"failed to decode input"),
    };

    // Process...
    let output = process(input);

    // Return result to host
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

- `0x01` - Compressed (LZ4)
- `0x02` - Encrypted
- `0x04` - Expects response
- `0x08` - Is error response

## Comparison with Previous Design

| Aspect | holochain-wasmer | aingle-wasmer |
|--------|------------------|---------------|
| Protocol | ptr+len (raw) | Versioned envelope |
| Checksum | None | CRC32 |
| Memory | leak/deallocate | Arena (bumpalo) |
| Serialization | MessagePack | Custom + CRC32 |
| no_std types | No | Yes |

## Features

```toml
[features]
default = ["wasmer_sys_dev"]
wasmer_sys_dev = ["wasmer/cranelift"]  # Fast compile, good for dev
wasmer_sys_prod = ["wasmer/llvm"]      # Optimized runtime
```

## Configuration

```rust
EngineConfig {
    // Max operations before timeout (100B default)
    metering_limit: 100_000_000_000,
    // Canonicalize NaN for determinism
    canonicalize_nans: true,
    // Module cache size (256MB default)
    cache_size: 256 * 1024 * 1024,
    // iOS compatibility (0x4000 default)
    static_memory_bound: 0x4000,
}
```

## Testing

```bash
cargo test --workspace
```

## Benchmarks

```bash
cargo bench
```

## License

Apache-2.0

## Contributing

Contributions welcome! Please read our contributing guidelines first.
