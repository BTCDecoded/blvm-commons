//! Diff Parser for Consensus Modification Verification
//!
//! Parses unified diff format to detect non-import changes in consensus-critical files.
//! Used by cross-layer validation to enforce "import-only" rules.

use crate::error::GovernanceError;
use regex::Regex;
use std::collections::HashSet;
use tracing::debug;

/// Represents a file diff with additions and deletions
#[derive(Debug, Clone, Default)]
pub struct FileDiff {
    pub filename: String,
    pub additions: Vec<String>,
    pub deletions: Vec<String>,
    /// Source path when it differs from `filename` (rename/copy), from
    /// `rename from`/`copy from` or a `---` line that is not /dev/null.
    pub old_filename: Option<String>,
    /// `Binary files ... differ` / `GIT binary patch`: no lines to inspect.
    pub binary: bool,
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
        let mut hunk_additions = Vec::new();
        let mut hunk_deletions = Vec::new();

        // GitHub returns git-format diffs, where every file starts with a
        // `diff --git` line. In that format only `diff --git` ends a file, so
        // hunk body lines that look like `---`/`+++` headers cannot split it.
        let git_format = lines.iter().any(|l| l.starts_with("diff --git "));

        // Regex to match file header lines (a tab-separated timestamp is dropped)
        let file_header_re = Regex::new(r"^(?:---|\+\+\+) ([^\t]+)").unwrap();
        // Regex to match hunk header lines
        let hunk_header_re = Regex::new(r"^@@ .+ @@").unwrap();

        for (i, &line) in lines.iter().enumerate() {
            // A new file starts at `diff --git` (git format), or at a `---`
            // line immediately followed by `+++` (plain unified diff).
            let starts_file = if git_format {
                line.starts_with("diff --git ")
            } else {
                line.starts_with("--- ") && lines.get(i + 1).is_some_and(|l| l.starts_with("+++ "))
            };
            if starts_file {
                if let Some(mut file) = current_file.take() {
                    file.additions.append(&mut hunk_additions);
                    file.deletions.append(&mut hunk_deletions);
                    file_diffs.push(file);
                }
                // Provisional name from `diff --git a/x b/y`; set by `---`/`+++` below
                let filename = if git_format {
                    line.rsplit(" b/").next().unwrap_or("")
                } else {
                    ""
                };
                current_file = Some(FileDiff {
                    filename: filename.to_string(),
                    ..Default::default()
                });
                in_hunk = false;
                hunk_additions.clear();
                hunk_deletions.clear();
                if git_format {
                    continue;
                }
            }

            // Extended headers (git format), only before a file's first hunk
            if !in_hunk {
                if let Some(ref mut file) = current_file {
                    if let Some(src) = line
                        .strip_prefix("rename from ")
                        .or_else(|| line.strip_prefix("copy from "))
                    {
                        file.old_filename = Some(src.to_string());
                        continue;
                    }
                    if line.starts_with("Binary files ") || line == "GIT binary patch" {
                        file.binary = true;
                        continue;
                    }
                }
            }

            // Check for file header (--- or +++), only before a file's first hunk
            if !in_hunk {
                if let Some(caps) = file_header_re.captures(line) {
                    let path = caps.get(1).unwrap().as_str();
                    // Remove a/ or b/ prefix if present
                    let clean_path = path.strip_prefix("a/").unwrap_or(path);
                    let clean_path = clean_path.strip_prefix("b/").unwrap_or(clean_path);

                    // `+++` (destination) overrides `---`; /dev/null is a new or deleted file
                    if let Some(ref mut file) = current_file {
                        if path != "/dev/null" {
                            if line.starts_with("---") {
                                file.old_filename = Some(clean_path.to_string());
                            }
                            file.filename = clean_path.to_string();
                        }
                    }
                    continue;
                }
            }

            // Check for hunk header
            if hunk_header_re.is_match(line) {
                // Save previous hunk's changes to current file
                if let Some(ref mut file) = current_file {
                    file.additions.append(&mut hunk_additions);
                    file.deletions.append(&mut hunk_deletions);
                }
                in_hunk = true;
                hunk_additions.clear();
                hunk_deletions.clear();
                continue;
            }

            // Process diff lines within a hunk (header lookalikes included)
            if in_hunk {
                if let Some(ref mut _file) = current_file {
                    if line.starts_with("+") {
                        // Added line (remove the + prefix)
                        let content = line.strip_prefix("+").unwrap_or(line).to_string();
                        hunk_additions.push(content);
                    } else if line.starts_with("-") {
                        // Deleted line (remove the - prefix)
                        let content = line.strip_prefix("-").unwrap_or(line).to_string();
                        hunk_deletions.push(content);
                    }
                    // Lines starting with space are context (unchanged), ignore them
                }
            }
        }

