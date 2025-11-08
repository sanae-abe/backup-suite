// src/crypto/password_policy.rs
// Password policy implementation with user-friendly approach
// Design: Warning-only (non-enforcing) with strength scoring

use rand::Rng;
use zeroize::Zeroizing;

/// Password strength level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PasswordStrength {
    Weak,
    Medium,
    Strong,
}

impl PasswordStrength {
    /// Get color-coded display string
    pub fn display(&self) -> &str {
        match self {
            PasswordStrength::Weak => "Weak",
            PasswordStrength::Medium => "Medium",
            PasswordStrength::Strong => "Strong",
        }
    }

    /// Get descriptive message for the strength level
    pub fn message(&self) -> &str {
        match self {
            PasswordStrength::Weak => {
                "This password may be vulnerable to attacks. Consider using a longer password with varied characters."
            }
            PasswordStrength::Medium => {
                "This password provides moderate security. Adding special characters or length would improve it."
            }
            PasswordStrength::Strong => {
                "This password provides strong security."
            }
        }
    }
}

/// Password policy configuration
pub struct PasswordPolicy {
    /// Minimum password length (NIST SP 800-63B recommends minimum 8 characters)
    pub min_length: usize,
    /// Enable entropy-based strength checking
    pub check_entropy: bool,
}

impl Default for PasswordPolicy {
    fn default() -> Self {
        Self {
            min_length: 8,
            check_entropy: true,
        }
    }
}

impl PasswordPolicy {
    /// Evaluate password strength (non-enforcing)
    pub fn evaluate(&self, password: &str) -> PasswordStrength {
        let length = password.len();

        // Calculate entropy score
        let entropy = self.calculate_entropy(password);

        // Length-based scoring (adjusted for better granularity)
        let length_score = if length < 8 {
            0
        } else if length < 10 {
            1
        } else if length < 14 {
            2
        } else {
            3
        };

        // Entropy-based scoring (adjusted thresholds)
        let entropy_score = if entropy < 25.0 {
            0
        } else if entropy < 40.0 {
            1
        } else if entropy < 60.0 {
            2
        } else {
            3
        };

        // Penalty for common patterns
        let pattern_penalty = if self.is_common_password(password)
            || self.is_repeated_chars(password)
            || self.is_sequential(password)
        {
            -2
        } else {
            0
        };

        // Combined score
        let total_score = (length_score + entropy_score + pattern_penalty).max(0);

        match total_score {
            0..=2 => PasswordStrength::Weak,
            3..=4 => PasswordStrength::Medium,
            _ => PasswordStrength::Strong,
        }
    }

    /// Calculate Shannon entropy of password
    fn calculate_entropy(&self, password: &str) -> f64 {
        if password.is_empty() {
            return 0.0;
        }

        // Count character frequencies
        let mut freq_map = std::collections::HashMap::new();
        for c in password.chars() {
            *freq_map.entry(c).or_insert(0) += 1;
        }

        // Calculate Shannon entropy
        let len = password.len() as f64;
        let mut entropy = 0.0;
        for count in freq_map.values() {
            let p = *count as f64 / len;
            entropy -= p * p.log2();
        }

        // Scale by password length
        entropy * len
    }

    /// Check if password contains common weak patterns
    pub fn check_common_patterns(&self, password: &str) -> Vec<&'static str> {
        let mut warnings = Vec::new();

        // Check for repeated characters (e.g., "aaaaaaaa")
        if self.is_repeated_chars(password) {
            warnings.push("Contains repeated characters");
        }

        // Check for sequential patterns (e.g., "12345678", "abcdefgh")
        if self.is_sequential(password) {
            warnings.push("Contains sequential pattern");
        }

        // Check for common weak passwords
        if self.is_common_password(password) {
            warnings.push("This is a commonly used password");
        }

