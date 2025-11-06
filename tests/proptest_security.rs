//! Property-based testing for security functions
//!
//! This module contains comprehensive property-based tests for:
//! - Directory traversal attack prevention
//! - Path sanitization correctness
//! - Safe path joining verification
//!
//! Uses proptest to generate random attack patterns and verify security guarantees.

use backup_suite::security::{safe_join, sanitize_path_component};
use proptest::prelude::*;
use std::path::PathBuf;
use tempfile::TempDir;

// Configure proptest to use fewer cases for faster testing
// Can be overridden with PROPTEST_CASES environment variable
// Note: Use 10 cases for quick validation, can be increased for thorough testing
const PROPTEST_CASES: u32 = 10;

// Property test: Directory traversal attack patterns
//
// Verifies that safe_join correctly prevents directory traversal attacks
// by testing various combinations of parent directory references.
//
// Attack Patterns Tested:
// - Multiple `../` sequences (1-20 levels)
// - Random target path segments
// - Combined attack patterns
//
// Security Guarantee:
// The joined path MUST remain within the base directory after canonicalization.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(PROPTEST_CASES))]
    #[test]
    fn prop_directory_traversal_patterns(
        parent_count in 1usize..20,
        target_segments in prop::collection::vec(r"[a-z]{1,10}", 1..5)
    ) {
        let temp_dir = TempDir::new().unwrap();
        let base = temp_dir.path();

        // Construct attack path: ../../../target/path
        let attack_prefix = "../".repeat(parent_count);
        let attack_suffix = target_segments.join("/");
        let attack_path = PathBuf::from(format!("{}{}", attack_prefix, attack_suffix));

        let result = safe_join(base, &attack_path);

        // Verify the result is safe
        if let Ok(joined) = result {
            let canonical_base = base.canonicalize().unwrap();
            let joined_parent = joined.parent().unwrap_or(&joined);

            // If the joined path's parent exists, verify it's within base
            if joined_parent.exists() {
                let canonical_joined = joined_parent.canonicalize().unwrap();
                prop_assert!(
                    canonical_joined.starts_with(&canonical_base),
                    "Path traversal vulnerability detected: {:?} is outside {:?}",
                    canonical_joined,
                    canonical_base
                );
            } else {
                // If path doesn't exist yet, verify it starts with base
                prop_assert!(
                    joined.starts_with(base),
                    "Non-existent path {:?} should start with base {:?}",
                    joined,
                    base
                );
            }
        }
    }
}

// Property test: Path sanitization correctness
//
// Verifies that sanitize_path_component removes all dangerous characters
// and produces safe filesystem-compatible names.
//
// Security Guarantee:
// - Only alphanumeric, hyphen, and underscore characters allowed
// - No `..` sequences
// - No path separators (`/` or `\`)
proptest! {
    #![proptest_config(ProptestConfig::with_cases(PROPTEST_CASES))]
    #[test]
    fn prop_path_sanitization(
        input in r"[a-zA-Z0-9 \-_/\\\.\:;,]{1,50}"
    ) {
        let sanitized = sanitize_path_component(&input);

        // Verify only safe characters
        prop_assert!(
            sanitized.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_'),
            "Unsafe characters found in sanitized path: {:?}",
            sanitized
        );

        // Verify no directory traversal sequences
        prop_assert!(
            !sanitized.contains(".."),
            "Sanitized path contains '..' sequence: {:?}",
            sanitized
        );

        // Verify no path separators
        prop_assert!(
            !sanitized.contains('/'),
            "Sanitized path contains '/' separator: {:?}",
            sanitized
        );
        prop_assert!(
            !sanitized.contains('\\'),
            "Sanitized path contains '\\' separator: {:?}",
            sanitized
        );
    }
}

// Property test: Unicode path sanitization
//
// Verifies that sanitize_path_component handles Unicode correctly.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(PROPTEST_CASES))]
    #[test]
    fn prop_unicode_sanitization(
        input in r"[\u{0000}-\u{007F}\u{3042}-\u{3093}]{1,30}"
    ) {
        let sanitized = sanitize_path_component(&input);

        // Verify only safe characters (including Unicode alphanumerics)
        prop_assert!(
            sanitized.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_'),
            "Unsafe characters found in Unicode sanitized path: {:?}",
            sanitized
        );
    }
}

