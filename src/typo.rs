// Typo detection and suggestion module
// Implements Levenshtein distance algorithm for command name similarity

/// Calculate Levenshtein distance between two strings
///
/// This algorithm computes the minimum number of single-character edits
/// (insertions, deletions, or substitutions) required to change one string into another.
///
/// # Examples
///
/// ```
/// use backup_suite::typo::levenshtein_distance;
///
/// assert_eq!(levenshtein_distance("kitten", "sitting"), 3);
/// assert_eq!(levenshtein_distance("backup", "bakup"), 1);
/// assert_eq!(levenshtein_distance("restore", "restor"), 1);
/// ```
pub fn levenshtein_distance(a: &str, b: &str) -> usize {
    let a_len = a.chars().count();
    let b_len = b.chars().count();

    // Early returns for edge cases
    if a_len == 0 {
        return b_len;
    }
    if b_len == 0 {
        return a_len;
    }

    // Create distance matrix
    let mut matrix = vec![vec![0; b_len + 1]; a_len + 1];

    // Initialize first column and row
    for (i, row) in matrix.iter_mut().enumerate().take(a_len + 1) {
        row[0] = i;
    }
    for (j, elem) in matrix[0].iter_mut().enumerate().take(b_len + 1) {
        *elem = j;
    }

    // Convert to vectors of chars for indexing
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();

    // Calculate distances
    for (i, a_char) in a_chars.iter().enumerate() {
        for (j, b_char) in b_chars.iter().enumerate() {
            let cost = if a_char == b_char { 0 } else { 1 };

            matrix[i + 1][j + 1] = std::cmp::min(
                std::cmp::min(
                    matrix[i][j + 1] + 1, // deletion
                    matrix[i + 1][j] + 1, // insertion
                ),
                matrix[i][j] + cost, // substitution
            );
        }
    }

    matrix[a_len][b_len]
}

/// Find the most similar command from a list of valid commands
///
/// Returns the closest match if the distance is <= max_distance, otherwise None.
///
/// # Arguments
///
/// * `typo` - The potentially misspelled command
/// * `valid_commands` - List of valid command names
/// * `max_distance` - Maximum Levenshtein distance to consider (default: 2)
///
/// # Examples
///
/// ```
/// use backup_suite::typo::find_similar_command;
///
/// let commands = vec!["add", "list", "remove", "run", "restore"];
/// assert_eq!(find_similar_command("bakup", &commands, 2), None);
/// assert_eq!(find_similar_command("restor", &commands, 2), Some("restore".to_string()));
/// assert_eq!(find_similar_command("ad", &commands, 2), Some("add".to_string()));
/// ```
pub fn find_similar_command(
    typo: &str,
    valid_commands: &[&str],
    max_distance: usize,
) -> Option<String> {
    let typo_lower = typo.to_lowercase();

    let mut best_match: Option<(String, usize)> = None;

    for &cmd in valid_commands {
        let distance = levenshtein_distance(&typo_lower, &cmd.to_lowercase());

        // Only consider matches within max_distance
        if distance <= max_distance {
            match &best_match {
                None => best_match = Some((cmd.to_string(), distance)),
                Some((_, best_distance)) => {
                    if distance < *best_distance {
                        best_match = Some((cmd.to_string(), distance));
                    }
                }
            }
        }
    }

    best_match.map(|(cmd, _)| cmd)
}

/// Generate a "did you mean" error message
///
/// # Arguments
///
/// * `typo` - The misspelled command
/// * `suggestion` - The suggested correct command
/// * `with_color` - Whether to include ANSI color codes
///
/// # Returns
///
/// A formatted error message suggesting the correct command
pub fn format_did_you_mean(typo: &str, suggestion: &str, with_color: bool) -> String {
    let (yellow, green, reset) = if with_color {
        ("\x1b[33m", "\x1b[32m", "\x1b[0m")
    } else {
        ("", "", "")
    };

    format!(
        "{}error:{} unrecognized subcommand '{}'\n\n{}Did you mean '{}{}{}'?{}\n\nFor more information, try '{}--help{}'.",
        yellow,
        reset,
        typo,
        yellow,
        green,
        suggestion,
        yellow,
        reset,
        green,
        reset
    )
}

