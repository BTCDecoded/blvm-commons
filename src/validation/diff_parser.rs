//! Diff Parser for Consensus Modification Verification
//!
//! Parses unified diff format to detect non-import changes in consensus-critical files.
//! Used by cross-layer validation to enforce "import-only" rules.

use crate::error::GovernanceError;
use regex::Regex;
use std::collections::HashSet;
use tracing::debug;

/// Represents a file diff with additions and deletions
#[derive(Debug, Clone)]
pub struct FileDiff {
    pub filename: String,
    pub additions: Vec<String>,
    pub deletions: Vec<String>,
}

/// Diff parser for analyzing code changes
pub struct DiffParser;

impl DiffParser {
    /// Parse unified diff format
    ///
    /// Unified diff format:
    /// ```text
    /// --- a/path/to/file.rs
    /// +++ b/path/to/file.rs
    /// @@ -start,count +start,count @@
    /// -deleted line
    /// +added line
    ///  unchanged line
    /// ```
    pub fn parse_unified_diff(diff: &str) -> Result<Vec<FileDiff>, GovernanceError> {
        let lines: Vec<&str> = diff.lines().collect();
        let mut file_diffs = Vec::new();
        let mut current_file: Option<FileDiff> = None;
        let mut in_hunk = false;

        // GitHub returns git-format diffs, where every file starts with a
        // `diff --git` line. In that format only `diff --git` ends a file, so
        // hunk body lines that look like `---`/`+++` headers cannot split it.
        let git_format = lines.iter().any(|l| l.starts_with("diff --git "));

        // Regex to match hunk header lines
        let hunk_header_re = Regex::new(r"^@@ .+ @@").unwrap();

        fn clean_path(path: &str) -> Option<String> {
            // Drop a trailing tab-separated timestamp (plain `diff -u` output)
            let path = path.split('\t').next().unwrap_or(path).trim();
            if path == "/dev/null" {
                return None;
            }
            let path = path
                .strip_prefix("a/")
                .or_else(|| path.strip_prefix("b/"))
                .unwrap_or(path);
            Some(path.to_string())
        }

        let mut i = 0;
        while i < lines.len() {
            let line = lines[i];
            i += 1;

            // Start of a new file: `diff --git a/x b/y`, or (plain unified
            // diff) a `--- ` line immediately followed by `+++ `.
            let is_git_header = line.starts_with("diff --git ");
            let is_plain_header = !git_format
                && line.starts_with("--- ")
                && lines.get(i).is_some_and(|next| next.starts_with("+++ "));

            if is_git_header || is_plain_header {
                if let Some(file) = current_file.take() {
                    file_diffs.push(file);
                }
                let filename = if is_git_header {
                    // Provisional name from `diff --git a/x b/y`; refined by `+++`/`---`
                    line.rsplit(" b/").next().unwrap_or("").to_string()
                } else {
                    clean_path(&line[4..]).unwrap_or_default()
                };
                current_file = Some(FileDiff {
                    filename,
                    additions: Vec::new(),
                    deletions: Vec::new(),
                });
                in_hunk = false;
                continue;
            }

            // File header lines (before the first hunk of a file): finalize path
            if !in_hunk && current_file.is_some() {
                if let Some(path) = line.strip_prefix("+++ ") {
                    if let (Some(file), Some(path)) = (current_file.as_mut(), clean_path(path)) {
                        file.filename = path;
                    }
                    continue;
                }
                if let Some(path) = line.strip_prefix("--- ") {
                    if let (Some(file), Some(path)) = (current_file.as_mut(), clean_path(path)) {
                        // Overridden by `+++` unless that side is /dev/null (deleted file)
                        file.filename = path;
                    }
                    continue;
                }
            }

            // Hunk header
            if hunk_header_re.is_match(line) && current_file.is_some() {
                in_hunk = true;
                continue;
            }

            // Hunk body: collect until the next file header
            if in_hunk {
                if let Some(ref mut file) = current_file {
                    if let Some(content) = line.strip_prefix('+') {
                        file.additions.push(content.to_string());
                    } else if let Some(content) = line.strip_prefix('-') {
                        file.deletions.push(content.to_string());
                    }
                    // Context lines (' ') and "\ No newline at end of file" are ignored
                }
            }
        }

        if let Some(file) = current_file {
            file_diffs.push(file);
        }

        debug!("Parsed {} file diffs", file_diffs.len());
        Ok(file_diffs)
    }

