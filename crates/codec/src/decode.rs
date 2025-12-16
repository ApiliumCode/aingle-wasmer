//! Decoding functionality

use aingle_wasm_types::{
    EnvelopeHeader, EnvelopeError, WasmError, DeserializeError,
};
use crate::checksum::compute_checksum;

/// Decoder for WASM messages
pub struct Decoder<'a> {
    buffer: &'a [u8],
    position: usize,
}

impl<'a> Decoder<'a> {
    /// Create a new decoder
    pub fn new(buffer: &'a [u8]) -> Self {
        Self { buffer, position: 0 }
    }

    /// Get remaining bytes
    pub fn remaining(&self) -> usize {
        self.buffer.len() - self.position
    }

    /// Get current position
    pub fn position(&self) -> usize {
        self.position
    }

    /// Read bytes from the buffer
    pub fn read_bytes(&mut self, len: usize) -> Result<&'a [u8], WasmError> {
        if self.remaining() < len {
            return Err(WasmError::Deserialize(DeserializeError::UnexpectedEof));
        }
        let bytes = &self.buffer[self.position..self.position + len];
        self.position += len;
        Ok(bytes)
    }

    /// Read a u8
    pub fn read_u8(&mut self) -> Result<u8, WasmError> {
        let bytes = self.read_bytes(1)?;
        Ok(bytes[0])
    }

    /// Read a u16 (little-endian)
    pub fn read_u16(&mut self) -> Result<u16, WasmError> {
        let bytes = self.read_bytes(2)?;
        Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
    }

    /// Read a u32 (little-endian)
    pub fn read_u32(&mut self) -> Result<u32, WasmError> {
        let bytes = self.read_bytes(4)?;
        Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    /// Read a u64 (little-endian)
    pub fn read_u64(&mut self) -> Result<u64, WasmError> {
        let bytes = self.read_bytes(8)?;
        Ok(u64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3],
            bytes[4], bytes[5], bytes[6], bytes[7],
        ]))
    }

    /// Get remaining buffer as slice
    pub fn remaining_slice(&self) -> &'a [u8] {
        &self.buffer[self.position..]
    }
}

/// Decoded envelope with payload reference
pub struct DecodedEnvelope<'a> {
    /// The envelope header
    pub header: EnvelopeHeader,
    /// The payload bytes (zero-copy reference)
    pub payload: &'a [u8],
}

/// Decode an envelope from a buffer
pub fn decode_envelope(buffer: &[u8]) -> Result<DecodedEnvelope<'_>, WasmError> {
    if buffer.len() < EnvelopeHeader::SIZE {
        return Err(WasmError::Deserialize(DeserializeError::UnexpectedEof));
    }

    let header_bytes: [u8; EnvelopeHeader::SIZE] = buffer[..EnvelopeHeader::SIZE]
        .try_into()
        .map_err(|_| WasmError::Deserialize(DeserializeError::InvalidFormat))?;

    let header = EnvelopeHeader::from_bytes(&header_bytes);

    // Validate header
    header.validate().map_err(|e| {
        WasmError::Deserialize(match e {
            EnvelopeError::InvalidMagic(_) => DeserializeError::InvalidFormat,
            EnvelopeError::UnsupportedVersion(_) => DeserializeError::InvalidFormat,
            _ => DeserializeError::InvalidFormat,
        })
    })?;

    let payload_start = EnvelopeHeader::SIZE;
    let payload_end = payload_start + header.payload_len as usize;

    if buffer.len() < payload_end {
        return Err(WasmError::Deserialize(DeserializeError::UnexpectedEof));
    }

    let payload = &buffer[payload_start..payload_end];

    // Verify checksum
    let actual_checksum = compute_checksum(payload);
    if actual_checksum != header.checksum {
        return Err(WasmError::Deserialize(DeserializeError::InvalidFormat));
    }

    Ok(DecodedEnvelope { header, payload })
}

/// Decode payload directly (without envelope) - for compatibility
pub fn decode_raw(buffer: &[u8]) -> &[u8] {
    buffer
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encode::encode_with_envelope;

    #[test]
    fn test_decoder_basic() {
        let buf = [0x42, 0x34, 0x12, 0xEF, 0xBE, 0xAD, 0xDE];
        let mut dec = Decoder::new(&buf);

        assert_eq!(dec.read_u8().unwrap(), 0x42);
        assert_eq!(dec.read_u16().unwrap(), 0x1234);
        assert_eq!(dec.read_u32().unwrap(), 0xDEADBEEF);
        assert_eq!(dec.remaining(), 0);
    }

    #[test]
    fn test_roundtrip() {
        let payload = b"test payload data";
        let mut buffer = [0u8; 128];

        let len = encode_with_envelope(payload, 0, &mut buffer).unwrap();
        let decoded = decode_envelope(&buffer[..len]).unwrap();

        assert_eq!(decoded.payload, payload);
        assert!(!decoded.header.is_error());
    }

    #[test]
    fn test_checksum_validation() {
        let payload = b"test";
        let mut buffer = [0u8; 64];

        let len = encode_with_envelope(payload, 0, &mut buffer).unwrap();

        // Corrupt the payload
        buffer[EnvelopeHeader::SIZE] ^= 0xFF;

        let result = decode_envelope(&buffer[..len]);
        assert!(result.is_err());
    }
}