/// List of all valid backup-suite commands
pub const VALID_COMMANDS: &[&str] = &[
    "add",
    "list",
    "ls",
    "remove",
    "clear",
    "rm",
    "run",
    "restore",
    "cleanup",
    "status",
    "history",
    "dashboard",
    "open",
    "completion",
    "schedule",
    "config",
    "smart",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_levenshtein_distance_identical() {
        assert_eq!(levenshtein_distance("test", "test"), 0);
        assert_eq!(levenshtein_distance("backup", "backup"), 0);
    }

    #[test]
    fn test_levenshtein_distance_single_char() {
        assert_eq!(levenshtein_distance("backup", "bakup"), 1);
        assert_eq!(levenshtein_distance("restore", "restor"), 1);
        assert_eq!(levenshtein_distance("add", "ad"), 1);
    }

    #[test]
    fn test_levenshtein_distance_multiple_chars() {
        assert_eq!(levenshtein_distance("kitten", "sitting"), 3);
        assert_eq!(levenshtein_distance("saturday", "sunday"), 3);
    }

    #[test]
    fn test_levenshtein_distance_empty() {
        assert_eq!(levenshtein_distance("", "test"), 4);
        assert_eq!(levenshtein_distance("test", ""), 4);
        assert_eq!(levenshtein_distance("", ""), 0);
    }

    #[test]
    fn test_find_similar_command_exact_match() {
        let commands = vec!["add", "list", "remove"];
        assert_eq!(
            find_similar_command("add", &commands, 2),
            Some("add".to_string())
        );
    }

    #[test]
    fn test_find_similar_command_close_match() {
        let commands = vec!["add", "list", "remove", "restore"];
        assert_eq!(
            find_similar_command("restor", &commands, 2),
            Some("restore".to_string())
        );
        assert_eq!(
            find_similar_command("ad", &commands, 2),
            Some("add".to_string())
        );
    }

    #[test]
    fn test_find_similar_command_no_match() {
        let commands = vec!["add", "list", "remove"];
        assert_eq!(find_similar_command("xyz", &commands, 2), None);
        assert_eq!(
            find_similar_command("completely_different", &commands, 2),
            None
        );
    }

    #[test]
    fn test_find_similar_command_case_insensitive() {
        let commands = vec!["Add", "List", "Remove"];
        assert_eq!(
            find_similar_command("add", &commands, 2),
            Some("Add".to_string())
        );
        assert_eq!(
            find_similar_command("LIST", &commands, 2),
            Some("List".to_string())
        );
    }

    #[test]
    fn test_find_similar_command_max_distance() {
        let commands = vec!["backup"];
        // Distance 1: should match
        assert_eq!(
            find_similar_command("bakup", &commands, 2),
            Some("backup".to_string())
        );
        // Distance 3: should not match with max_distance=2
        assert_eq!(find_similar_command("bak", &commands, 2), None);
    }

    #[test]
    fn test_format_did_you_mean_with_color() {
        let result = format_did_you_mean("bakup", "backup", true);
        assert!(result.contains("unrecognized subcommand 'bakup'"));
        // Color codes are embedded, so check for core components
        assert!(result.contains("backup"));
        assert!(result.contains("Did you mean"));
        assert!(result.contains("--help"));
        assert!(result.contains("\x1b[")); // ANSI color code
        assert!(result.contains("\x1b[32m")); // Green color
        assert!(result.contains("\x1b[33m")); // Yellow color
        assert!(result.contains("\x1b[0m")); // Reset
    }

    #[test]
    fn test_format_did_you_mean_without_color() {
        let result = format_did_you_mean("restor", "restore", false);
        assert!(result.contains("unrecognized subcommand 'restor'"));
        assert!(result.contains("Did you mean 'restore'?"));
        assert!(!result.contains("\x1b[")); // No ANSI color codes
    }

    #[test]
    fn test_valid_commands_list() {
        // Verify all expected commands are present
        assert!(VALID_COMMANDS.contains(&"add"));
        assert!(VALID_COMMANDS.contains(&"list"));
        assert!(VALID_COMMANDS.contains(&"remove"));
        assert!(VALID_COMMANDS.contains(&"run"));
        assert!(VALID_COMMANDS.contains(&"restore"));
        assert!(VALID_COMMANDS.contains(&"cleanup"));
        assert!(VALID_COMMANDS.contains(&"schedule"));
        assert!(VALID_COMMANDS.contains(&"config"));

        // Verify aliases are present
        assert!(VALID_COMMANDS.contains(&"ls"));
        assert!(VALID_COMMANDS.contains(&"rm"));
    }

    #[test]
    fn test_real_world_typos() {
        // Common typos users might make
        let typos = vec![
            ("bakup", "backup"), // Would need "backup" in command list (not currently a command)
            ("restor", "restore"),
            ("ad", "add"),
            ("lst", "list"),
            ("rmove", "remove"),
            ("claenup", "cleanup"),
        ];

        for (typo, expected) in typos {
            if VALID_COMMANDS.contains(&expected) {
                let result = find_similar_command(typo, VALID_COMMANDS, 2);
                assert_eq!(
                    result,
                    Some(expected.to_string()),
                    "Failed for typo '{}', expected '{}'",
                    typo,
                    expected
                );
            }
        }
    }
}