    /// Check if file changes are import-only
    ///
    /// Returns true if all changes are:
    /// - Import statements (use declarations)
    /// - Comments
    /// - Whitespace
    ///
    /// Returns false if any changes include:
    /// - Function definitions
    /// - Type definitions
    /// - Logic changes
    /// - Macro definitions
    pub fn is_import_only_changes(file_diff: &FileDiff) -> bool {
        // Check both additions and deletions
        for line in &file_diff.additions {
            if !Self::is_import_or_comment_line(line) {
                debug!(
                    "Non-import change detected in {}: {}",
                    file_diff.filename,
                    line.trim()
                );
                return false;
            }
        }

        for line in &file_diff.deletions {
            if !Self::is_import_or_comment_line(line) {
                debug!(
                    "Non-import change detected in {}: {}",
                    file_diff.filename,
                    line.trim()
                );
                return false;
            }
        }

        true
    }

    /// Check if a line is an import statement or comment
    fn is_import_or_comment_line(line: &str) -> bool {
        let trimmed = line.trim();

        // Empty lines or whitespace
        if trimmed.is_empty() {
            return true;
        }

        // Comments (single-line and multi-line)
        if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with("*") {
            return true;
        }

        // Import statements (use declarations)
        // Match: use ... ;
        // Match: pub use ... ;
        // Match: #[...] use ... ;
        if trimmed.starts_with("use ") || trimmed.starts_with("pub use ") {
            return true;
        }

        // Attribute macros before use statements (e.g., #[cfg(...)] use ...)
        if trimmed.starts_with("#[") && trimmed.contains("use ") {
            return true;
        }

        // Module declarations (pub mod ...) - sometimes needed for imports
        if trimmed.starts_with("pub mod ") || trimmed.starts_with("mod ") {
            return true;
        }

        false
    }

    /// Extract import statements from diff
    pub fn extract_imports(file_diff: &FileDiff) -> Vec<String> {
        let mut imports = HashSet::new();

        for line in &file_diff.additions {
            if Self::is_import_or_comment_line(line) && line.contains("use ") {
                imports.insert(line.trim().to_string());
            }
        }

        for line in &file_diff.deletions {
            if Self::is_import_or_comment_line(line) && line.contains("use ") {
                imports.insert(line.trim().to_string());
            }
        }

        imports.into_iter().collect()
    }