// Property test: Safe join with sanitized components
//
// Verifies that combining sanitize_path_component with safe_join
// provides defense-in-depth against path traversal.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(PROPTEST_CASES))]
    #[test]
    fn prop_sanitize_then_join(
        dangerous_name in r"[a-zA-Z0-9\.\./\\]{1,50}",
        subdir in r"[a-z]{1,10}"
    ) {
        let temp_dir = TempDir::new().unwrap();
        let base = temp_dir.path();

        // First sanitize the dangerous name
        let safe_name = sanitize_path_component(&dangerous_name);

        // Then join with base
        let child_path = PathBuf::from(&subdir).join(&safe_name);
        let result = safe_join(base, &child_path);

        prop_assert!(result.is_ok(), "safe_join failed for sanitized path");

        let joined = result.unwrap();
        prop_assert!(
            joined.starts_with(base),
            "Joined path {:?} is outside base {:?}",
            joined,
            base
        );
    }
}

// Property test: Absolute path rejection
//
// Verifies that absolute paths are correctly handled by safe_join.
#[cfg(unix)]
proptest! {
    #![proptest_config(ProptestConfig::with_cases(PROPTEST_CASES))]
    #[test]
    fn prop_absolute_path_unix(
        segments in prop::collection::vec(r"[a-z]{1,10}", 1..5)
    ) {
        let temp_dir = TempDir::new().unwrap();
        let base = temp_dir.path();

        // Create absolute path: /segment1/segment2/...
        let absolute_path = PathBuf::from("/").join(segments.join("/"));

        let result = safe_join(base, &absolute_path);

        // Absolute paths should either be rejected or canonicalized safely
        if let Ok(joined) = result {
            // If accepted, it must start with base
            prop_assert!(
                joined.starts_with(base),
                "Absolute path {:?} was joined but is outside base {:?}",
                joined,
                base
            );
        }
        // Otherwise, rejection is also acceptable
    }
}

// Property test: Windows-style path separators
//
// Verifies that Windows-style backslash separators are handled correctly.
#[cfg(windows)]
proptest! {
    #![proptest_config(ProptestConfig::with_cases(PROPTEST_CASES))]
    #[test]
    fn prop_windows_separators(
        segments in prop::collection::vec(r"[a-z]{1,10}", 1..5)
    ) {
        let temp_dir = TempDir::new().unwrap();
        let base = temp_dir.path();

        // Create path with Windows separators
        let windows_path = segments.join("\\");
        let child = PathBuf::from(&windows_path);

        let result = safe_join(base, &child);
        prop_assert!(result.is_ok(), "Windows path handling failed");

        let joined = result.unwrap();
        prop_assert!(
            joined.starts_with(base),
            "Windows path {:?} is outside base {:?}",
            joined,
            base
        );
    }
}

// Property test: Deep directory nesting
//
// Verifies that deeply nested directory structures are handled correctly.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(PROPTEST_CASES))]
    #[test]
    fn prop_deep_nesting(
        depth in 1usize..20,
    ) {
        let temp_dir = TempDir::new().unwrap();
        let base = temp_dir.path();

        // Create deeply nested path
        let segments: Vec<_> = (0..depth).map(|i| format!("dir{}", i)).collect();
        let deep_path = PathBuf::from(segments.join("/"));

        let result = safe_join(base, &deep_path);
        prop_assert!(result.is_ok(), "Deep nesting handling failed");

        let joined = result.unwrap();
        prop_assert!(
            joined.starts_with(base),
            "Deep nested path {:?} is outside base {:?}",
            joined,
            base
        );
    }
}

// Property test: Empty path handling
//
// Verifies that empty paths are handled gracefully.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(PROPTEST_CASES))]
    #[test]
    fn prop_empty_path(
        _dummy in 0..1usize
    ) {
        let temp_dir = TempDir::new().unwrap();
        let base = temp_dir.path();

        let empty_path = PathBuf::from("");
        let result = safe_join(base, &empty_path);

        prop_assert!(result.is_ok(), "Empty path handling failed");

        let joined = result.unwrap();
        prop_assert!(
            joined == base,
            "Empty path should result in base directory"
        );
    }
}

