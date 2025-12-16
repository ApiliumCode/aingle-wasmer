//! Encoding functionality

use crate::checksum::compute_checksum;
use aingle_wasmer_common::{EnvelopeHeader, WasmError, WasmSlice};

/// Encoder for WASM messages
pub struct Encoder<'a> {
    buffer: &'a mut [u8],
    position: usize,
}

impl<'a> Encoder<'a> {
    /// Create a new encoder with the given buffer
    pub fn new(buffer: &'a mut [u8]) -> Self {
        Self {
            buffer,
            position: 0,
        }
    }

    /// Get remaining capacity
    pub fn remaining(&self) -> usize {
        self.buffer.len() - self.position
    }

    /// Get current position
    pub fn position(&self) -> usize {
        self.position
    }

    /// Write bytes to the buffer
    pub fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), WasmError> {
        if self.remaining() < bytes.len() {
            return Err(WasmError::Serialize(
                aingle_wasmer_common::SerializeError::BufferTooSmall {
                    needed: bytes.len(),
                    available: self.remaining(),
                },
            ));
        }
        self.buffer[self.position..self.position + bytes.len()].copy_from_slice(bytes);
        self.position += bytes.len();
        Ok(())
    }

    /// Write a u8
    pub fn write_u8(&mut self, value: u8) -> Result<(), WasmError> {
        self.write_bytes(&[value])
    }

    /// Write a u16 (little-endian)
    pub fn write_u16(&mut self, value: u16) -> Result<(), WasmError> {
        self.write_bytes(&value.to_le_bytes())
    }

    /// Write a u32 (little-endian)
    pub fn write_u32(&mut self, value: u32) -> Result<(), WasmError> {
        self.write_bytes(&value.to_le_bytes())
    }

    /// Write a u64 (little-endian)
    pub fn write_u64(&mut self, value: u64) -> Result<(), WasmError> {
        self.write_bytes(&value.to_le_bytes())
    }

    /// Finish encoding and return the used portion of the buffer
    pub fn finish(self) -> &'a [u8] {
        &self.buffer[..self.position]
    }
}

/// Encode a payload with envelope header
pub fn encode_with_envelope(
    payload: &[u8],
    flags: u8,
    output: &mut [u8],
) -> Result<usize, WasmError> {
    let total_size = EnvelopeHeader::SIZE + payload.len();

    if output.len() < total_size {
        return Err(WasmError::Serialize(
            aingle_wasmer_common::SerializeError::BufferTooSmall {
                needed: total_size,
                available: output.len(),
            },
        ));
    }

    let checksum = compute_checksum(payload);
    let header = EnvelopeHeader::new(payload.len() as u32, checksum, flags);

    let mut encoder = Encoder::new(output);
    encoder.write_bytes(&header.to_bytes())?;
    encoder.write_bytes(payload)?;

    Ok(encoder.position())
}

/// Encode data to a WasmSlice (for guest use)
///
/// Returns the slice pointing to the encoded data in the provided buffer.
pub fn encode_to_slice(
    payload: &[u8],
    buffer: &mut [u8],
    buffer_ptr: u32,
) -> Result<WasmSlice, WasmError> {
    let len = encode_with_envelope(payload, 0, buffer)?;
    Ok(WasmSlice::new(buffer_ptr, len as u32))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encoder_basic() {
        let mut buf = [0u8; 32];
        let mut enc = Encoder::new(&mut buf);

        enc.write_u8(0x42).unwrap();
        enc.write_u16(0x1234).unwrap();
        enc.write_u32(0xDEADBEEF).unwrap();

        let result = enc.finish();
        assert_eq!(result.len(), 7);
        assert_eq!(result[0], 0x42);
    }

    #[test]
    fn test_encode_with_envelope() {
        let payload = b"hello world";
        let mut output = [0u8; 64];

        let len = encode_with_envelope(payload, 0, &mut output).unwrap();
        assert_eq!(len, EnvelopeHeader::SIZE + payload.len());

        // Check magic bytes
        assert_eq!(output[0], 0x49); // 'I'
        assert_eq!(output[1], 0x41); // 'A'
    }
}