    /// Check if diff contains any consensus-critical changes
    ///
    /// Consensus-critical patterns:
    /// - Function definitions (pub fn, fn)
    /// - Type definitions (struct, enum, trait, impl)
    /// - Macro definitions (macro_rules!, #[macro_export])
    /// - Logic changes (if, match, return, etc.)
    pub fn contains_consensus_logic_changes(file_diff: &FileDiff) -> bool {
        // Patterns that indicate logic changes
        let logic_patterns = vec![
            "fn ",             // Function definitions
            "pub fn ",         // Public function definitions
            "struct ",         // Struct definitions
            "enum ",           // Enum definitions
            "trait ",          // Trait definitions
            "impl ",           // Implementation blocks
            "macro_rules!",    // Macro definitions
            "#[macro_export]", // Macro exports
            "if ",             // Conditional logic
            "match ",          // Pattern matching
            "return ",         // Return statements
            "-> ",             // Function return types
            "=> ",             // Match arms, closures
        ];

        for line in &file_diff.additions {
            let trimmed = line.trim();
            for pattern in &logic_patterns {
                if trimmed.contains(pattern) && !Self::is_import_or_comment_line(line) {
                    debug!(
                        "Consensus logic change detected in {}: {}",
                        file_diff.filename, trimmed
                    );
                    return true;
                }
            }
        }

        for line in &file_diff.deletions {
            let trimmed = line.trim();
            for pattern in &logic_patterns {
                if trimmed.contains(pattern) && !Self::is_import_or_comment_line(line) {
                    debug!(
                        "Consensus logic change detected in {}: {}",
                        file_diff.filename, trimmed
                    );
                    return true;
                }
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_diff() {
        let diff = r#"--- a/src/block.rs
+++ b/src/block.rs
@@ -1,3 +1,4 @@
 use crate::types::Network;
+use crate::bip_validation;
 use std::collections::HashMap;
"#;

        let file_diffs = DiffParser::parse_unified_diff(diff).unwrap();
        assert_eq!(file_diffs.len(), 1);
        assert_eq!(file_diffs[0].filename, "src/block.rs");
        assert_eq!(file_diffs[0].additions.len(), 1);
        assert!(file_diffs[0].additions[0].contains("use crate::bip_validation"));
    }

    #[test]
    fn test_is_import_only() {
        let diff = FileDiff {
            filename: "src/block.rs".to_string(),
            additions: vec!["use crate::bip_validation;".to_string()],
            deletions: vec![],
        };

        assert!(DiffParser::is_import_only_changes(&diff));
    }

    #[test]
    fn test_is_not_import_only() {
        let diff = FileDiff {
            filename: "src/block.rs".to_string(),
            additions: vec!["pub fn connect_block() {".to_string()],
            deletions: vec![],
        };

        assert!(!DiffParser::is_import_only_changes(&diff));
    }

    #[test]
    fn test_contains_consensus_logic() {
        let diff = FileDiff {
            filename: "src/block.rs".to_string(),
            additions: vec!["    pub fn validate_block() {".to_string()],
            deletions: vec![],
        };

        assert!(DiffParser::contains_consensus_logic_changes(&diff));
    }

    #[test]
    fn test_parse_multiple_files() {
        let diff = r#"--- a/src/block.rs
+++ b/src/block.rs
@@ -1,3 +1,4 @@
 use crate::types::Network;
+use crate::bip_validation;
 use std::collections::HashMap;
--- a/src/transaction.rs
+++ b/src/transaction.rs
@@ -1,2 +1,3 @@
 use crate::types::Hash;
+use crate::validation;
"#;

        let file_diffs = DiffParser::parse_unified_diff(diff).unwrap();
        assert_eq!(file_diffs.len(), 2);
        assert_eq!(file_diffs[0].filename, "src/block.rs");
        assert_eq!(file_diffs[1].filename, "src/transaction.rs");
    }

    #[test]
    fn test_parse_diff_with_context_lines() {
        let diff = r#"--- a/src/block.rs
+++ b/src/block.rs
@@ -1,5 +1,6 @@
 use crate::types::Network;
 use std::collections::HashMap;
+use crate::bip_validation;
 
 pub fn connect_block() {
     // existing code
"#;

        let file_diffs = DiffParser::parse_unified_diff(diff).unwrap();
        assert_eq!(file_diffs.len(), 1);
        assert_eq!(file_diffs[0].additions.len(), 1);
        assert!(file_diffs[0].additions[0].contains("use crate::bip_validation"));
    }

    #[test]
    fn test_is_import_only_with_comments() {
        let diff = FileDiff {
            filename: "src/block.rs".to_string(),
            additions: vec![
                "// New import for BIP validation".to_string(),
                "use crate::bip_validation;".to_string(),
            ],
            deletions: vec![],
        };

        assert!(DiffParser::is_import_only_changes(&diff));
    }

    #[test]
    fn test_is_import_only_with_whitespace() {
        let diff = FileDiff {
            filename: "src/block.rs".to_string(),
            additions: vec![
                "".to_string(), // Empty line
                "use crate::bip_validation;".to_string(),
                "    ".to_string(), // Whitespace
            ],
            deletions: vec![],
        };

        assert!(DiffParser::is_import_only_changes(&diff));
    }

    #[test]
    fn test_is_not_import_only_with_function() {
        let diff = FileDiff {
            filename: "src/block.rs".to_string(),
            additions: vec![
                "use crate::bip_validation;".to_string(),
                "pub fn new_function() {".to_string(), // This should fail
            ],
            deletions: vec![],
        };

        assert!(!DiffParser::is_import_only_changes(&diff));
    }

    #[test]
    fn test_extract_imports() {
        let diff = FileDiff {
            filename: "src/block.rs".to_string(),
            additions: vec![
                "use crate::bip_validation;".to_string(),
                "use std::collections::HashMap;".to_string(),
                "pub fn connect_block() {".to_string(), // Not an import
            ],
            deletions: vec!["use crate::old_module;".to_string()],
        };

        let imports = DiffParser::extract_imports(&diff);
        assert_eq!(imports.len(), 3); // Should extract all imports
        assert!(imports.iter().any(|i| i.contains("bip_validation")));
        assert!(imports.iter().any(|i| i.contains("HashMap")));
        assert!(imports.iter().any(|i| i.contains("old_module")));
    }

    #[test]
    fn test_parse_empty_diff() {
        let diff = "";
        let file_diffs = DiffParser::parse_unified_diff(diff).unwrap();
        assert_eq!(file_diffs.len(), 0);
    }

    #[test]
    fn test_parse_diff_with_deletions_only() {
        let diff = FileDiff {
            filename: "src/block.rs".to_string(),
            additions: vec![],
            deletions: vec!["use crate::old_module;".to_string()],
        };

        assert!(DiffParser::is_import_only_changes(&diff));
    }

    #[test]
    fn test_parse_multiple_files_collects_changes_per_file() {
        let diff = r#"--- a/src/block.rs
+++ b/src/block.rs
@@ -1,2 +1,3 @@
 use crate::types::Network;
+use crate::bip_validation;
 use std::collections::HashMap;
--- a/src/transaction.rs
+++ b/src/transaction.rs
@@ -1,2 +1,2 @@
 use crate::types::Hash;
-use crate::old_validation;
+use crate::validation;
"#;

        let file_diffs = DiffParser::parse_unified_diff(diff).unwrap();
        assert_eq!(file_diffs.len(), 2);
        assert_eq!(file_diffs[0].filename, "src/block.rs");
        assert_eq!(file_diffs[0].additions, vec!["use crate::bip_validation;"]);
        assert!(file_diffs[0].deletions.is_empty());
        assert_eq!(file_diffs[1].filename, "src/transaction.rs");
        assert_eq!(file_diffs[1].additions, vec!["use crate::validation;"]);
        assert_eq!(file_diffs[1].deletions, vec!["use crate::old_validation;"]);
    }

    /// Regression: a consensus-file diff that changes logic must not be
    /// reported as import-only (cross_layer.rs import-only gate).
    #[test]
    fn test_consensus_diff_with_logic_change_is_not_import_only() {
        let diff = r#"--- a/src/block.rs
+++ b/src/block.rs
@@ -10,3 +10,4 @@
 use crate::types::Network;
+use crate::bip_validation;
 pub fn connect_block() {
-    if height > 0 { return Err(Invalid); }
+    return Ok(());
"#;

        let file_diffs = DiffParser::parse_unified_diff(diff).unwrap();
        assert_eq!(file_diffs.len(), 1);
        assert_eq!(file_diffs[0].filename, "src/block.rs");
        assert!(!DiffParser::is_import_only_changes(&file_diffs[0]));
        assert!(DiffParser::contains_consensus_logic_changes(&file_diffs[0]));
    }

    /// Same as above, in the `diff --git` format the GitHub API returns.
    #[test]
    fn test_git_format_consensus_diff_with_logic_change_is_not_import_only() {
        let diff = r#"diff --git a/src/block.rs b/src/block.rs
index 1111111..2222222 100644
--- a/src/block.rs
+++ b/src/block.rs
@@ -10,2 +10,2 @@ pub fn connect_block() {
-    if height > 0 { return Err(Invalid); }
+    return Ok(());
diff --git a/README.md b/README.md
index 3333333..4444444 100644
--- a/README.md
+++ b/README.md
@@ -1 +1 @@
-old
+new
"#;

        let file_diffs = DiffParser::parse_unified_diff(diff).unwrap();
        assert_eq!(file_diffs.len(), 2);
        assert_eq!(file_diffs[0].filename, "src/block.rs");
        assert_eq!(file_diffs[0].additions, vec!["    return Ok(());"]);
        assert!(!DiffParser::is_import_only_changes(&file_diffs[0]));
        assert_eq!(file_diffs[1].filename, "README.md");
        assert_eq!(file_diffs[1].additions, vec!["new"]);
    }

    /// In git format, hunk body lines that merely look like `---`/`+++`
    /// headers (e.g. removed "-- x" / added "++ y" inside a block comment)
    /// must stay attached to the current file.
    #[test]
    fn test_git_format_header_lookalikes_stay_in_hunk() {
        let diff = r#"diff --git a/src/block.rs b/src/block.rs
index 1111111..2222222 100644
--- a/src/block.rs
+++ b/src/block.rs
@@ -1,2 +1,3 @@
--- a/src/block.rs
+++ b/docs/notes.md
+pub fn bypass() {}
"#;

        let file_diffs = DiffParser::parse_unified_diff(diff).unwrap();
        assert_eq!(file_diffs.len(), 1);
        assert_eq!(file_diffs[0].filename, "src/block.rs");
        assert!(!DiffParser::is_import_only_changes(&file_diffs[0]));
    }

    #[test]
    fn test_parse_new_and_deleted_files() {
        let diff = r#"--- /dev/null
+++ b/src/new.rs
@@ -0,0 +1 @@
+use crate::types::Hash;
--- a/src/old.rs
+++ /dev/null
@@ -1 +0,0 @@
-pub fn removed() {}
"#;

        let file_diffs = DiffParser::parse_unified_diff(diff).unwrap();
        assert_eq!(file_diffs.len(), 2);
        assert_eq!(file_diffs[0].filename, "src/new.rs");
        assert_eq!(file_diffs[0].additions, vec!["use crate::types::Hash;"]);
        assert_eq!(file_diffs[1].filename, "src/old.rs");
        assert_eq!(file_diffs[1].deletions, vec!["pub fn removed() {}"]);
        assert!(!DiffParser::is_import_only_changes(&file_diffs[1]));
    }
}