        // Save last file
        if let Some(mut file) = current_file {
            // Save any remaining hunk changes
            file.additions.append(&mut hunk_additions);
            file.deletions.append(&mut hunk_deletions);
            file_diffs.push(file);
        }

        for f in &mut file_diffs {
            if f.old_filename.as_deref() == Some(f.filename.as_str()) {
                f.old_filename = None;
            }
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

    /// Check if a line is an import statement or comment.
    ///
    /// A line counts only when it is entirely one of:
    /// - blank,
    /// - a `//` line comment (including `///` and `//!` doc comments),
    /// - a single private `use path::item;` or `use path::{a, b};` item,
    ///   optionally preceded by `#[cfg(...)]` attributes, with nothing after
    ///   the `;` except a `//` comment.
    ///
    /// Block comments (`/*`, `*/`, `*` continuation lines), `mod` items,
    /// standalone attributes, multi-line `use` lists and any trailing code
    /// are rejected, so `mod evil { pub fn ..`, `use x; pub fn ..` and
    /// `/* */ return Ok(());` do not pass. So are forms that can change
    /// which code runs or what the crate exports: any attribute other than
    /// `#[cfg(...)]` (an attribute macro can expand to anything),
    /// `extern crate`, `pub`/`pub(...)` re-exports, `as` renames and glob
    /// (`*`) imports, which can shadow a name consensus code calls.
    fn is_import_or_comment_line(line: &str) -> bool {
        let trimmed = line.trim();

        // Empty lines or whitespace
        if trimmed.is_empty() {
            return true;
        }

        // Block comment markers can hide or reveal code; never accept them
        if trimmed.contains("/*") || trimmed.contains("*/") {
            return false;
        }

        // Line comments
        if trimmed.starts_with("//") {
            return true;
        }

        // Drop a trailing `//` comment, then require a single import item
        let code = match trimmed.find("//") {
            Some(i) => trimmed[..i].trim_end(),
            None => trimmed,
        };
        let mut code = code;
        while code.starts_with("#[") {
            code = match Self::strip_leading_attribute(code) {
                Some(rest) => rest,
                None => return false,
            };
        }
        Self::import_item_re().is_match(code)
            && !code
                .split(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
                .any(|word| word == "as")
    }

    /// Strip one leading `#[cfg(...)]` attribute (bracket-balanced). Returns
    /// the line unchanged when there is no attribute, and `None` for any
    /// other attribute, an unbalanced one, or one with nothing after it.
    fn strip_leading_attribute(code: &str) -> Option<&str> {
        let Some(rest) = code.strip_prefix("#[") else {
            return Some(code);
        };
        let mut depth = 1usize;
        for (i, c) in rest.char_indices() {
            match c {
                '[' => depth += 1,
                ']' => {
                    depth -= 1;
                    if depth == 0 {
                        let is_cfg = rest[..i]
                            .trim_start()
                            .strip_prefix("cfg")
                            .is_some_and(|args| args.trim_start().starts_with('('));
                        let after = rest[i + 1..].trim_start();
                        return (is_cfg && !after.is_empty()).then_some(after);
                    }
                }
                _ => {}
            }
        }
        None
    }

    /// Private `use path::item;` or `use path::{a, b};`, whole line. `as`
    /// renames are rejected separately.
    fn import_item_re() -> &'static Regex {
        static RE: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
        RE.get_or_init(|| Regex::new(r"^use\s+[A-Za-z0-9_:{},\s]+;$").expect("import regex"))
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
            ..Default::default()
        };

        assert!(DiffParser::is_import_only_changes(&diff));
    }

    #[test]
    fn test_is_not_import_only() {
        let diff = FileDiff {
            filename: "src/block.rs".to_string(),
            additions: vec!["pub fn connect_block() {".to_string()],
            deletions: vec![],
            ..Default::default()
        };

        assert!(!DiffParser::is_import_only_changes(&diff));
    }

