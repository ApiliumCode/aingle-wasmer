//! Checksum computation for data integrity

/// Compute CRC32 checksum of data
pub fn compute_checksum(data: &[u8]) -> u32 {
    crc32fast::hash(data)
}

/// Verify checksum matches expected value
pub fn verify_checksum(data: &[u8], expected: u32) -> bool {
    compute_checksum(data) == expected
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checksum() {
        let data = b"hello world";
        let checksum = compute_checksum(data);

        assert!(verify_checksum(data, checksum));
        assert!(!verify_checksum(data, checksum + 1));
    }

    #[test]
    fn test_empty_checksum() {
        let checksum = compute_checksum(&[]);
        assert!(verify_checksum(&[], checksum));
    }
}