// Property test: Special character sanitization
//
// Verifies that special characters are correctly removed.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(PROPTEST_CASES))]
    #[test]
    fn prop_special_characters(
        prefix in r"[a-z]{1,5}",
        special in r"[:;<>!@#$%^&*+=]{1,10}",
        suffix in r"[a-z]{1,5}"
    ) {
        let input = format!("{}{}{}", prefix, special, suffix);
        let sanitized = sanitize_path_component(&input);

        // Verify no special characters remain
        prop_assert!(
            sanitized.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_'),
            "Special characters not removed: input={:?}, output={:?}",
            input,
            sanitized
        );

        // Verify prefix and suffix are preserved (if they were alphanumeric)
        prop_assert!(
            sanitized.contains(&prefix),
            "Alphanumeric prefix lost: input={:?}, output={:?}",
            input,
            sanitized
        );
        prop_assert!(
            sanitized.contains(&suffix),
            "Alphanumeric suffix lost: input={:?}, output={:?}",
            input,
            sanitized
        );
    }
}

// Property test: Null byte injection prevention
//
// Verifies that null bytes are correctly filtered out.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(PROPTEST_CASES))]
    #[test]
    fn prop_null_byte_injection(
        prefix in r"[a-z]{1,10}",
        suffix in r"[a-z]{1,10}"
    ) {
        let input = format!("{}\0{}", prefix, suffix);
        let sanitized = sanitize_path_component(&input);

        // Verify no null bytes
        prop_assert!(
            !sanitized.contains('\0'),
            "Null byte found in sanitized path: {:?}",
            sanitized
        );

        // Verify alphanumeric characters are preserved
        prop_assert!(
            sanitized.contains(&prefix) && sanitized.contains(&suffix),
            "Alphanumeric parts lost during sanitization: input={:?}, output={:?}",
            input,
            sanitized
        );
    }
}

// Property test: Mixed attack patterns
//
// Combines multiple attack patterns to test defense-in-depth.
proptest! {
    #![proptest_config(ProptestConfig::with_cases(PROPTEST_CASES))]
    #[test]
    fn prop_mixed_attack_patterns(
        parent_count in 1usize..10,
        special_chars in r"[/:;<>]{0,5}",
        target in r"[a-z]{1,10}"
    ) {
        let temp_dir = TempDir::new().unwrap();
        let base = temp_dir.path();

        // Construct complex attack: ../..:;/<>target
        let attack = format!("{}{}{}", "../".repeat(parent_count), special_chars, target);
        let attack_path = PathBuf::from(&attack);

        let result = safe_join(base, &attack_path);

        // Verify the result is safe (if successful)
        if let Ok(joined) = result {
            prop_assert!(
                joined.starts_with(base),
                "Mixed attack bypassed security: {:?} is outside {:?}",
                joined,
                base
            );
        }
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    /// Sanity check: Ensure proptest framework is working
    #[test]
    fn test_proptest_security_sanity() {
        proptest!(|(x in 0..100u32)| {
            assert!(x < 100);
        });
    }

    /// Manual test: Verify directory traversal prevention
    #[test]
    fn test_directory_traversal_manual() {
        let temp_dir = TempDir::new().unwrap();
        let base = temp_dir.path();

        let attack_patterns = vec![
            "../../../etc/passwd",
            "..\\..\\..\\windows\\system32",
            "normal/../../../etc/passwd",
            "./../../etc/passwd",
        ];

        for pattern in attack_patterns {
            let attack_path = PathBuf::from(pattern);
            let result = safe_join(base, &attack_path);

            if let Ok(joined) = result {
                assert!(
                    joined.starts_with(base),
                    "Attack pattern '{}' bypassed security",
                    pattern
                );
            }
        }
    }

    /// Manual test: Verify sanitization removes dangerous characters
    #[test]
    fn test_sanitization_manual() {
        let dangerous_inputs = vec![
            ("file.txt", "filetxt"),
            ("../../etc/passwd", "etcpasswd"),
            ("file:with:colons", "filewithcolons"),
            ("file with spaces", "filewithspaces"),
            ("file\x00null\x00bytes", "filenullbytes"),
        ];

        for (input, expected) in dangerous_inputs {
            let sanitized = sanitize_path_component(input);
            assert_eq!(
                sanitized, expected,
                "Sanitization failed for input: {}",
                input
            );
        }
    }
}
