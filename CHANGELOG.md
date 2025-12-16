# Changelog

All notable changes to aingle-wasmer will be documented in this file.

## [0.1.0] - 2024-12-16

### Added
- Complete redesign of WASM integration layer
- New envelope protocol with versioning and CRC32 checksums
- Arena-based memory allocation using bumpalo
- Four-crate architecture for clean separation

### Crates
- `aingle_wasm_types` - no_std compatible core types
  - EnvelopeHeader with magic bytes, version, flags
  - WasmSlice, WasmResult, WasmRef types
  - WasmEncode/WasmDecode traits
  - Comprehensive error types

- `aingle_wasm_codec` - Encode/decode with integrity
  - CRC32 checksum for all payloads
  - Streaming encoder/decoder
  - Zero-copy where possible

- `aingle_wasm_guest` - Guest-side utilities
  - Arena allocator (bumpalo)
  - host_args, return_ok, return_err helpers
  - host_externs! macro for imports

- `aingle_wasm_host` - Wasmer 6.0 integration
  - WasmEngine with configurable metering
  - Module caching with LRU eviction
  - WasmInstance for function calls

### Protocol
- 12-byte envelope header
- Magic bytes: 0x4149 ("AI")
- Protocol version: 1
- Flags for compression, encryption, errors
- CRC32 integrity checking

### Improvements over previous design
- Versioned wire format enables protocol evolution
- Checksums detect data corruption
- Arena allocation reduces fragmentation
- no_std types for embedded targets
- Cleaner crate separation

### Dependencies
- wasmer 6.0.0
- wasmer-middlewares 6.0.0
- bumpalo 3.16
- crc32fast 1.4
- parking_lot 0.12
- thiserror 2
