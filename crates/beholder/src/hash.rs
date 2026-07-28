//! Content addressing helpers.
//!
//! Every hash beholder stores is a function of bytes alone, so a stored result
//! validates identically on any machine and in any checkout location.

use sha2::{Digest, Sha256};

/// Lowercase hex SHA-256 of the given bytes.
pub fn hex_sha256(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

/// Content address of a file, as lowercase hex.
///
/// This is git's blob framing — `blob <len>\0` followed by the contents — under
/// SHA-256, so the value is a function of the bytes alone and is independent of
/// the repository's own object format.
pub fn content_hash(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(format!("blob {}\0", bytes.len()).as_bytes());
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hashes_are_stable() {
        assert_eq!(hex_sha256(b"beholder"), hex_sha256(b"beholder"));
        assert_ne!(hex_sha256(b"beholder"), hex_sha256(b"beholders"));
    }

    #[test]
    fn content_hash_includes_the_length_header() {
        assert_ne!(content_hash(b"beholder"), hex_sha256(b"beholder"));
    }
}
