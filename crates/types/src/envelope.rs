//! Envelope format for AIngle WASM communication
//!
//! The envelope provides a versioned, checksummed wire format for
//! host↔guest communication that supports future protocol evolution.

use crate::{MAGIC, PROTOCOL_VERSION};

/// Flags for envelope options
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EnvelopeFlags {
    /// No special flags
    None = 0,
    /// Payload is compressed (LZ4)
    Compressed = 1 << 0,
    /// Payload is encrypted
    Encrypted = 1 << 1,
    /// Response expected (for async calls)
    ExpectsResponse = 1 << 2,
    /// This is an error response
    IsError = 1 << 3,
}

impl EnvelopeFlags {
    /// Check if a flag is set in a byte
    #[inline]
    pub fn is_set(self, flags: u8) -> bool {
        flags & (self as u8) != 0
    }

    /// Combine multiple flags
    #[inline]
    pub fn combine(flags: &[EnvelopeFlags]) -> u8 {
        flags.iter().fold(0u8, |acc, f| acc | (*f as u8))
    }
}

/// Header for WASM envelope messages
///
/// Fixed 12-byte header that prefixes all host↔guest messages.
/// Designed for efficient parsing and zero-copy operations.
///
/// ```text
/// +-------+-------+-------+-------+
/// | magic (2B)    | ver   | flags |
/// +-------+-------+-------+-------+
/// | payload_len (4B)              |
/// +-------+-------+-------+-------+
/// | checksum (4B)                 |
/// +-------+-------+-------+-------+
/// | payload (variable)...         |
/// +-------+-------+-------+-------+
/// ```
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct EnvelopeHeader {
    /// Magic bytes: 0x4149 ("AI")
    pub magic: u16,
    /// Protocol version
    pub version: u8,
    /// Flags (compression, encryption, etc.)
    pub flags: u8,
    /// Length of payload in bytes
    pub payload_len: u32,
    /// CRC32 checksum of payload
    pub checksum: u32,
}

impl EnvelopeHeader {
    /// Size of the header in bytes
    pub const SIZE: usize = 12;

    /// Create a new envelope header
    #[inline]
    pub const fn new(payload_len: u32, checksum: u32, flags: u8) -> Self {
        Self {
            magic: MAGIC,
            version: PROTOCOL_VERSION,
            flags,
            payload_len,
            checksum,
        }
    }

    /// Validate the header
    #[inline]
    pub fn validate(&self) -> Result<(), EnvelopeError> {
        if self.magic != MAGIC {
            return Err(EnvelopeError::InvalidMagic(self.magic));
        }
        if self.version > PROTOCOL_VERSION {
            return Err(EnvelopeError::UnsupportedVersion(self.version));
        }
        Ok(())
    }

    /// Convert header to bytes
    #[inline]
    pub fn to_bytes(&self) -> [u8; Self::SIZE] {
        let mut bytes = [0u8; Self::SIZE];
        bytes[0..2].copy_from_slice(&self.magic.to_le_bytes());
        bytes[2] = self.version;
        bytes[3] = self.flags;
        bytes[4..8].copy_from_slice(&self.payload_len.to_le_bytes());
        bytes[8..12].copy_from_slice(&self.checksum.to_le_bytes());
        bytes
    }

    /// Parse header from bytes
    #[inline]
    pub fn from_bytes(bytes: &[u8; Self::SIZE]) -> Self {
        Self {
            magic: u16::from_le_bytes([bytes[0], bytes[1]]),
            version: bytes[2],
            flags: bytes[3],
            payload_len: u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]),
            checksum: u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]),
        }
    }

    /// Check if error flag is set
    #[inline]
    pub fn is_error(&self) -> bool {
        EnvelopeFlags::IsError.is_set(self.flags)
    }

    /// Check if compressed flag is set
    #[inline]
    pub fn is_compressed(&self) -> bool {
        EnvelopeFlags::Compressed.is_set(self.flags)
    }
}

/// Errors that can occur when parsing envelopes
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EnvelopeError {
    /// Invalid magic bytes
    InvalidMagic(u16),
    /// Unsupported protocol version
    UnsupportedVersion(u8),
    /// Checksum mismatch
    ChecksumMismatch { expected: u32, actual: u32 },
    /// Buffer too small
    BufferTooSmall { needed: usize, available: usize },
    /// Payload too large
    PayloadTooLarge(u32),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_header_roundtrip() {
        let header = EnvelopeHeader::new(1024, 0xDEADBEEF, 0);
        let bytes = header.to_bytes();
        let parsed = EnvelopeHeader::from_bytes(&bytes);

        // Copy fields from packed struct before comparing
        let magic = parsed.magic;
        let version = parsed.version;
        let payload_len = parsed.payload_len;
        let checksum = parsed.checksum;

        assert_eq!(magic, MAGIC);
        assert_eq!(version, PROTOCOL_VERSION);
        assert_eq!(payload_len, 1024);
        assert_eq!(checksum, 0xDEADBEEF);
    }

    #[test]
    fn test_header_validation() {
        let valid = EnvelopeHeader::new(100, 0, 0);
        assert!(valid.validate().is_ok());

        let invalid = EnvelopeHeader {
            magic: 0xFFFF,
            version: 1,
            flags: 0,
            payload_len: 0,
            checksum: 0,
        };
        assert!(matches!(invalid.validate(), Err(EnvelopeError::InvalidMagic(0xFFFF))));
    }

    #[test]
    fn test_flags() {
        let flags = EnvelopeFlags::combine(&[
            EnvelopeFlags::Compressed,
            EnvelopeFlags::IsError,
        ]);

        assert!(EnvelopeFlags::Compressed.is_set(flags));
        assert!(EnvelopeFlags::IsError.is_set(flags));
        assert!(!EnvelopeFlags::Encrypted.is_set(flags));
    }
}
