//! Authentication and request signing utilities.

use std::time::{SystemTime, UNIX_EPOCH};

use rand::Rng;

/// Generate the cyrb53 hash used for Fansly request signing.
///
/// This is a port of the JavaScript cyrb53 hash function used by Fansly.
/// Uses 32-bit integer overflow semantics to match JavaScript behavior.
pub fn cyrb53(input: &str, seed: i32) -> u64 {
    // Use i32 for 32-bit signed integer overflow behavior
    let mut h1: i32 = (0xdeadbeef_u32 as i32) ^ seed;
    let mut h2: i32 = (0x41c6ce57_u32 as i32) ^ seed;

    for ch in input.chars() {
        let code = ch as i32;
        h1 = imul32(h1 ^ code, 2654435761_u32 as i32);
        h2 = imul32(h2 ^ code, 1597334677_u32 as i32);
    }

    h1 = imul32(h1 ^ rshift32(h1, 16), 2246822507_u32 as i32);
    h1 ^= imul32(h2 ^ rshift32(h2, 13), 3266489909_u32 as i32);
    h2 = imul32(h2 ^ rshift32(h2, 16), 2246822507_u32 as i32);
    h2 ^= imul32(h1 ^ rshift32(h1, 13), 3266489909_u32 as i32);

    // 4294967296 * (2097151 & h2) + (h1 >>> 0)
    let h2_masked = (h2 as u32) & 0x1FFFFF;
    let h1_unsigned = h1 as u32;

    (h2_masked as u64) * 4294967296 + (h1_unsigned as u64)
}

/// 32-bit signed integer multiplication with overflow (matches JavaScript Math.imul)
fn imul32(a: i32, b: i32) -> i32 {
    a.wrapping_mul(b)
}

/// Unsigned right shift for 32-bit integers (matches JavaScript >>> operator)
fn rshift32(value: i32, bits: i32) -> i32 {
    ((value as u32) >> bits) as i32
}

/// Generate the check hash for a request.
///
/// Format: cyrb53(check_key + "_" + url_path + "_" + device_id)
pub fn generate_check_hash(check_key: &str, url_path: &str, device_id: &str) -> String {
    let input = format!("{}_{}_{}",  check_key, url_path, device_id);
    let hash = cyrb53(&input, 0);
    // Convert to hex without leading zeros
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
    fn test_cyrb53_known_values() {
        // Test cases from Python implementation
        assert_eq!(cyrb53("a", 0), 7929297801672961);
        assert_eq!(cyrb53("b", 0), 8684336938537663);
        assert_eq!(cyrb53("revenge", 0), 4051478007546757);
    }

    #[test]
    fn test_cyrb53_consistency() {
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
    fn test_generate_check_hash_format() {
        // Verify the hash is generated from the correct input format
        let hash1 = generate_check_hash("key", "/path", "device");
        let hash2 = generate_check_hash("key", "/path", "device");
        assert_eq!(hash1, hash2); // Same inputs should produce same hash

        let hash3 = generate_check_hash("key", "/different", "device");
        assert_ne!(hash1, hash3); // Different path should produce different hash
    }

    #[test]
    fn test_device_id_expired_none() {
        // None timestamp should always be considered expired
        assert!(is_device_id_expired(None));
    }

    #[test]
    fn test_device_id_expired_old() {
        // Timestamp from 200 minutes ago should be expired (> 180 min)
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;
        let old_timestamp = now - (200 * 60 * 1000); // 200 minutes ago
        assert!(is_device_id_expired(Some(old_timestamp)));
    }

    #[test]
    fn test_device_id_not_expired() {
        // Timestamp from 10 minutes ago should not be expired
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;
        let recent_timestamp = now - (10 * 60 * 1000); // 10 minutes ago
        assert!(!is_device_id_expired(Some(recent_timestamp)));
    }

    #[test]
    fn test_device_id_boundary() {
        // Exactly 180 minutes should not be expired (boundary test)
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;
        let boundary_timestamp = now - (180 * 60 * 1000); // Exactly 180 minutes
        assert!(!is_device_id_expired(Some(boundary_timestamp)));

        // 181 minutes should be expired
        let expired_timestamp = now - (181 * 60 * 1000);
        assert!(is_device_id_expired(Some(expired_timestamp)));
    }

    #[test]
    fn test_client_timestamp_has_offset() {
        // Client timestamp should be in the future (has random offset)
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;
        let client_ts = get_client_timestamp();

        // Should be at least 5000ms ahead
        assert!(client_ts >= now + 5000);
        // Should be at most 10000ms ahead
        assert!(client_ts <= now + 10001);
    }
}
