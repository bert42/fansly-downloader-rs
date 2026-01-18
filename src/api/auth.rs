//! Authentication and request signing utilities.

use std::time::{SystemTime, UNIX_EPOCH};

use rand::Rng;

/// Generate the cyrb53 hash used for Fansly request signing.
///
/// This is a port of the JavaScript cyrb53 hash function used by Fansly.
pub fn cyrb53(input: &str, seed: u64) -> u64 {
    let mut h1: u64 = 0xdeadbeef ^ seed;
    let mut h2: u64 = 0x41c6ce57 ^ seed;

    for ch in input.chars() {
        let code = ch as u64;
        h1 = (h1 ^ code).wrapping_mul(2654435761);
        h2 = (h2 ^ code).wrapping_mul(1597334677);
    }

    h1 = ((h1 ^ (h1 >> 16)).wrapping_mul(2246822507)) ^ ((h2 ^ (h2 >> 13)).wrapping_mul(3266489909));
    h2 = ((h2 ^ (h2 >> 16)).wrapping_mul(2246822507)) ^ ((h1 ^ (h1 >> 13)).wrapping_mul(3266489909));

    (h2 << 32) | (h1 & 0xFFFFFFFF)
}

/// Generate the check hash for a request.
///
/// Format: cyrb53(check_key + "_" + url_path + "_" + device_id)
pub fn generate_check_hash(check_key: &str, url_path: &str, device_id: &str) -> String {
    let input = format!("{}_{}_{}",  check_key, url_path, device_id);
    let hash = cyrb53(&input, 0);
    format!("{:x}", hash)
}

/// Get current timestamp in milliseconds with random offset.
///
/// Adds a random offset of 5000-10000ms to the current time.
pub fn get_client_timestamp() -> i64 {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64;

    let mut rng = rand::thread_rng();
    let offset = rng.gen_range(5000..=10000);

    now + offset
}

/// Check if device ID has expired (older than 180 minutes).
pub fn is_device_id_expired(timestamp: Option<i64>) -> bool {
    let Some(timestamp) = timestamp else {
        return true;
    };

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64;

    let age_ms = now - timestamp;
    let max_age_ms = 180 * 60 * 1000; // 180 minutes in milliseconds

    age_ms > max_age_ms
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cyrb53_basic() {
        // Test that the hash produces consistent results
        let hash1 = cyrb53("test", 0);
        let hash2 = cyrb53("test", 0);
        assert_eq!(hash1, hash2);

        // Different inputs should produce different hashes
        let hash3 = cyrb53("other", 0);
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_cyrb53_with_seed() {
        let hash1 = cyrb53("test", 0);
        let hash2 = cyrb53("test", 42);
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_generate_check_hash() {
        let hash = generate_check_hash("checkkey", "/api/v1/test", "device123");
        assert!(!hash.is_empty());
        // Should be hexadecimal
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_is_device_id_expired() {
        // No timestamp should be expired
        assert!(is_device_id_expired(None));

        // Recent timestamp should not be expired
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;
        assert!(!is_device_id_expired(Some(now)));

        // Old timestamp should be expired
        let old = now - (200 * 60 * 1000); // 200 minutes ago
        assert!(is_device_id_expired(Some(old)));
    }
}