    #[test]
    fn test_contains_consensus_logic() {
        let diff = FileDiff {
            filename: "src/block.rs".to_string(),
            additions: vec!["    pub fn validate_block() {".to_string()],
            deletions: vec![],
            ..Default::default()
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
            ..Default::default()
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
            ..Default::default()
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
            ..Default::default()
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
            ..Default::default()
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
            ..Default::default()
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

    /// A rename (`git diff -M`) that also changes logic keeps the source
    /// path, so a consensus file cannot be moved out of the gate's patterns.
    #[test]
    fn test_rename_with_logic_change_keeps_old_filename() {
        let diff = "diff --git a/src/block.rs b/src/blk.rs\n\
similarity index 80%\n\
rename from src/block.rs\n\
rename to src/blk.rs\n\
index 1111111..2222222 100644\n\
--- a/src/block.rs\n\
+++ b/src/blk.rs\n\
@@ -10,2 +10,2 @@\n\
-    if height > 0 { return Err(Invalid); }\n\
+    return Ok(());\n";
        let f = DiffParser::parse_unified_diff(diff).unwrap();
        assert_eq!(f.len(), 1);
        assert_eq!(f[0].filename, "src/blk.rs");
        assert_eq!(f[0].old_filename.as_deref(), Some("src/block.rs"));
        assert!(!f[0].binary);
        assert_eq!(f[0].additions, vec!["    return Ok(());"]);
        assert!(!DiffParser::is_import_only_changes(&f[0]));
    }

    /// A 100% rename has no `---`/`+++` lines and no hunks; the source path
    /// must still come from `rename from`.
    #[test]
    fn test_pure_rename_without_hunks_keeps_old_filename() {
        let diff = "diff --git a/src/block.rs b/src/blk.rs\n\
similarity index 100%\n\
rename from src/block.rs\n\
rename to src/blk.rs\n";
        let f = DiffParser::parse_unified_diff(diff).unwrap();
        assert_eq!(f.len(), 1);
        assert_eq!(f[0].filename, "src/blk.rs");
        assert_eq!(f[0].old_filename.as_deref(), Some("src/block.rs"));
        assert!(f[0].additions.is_empty() && f[0].deletions.is_empty());
    }

    #[test]
    fn test_copy_from_sets_old_filename() {
        let diff = "diff --git a/src/pow.rs b/src/pow2.rs\n\
similarity index 100%\n\
copy from src/pow.rs\n\
copy to src/pow2.rs\n";
        let f = DiffParser::parse_unified_diff(diff).unwrap();
        assert_eq!(f[0].old_filename.as_deref(), Some("src/pow.rs"));
    }

    /// Plain unified diff: a `---` path that differs from `+++` is a rename.
    #[test]
    fn test_plain_diff_differing_minus_path_is_old_filename() {
        let diff = "--- a/src/block.rs\n+++ b/src/blk.rs\n@@ -1 +1 @@\n-a\n+b\n";
        let f = DiffParser::parse_unified_diff(diff).unwrap();
        assert_eq!(f[0].filename, "src/blk.rs");
        assert_eq!(f[0].old_filename.as_deref(), Some("src/block.rs"));
    }

    /// Same path on both sides, new files and deleted files are not renames.
    #[test]
    fn test_same_path_new_and_deleted_files_have_no_old_filename() {
        let same = "--- a/src/block.rs\n+++ b/src/block.rs\n@@ -1 +1 @@\n-a\n+b\n";
        assert_eq!(
            DiffParser::parse_unified_diff(same).unwrap()[0].old_filename,
            None
        );
        let new_and_deleted = "--- /dev/null\n+++ b/src/new.rs\n@@ -0,0 +1 @@\n+x\n\
--- a/src/old.rs\n+++ /dev/null\n@@ -1 +0,0 @@\n-x\n";
        let f = DiffParser::parse_unified_diff(new_and_deleted).unwrap();
        assert_eq!(f[0].old_filename, None);
        assert_eq!(f[1].filename, "src/old.rs");
        assert_eq!(f[1].old_filename, None);
    }

    #[test]
    fn test_binary_file_in_consensus_path_is_flagged() {
        let diff = "diff --git a/src/consensus/table.bin b/src/consensus/table.bin\n\
index 1111111..2222222 100644\n\
Binary files a/src/consensus/table.bin and b/src/consensus/table.bin differ\n";
        let f = DiffParser::parse_unified_diff(diff).unwrap();
        assert_eq!(f.len(), 1);
        assert_eq!(f[0].filename, "src/consensus/table.bin");
        assert!(f[0].binary);
        // No lines to inspect, so the import-only check alone would pass it
        assert!(DiffParser::is_import_only_changes(&f[0]));

        let patch = "diff --git a/src/consensus/t.bin b/src/consensus/t.bin\n\
index 1111111..2222222 100644\n\
GIT binary patch\n\
literal 4\n\
LcmZQzWMT#Y01f~L\n";
        assert!(DiffParser::parse_unified_diff(patch).unwrap()[0].binary);
    }

    /// `---`/`+++` lookalikes inside a hunk are body lines: they must not
    /// rename the file, set old_filename, or be dropped.
    #[test]
    fn test_header_lookalike_body_lines_are_kept() {
        let diff = "diff --git a/src/block.rs b/src/block.rs\n\
index 1111111..2222222 100644\n\
--- a/src/block.rs\n\
+++ b/src/block.rs\n\
@@ -1,2 +1,2 @@\n\
--- a/x\n\
+++ b/y\n";
        let f = DiffParser::parse_unified_diff(diff).unwrap();
        assert_eq!(f.len(), 1);
        assert_eq!(f[0].filename, "src/block.rs");
        assert_eq!(f[0].old_filename, None);
        assert_eq!(f[0].deletions, vec!["-- a/x"]);
        assert_eq!(f[0].additions, vec!["++ b/y"]);
    }

    /// A tab-separated timestamp after a header path is not part of the name.
    #[test]
    fn test_tab_timestamp_stripped_from_header_paths() {
        let diff = "--- a/src/block.rs\t2026-10-22 08:00:00.000000000 -0400\n\
+++ b/src/block.rs\t2026-10-22 09:00:00.000000000 -0400\n\
@@ -1 +1 @@\n-a\n+b\n";
        let f = DiffParser::parse_unified_diff(diff).unwrap();
        assert_eq!(f[0].filename, "src/block.rs");
        assert_eq!(f[0].old_filename, None);
    }

    /// With no `---`/`+++` lines (mode change, empty file), the name comes
    /// from the `b/` side of the `diff --git` line.
    #[test]
    fn test_filename_from_diff_git_line() {
        let diff = "diff --git a/src/script.rs b/src/script.rs\n\
old mode 100644\n\
new mode 100755\n";
        let f = DiffParser::parse_unified_diff(diff).unwrap();
        assert_eq!(f.len(), 1);
        assert_eq!(f[0].filename, "src/script.rs");
        assert_eq!(f[0].old_filename, None);
        assert!(!f[0].binary);
    }

    fn import_only(lines: &[&str]) -> bool {
        DiffParser::is_import_only_changes(&FileDiff {
            filename: "src/block.rs".to_string(),
            additions: lines.iter().map(|l| l.to_string()).collect(),
            ..Default::default()
        })
    }

    #[test]
    fn test_inline_mod_with_code_is_not_import_only() {
        assert!(!import_only(&[
            "mod evil { pub fn bypass() -> bool { true } }"
        ]));
        assert!(!import_only(&["pub mod evil {"]));
        // A `mod x;` declaration pulls in a whole new file; not an import
        assert!(!import_only(&["mod evil;"]));
    }

    #[test]
    fn test_use_followed_by_code_on_one_line_is_not_import_only() {
        assert!(!import_only(&["use x; pub fn bypass() {}"]));
        assert!(!import_only(&["use crate::a; return Ok(());"]));
        assert!(!import_only(&["use crate::{a, b}; fn f() {}"]));
        assert!(!import_only(&["use crate::a"])); // no `;`
    }

    #[test]
    fn test_block_comment_tricks_are_not_import_only() {
        assert!(!import_only(&["/* */ return Ok(());"]));
        assert!(!import_only(&["/* use crate::a; */ return Ok(());"]));
        assert!(!import_only(&["use crate::a; /* */ fn f() {}"]));
        assert!(!import_only(&["*/ return Ok(());"]));
        assert!(!import_only(&[" * hidden in a block comment"]));
        assert!(!import_only(&["/*"]));
    }

    /// A line comment that closes a block comment can end a `/*` opened
    /// on an earlier line and expose the code after it.
    #[test]
    fn test_line_comment_with_block_marker_is_not_import_only() {
        assert!(!import_only(&["// */"]));
        assert!(!import_only(&["/// /* doc"]));
        assert!(!import_only(&["use crate::a; // */"]));
    }

    #[test]
    fn test_unbalanced_attribute_is_not_import_only() {
        assert!(!import_only(&["#[use crate::a;"]));
        assert!(!import_only(&["#[cfg(test) use crate::a;"]));
        assert!(!import_only(&["#[cfg(any()]] use crate::a;"]));
    }

    #[test]
    fn test_standalone_attribute_is_not_import_only() {
        assert!(!import_only(&["#[cfg(any())]"]));
        assert!(!import_only(&["#[cfg(feature = \"x\")] fn f() {}"]));
        assert!(!import_only(&["#[cfg(test)] use x; fn f() {}"]));
    }

    #[test]
    fn test_real_imports_are_import_only() {
        assert!(import_only(&[
            "use crate::bip_validation;",
            "use crate::types::{Hash, Network};",
            "use crate::{self, a::{b, c}};",
            "use ::std::collections::HashMap;",
            "#[cfg(feature = \"std\")] use std::vec::Vec;",
            "#[cfg(test)] use crate::test_util;",
            "#[cfg(test)] #[cfg(feature = \"std\")] use std::vec::Vec;",
            "use crate::x; // trailing comment",
            "/// doc comment",
            "//! module doc",
            "// note",
            "    ",
        ]));
    }

    /// An attribute macro can expand to any code, so only `#[cfg(..)]` may
    /// precede an import.
    #[test]
    fn test_non_cfg_attribute_is_not_import_only() {
        assert!(!import_only(&["#[evil_attr] use crate::a;"]));
        assert!(!import_only(&["#[evil_attr(x)] use crate::a;"]));
        assert!(!import_only(&["#[cfg(test)] #[evil_attr] use crate::a;"]));
        assert!(!import_only(&["#[cfgx(test)] use crate::a;"]));
    }

    #[test]
    fn test_cfg_attr_is_not_import_only() {
        assert!(!import_only(&[
            "#[cfg_attr(test, evil_attr)] use crate::a;"
        ]));
    }

    #[test]
    fn test_macro_use_is_not_import_only() {
        assert!(!import_only(&["#[macro_use] use crate::a;"]));
        assert!(!import_only(&["#[macro_use] extern crate evil;"]));
    }

    #[test]
    fn test_extern_crate_is_not_import_only() {
        assert!(!import_only(&["extern crate alloc;"]));
        assert!(!import_only(&["extern crate core as c;"]));
        assert!(!import_only(&["pub extern crate alloc;"]));
        assert!(!import_only(&["#[cfg(test)] extern crate std;"]));
    }

    /// Re-exports change the crate's public API.
    #[test]
    fn test_pub_use_is_not_import_only() {
        assert!(!import_only(&["pub use crate::a;"]));
        assert!(!import_only(&["pub(crate) use crate::a;"]));
        assert!(!import_only(&["pub(super) use crate::{a, b};"]));
        assert!(!import_only(&["pub(in crate::x) use crate::a;"]));
        assert!(!import_only(&["#[cfg(test)] pub use crate::a;"]));
    }

    /// A rename can shadow the name a consensus function calls.
    #[test]
    fn test_as_rename_is_not_import_only() {
        assert!(!import_only(&["use other::check as check_tx;"]));
        assert!(!import_only(&["use crate::{a, b as c};"]));
        assert!(!import_only(&["use crate::{a::{b as c}};"]));
        assert!(!import_only(&["use crate::x as _;"]));
    }

    /// A glob can bring in a name that shadows one consensus code calls.
    #[test]
    fn test_glob_import_is_not_import_only() {
        assert!(!import_only(&["use super::*;"]));
        assert!(!import_only(&["use crate::{a, *};"]));
        assert!(!import_only(&["use crate::{a::{*}};"]));
        assert!(!import_only(&["use crate::a::{ * };"]));
    }
}