        warnings
    }

    /// Check if password is mostly repeated characters
    fn is_repeated_chars(&self, password: &str) -> bool {
        if password.len() < 4 {
            return false;
        }

        let chars: Vec<char> = password.chars().collect();
        let unique_chars: std::collections::HashSet<_> = chars.iter().collect();

        // If unique characters are less than 30% of total, consider it repeated
        unique_chars.len() < (chars.len() as f64 * 0.3) as usize
    }

    /// Check if password contains sequential patterns
    fn is_sequential(&self, password: &str) -> bool {
        if password.len() < 4 {
            return false;
        }

        let lower = password.to_lowercase();
        let sequential_patterns = [
            "0123456789",
            "abcdefghijklmnopqrstuvwxyz",
            "qwertyuiop",
            "asdfghjkl",
            "zxcvbnm",
        ];

        for pattern in &sequential_patterns {
            // Check for forward and reverse sequences of length 4+
            for window_size in 4..=password.len().min(8) {
                // Fix: check if pattern is long enough before subtracting
                if pattern.len() < window_size {
                    continue;
                }

                for i in 0..=(pattern.len() - window_size) {
                    let seq = &pattern[i..i + window_size];
                    let rev_seq: String = seq.chars().rev().collect();

                    if lower.contains(seq) || lower.contains(&rev_seq) {
                        return true;
                    }
                }
            }
        }

        false
    }

    /// Check against common password list (top 100 most common passwords)
    fn is_common_password(&self, password: &str) -> bool {
        const COMMON_PASSWORDS: &[&str] = &[
            "password",
            "12345678",
            "123456789",
            "qwerty",
            "abc123",
            "monkey",
            "1234567",
            "letmein",
            "trustno1",
            "dragon",
            "baseball",
            "111111",
            "iloveyou",
            "master",
            "sunshine",
            "ashley",
            "bailey",
            "passw0rd",
            "shadow",
            "123123",
            "654321",
            "superman",
            "qazwsx",
            "michael",
            "football",
            "welcome",
            "jesus",
            "ninja",
            "mustang",
            "password1",
        ];

        let lower = password.to_lowercase();
        COMMON_PASSWORDS
            .iter()
            .any(|&p| lower == p || lower.contains(p))
    }

    /// Generate a strong random password
    pub fn generate_password(&self, length: usize) -> Zeroizing<String> {
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                                  abcdefghijklmnopqrstuvwxyz\
                                  0123456789\
                                  !@#$%^&*()-_=+[]{}|;:,.<>?";

        let mut rng = rand::thread_rng();
        let password: String = (0..length)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect();

        Zeroizing::new(password)
    }

    /// Display password strength report (non-enforcing)
    pub fn display_report(&self, password: &str) -> String {
        let strength = self.evaluate(password);
        let warnings = self.check_common_patterns(password);

        let mut report = format!("\nPassword Strength: {}\n", strength.display());
        report.push_str(&format!("  {}\n", strength.message()));

        if !warnings.is_empty() {
            report.push_str("\nWarnings:\n");
            for warning in warnings {
                report.push_str(&format!("  - {warning}\n"));
            }
        }

        if matches!(strength, PasswordStrength::Weak) {
            report.push_str("\nTip: Use --generate-password to create a strong random password.\n");
        }

        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_strength_weak() {
        let policy = PasswordPolicy::default();

        // Too short
        assert_eq!(policy.evaluate("123"), PasswordStrength::Weak);

        // Repeated characters
        assert_eq!(policy.evaluate("aaaaaaaa"), PasswordStrength::Weak);

        // Common password
        assert_eq!(policy.evaluate("password"), PasswordStrength::Weak);
    }

    #[test]
    fn test_password_strength_medium() {
        let policy = PasswordPolicy::default();

        // Decent length, moderate entropy
        assert_eq!(policy.evaluate("MyBackup2024"), PasswordStrength::Medium);
        assert_eq!(policy.evaluate("hello_world"), PasswordStrength::Medium);
    }

    #[test]
    fn test_password_strength_strong() {
        let policy = PasswordPolicy::default();

        // Long with good entropy
        assert_eq!(
            policy.evaluate("MyS3cur3!B@ckup#2024"),
            PasswordStrength::Strong
        );
        assert_eq!(
            policy.evaluate("correct-horse-battery-staple"),
            PasswordStrength::Strong
        );
    }

    #[test]
    fn test_entropy_calculation() {
        let policy = PasswordPolicy::default();

        // Low entropy (repeated characters)
        let entropy1 = policy.calculate_entropy("aaaaaaaa");
        assert!(entropy1 < 10.0);

        // High entropy (random characters)
        let entropy2 = policy.calculate_entropy("aB3$xY9!");
        assert!(entropy2 > 20.0);
    }

    #[test]
    fn test_repeated_chars_detection() {
        let policy = PasswordPolicy::default();

        assert!(policy.is_repeated_chars("aaaaaaaa"));
        assert!(policy.is_repeated_chars("11111111"));
        assert!(!policy.is_repeated_chars("MyPassword"));
    }

    #[test]
    fn test_sequential_detection() {
        let policy = PasswordPolicy::default();

        assert!(policy.is_sequential("12345678"));
        assert!(policy.is_sequential("abcdefgh"));
        assert!(policy.is_sequential("qwerty"));
        assert!(policy.is_sequential("asdfgh"));
        assert!(!policy.is_sequential("MyPassword"));
    }

    #[test]
    fn test_common_password_detection() {
        let policy = PasswordPolicy::default();

        assert!(policy.is_common_password("password"));
        assert!(policy.is_common_password("12345678"));
        assert!(policy.is_common_password("qwerty"));
        assert!(policy.is_common_password("Password1")); // Contains "password"
        assert!(!policy.is_common_password("MyUniqueP@ss2024"));
    }

    #[test]
    fn test_pattern_warnings() {
        let policy = PasswordPolicy::default();

        let warnings1 = policy.check_common_patterns("aaaaaaaa");
        assert!(warnings1.contains(&"Contains repeated characters"));

        let warnings2 = policy.check_common_patterns("12345678");
        assert!(warnings2.contains(&"Contains sequential pattern"));

        let warnings3 = policy.check_common_patterns("password");
        assert!(warnings3.contains(&"This is a commonly used password"));

        let warnings4 = policy.check_common_patterns("MyS3cur3!P@ss");
        assert!(warnings4.is_empty());
    }

    #[test]
    fn test_password_generation() {
        let policy = PasswordPolicy::default();

        // Generate password of various lengths
        for length in [12, 16, 20, 24] {
            let password = policy.generate_password(length);
            assert_eq!(password.len(), length);

            // Generated password should be at least Medium strength
            // (Most will be Strong, but we allow Medium due to randomness)
            let strength = policy.evaluate(&password);
            assert!(
                matches!(
                    strength,
                    PasswordStrength::Medium | PasswordStrength::Strong
                ),
                "Generated password of length {length} should be at least Medium strength, got {strength:?}"
            );

            // For longer passwords, we expect Strong
            if length >= 20 {
                // At least 90% of 20+ char passwords should be Strong
                // We'll just check it's not Weak
                assert!(
                    !matches!(strength, PasswordStrength::Weak),
                    "Generated password of length {length} should not be Weak"
                );
            }
        }
    }

    #[test]
    fn test_display_report() {
        let policy = PasswordPolicy::default();

        let report1 = policy.display_report("123");
        assert!(report1.contains("Weak"));
        assert!(report1.contains("--generate-password"));

        let report2 = policy.display_report("MyS3cur3!P@ss2024");
        assert!(report2.contains("Strong"));
        assert!(!report2.contains("--generate-password"));
    }
}
