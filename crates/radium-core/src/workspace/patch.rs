//! Patch application types and schemas.
//!
//! This module defines the input and output formats for the apply_patch tool,
//! supporting both unified diff format and structured hunks.

use crate::workspace::boundary::BoundaryValidator;
use crate::workspace::errors::{ErrorContext, FileOperationError, FileOperationResult};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Input format for patch application.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchInput {
    /// The patch content in unified diff format or structured hunks.
    pub patch: PatchContent,

    /// Whether to perform a dry-run (preview without applying).
    #[serde(default)]
    pub dry_run: bool,

    /// Whether to allow creating new files.
    #[serde(default = "default_true")]
    pub allow_create: bool,

    /// Expected file hash for validation (optional).
    pub expected_hash: Option<String>,

    /// Additional options for patch application.
    #[serde(default)]
    pub options: PatchOptions,
}

fn default_true() -> bool {
    true
}

/// Patch content format.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "format", rename_all = "snake_case")]
pub enum PatchContent {
    /// Unified diff format (standard git diff format).
    UnifiedDiff {
        /// The unified diff content as a string.
        content: String,
    },
    /// Structured hunks format (more explicit, easier to validate).
    StructuredHunks {
        /// List of file patches.
        files: Vec<FilePatch>,
    },
}

/// A patch for a single file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilePatch {
    /// Path to the file (relative to workspace root).
    pub path: String,

    /// List of hunks to apply.
    pub hunks: Vec<Hunk>,
}

/// A single hunk (change block) in a patch.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hunk {
    /// Starting line number in the original file (1-indexed).
    pub old_start: usize,

    /// Number of lines in the original file to replace.
    pub old_count: usize,

    /// Starting line number in the new file (1-indexed).
    pub new_start: usize,

    /// Number of lines in the new file.
    pub new_count: usize,

    /// Context lines before the change (for validation).
    pub context_before: Vec<String>,

    /// Lines to remove (from original file).
    pub removed_lines: Vec<String>,

    /// Lines to add (to new file).
    pub added_lines: Vec<String>,

    /// Context lines after the change (for validation).
    pub context_after: Vec<String>,
}

/// Options for patch application.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PatchOptions {
    /// Number of context lines to require for matching (default: 3).
    #[serde(default = "default_context_lines")]
    pub context_lines: usize,

    /// Whether to ignore whitespace differences.
    #[serde(default)]
    pub ignore_whitespace: bool,

    /// Whether to allow fuzz factor (matching with slight line number offsets).
    #[serde(default)]
    pub allow_fuzz: bool,

    /// Maximum fuzz factor (number of lines to search for context match).
    #[serde(default = "default_fuzz")]
    pub max_fuzz: usize,
}

fn default_context_lines() -> usize {
    3
}

fn default_fuzz() -> usize {
    2
}

/// Result of patch application.
#[derive(Debug, Clone)]
pub struct PatchResult {
    /// Whether the patch was successfully applied.
    pub success: bool,

    /// List of files that were changed.
    pub changed_files: Vec<ChangedFile>,

    /// List of errors encountered during application.
    pub errors: Vec<FileOperationError>,

    /// Summary of the patch application.
    pub summary: PatchSummary,
}

/// Information about a changed file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangedFile {
    /// Path to the file (canonicalized).
    pub path: PathBuf,

    /// Whether the file was created (didn't exist before).
    pub created: bool,

    /// Whether the file was deleted.
    pub deleted: bool,

    /// Diff showing the changes (unified diff format).
    pub diff: String,

    /// Number of hunks applied.
    pub hunks_applied: usize,

    /// Number of lines added.
    pub lines_added: usize,

    /// Number of lines removed.
    pub lines_removed: usize,
}

/// Summary of patch application.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchSummary {
    /// Total number of files in the patch.
    pub total_files: usize,

    /// Number of files successfully changed.
    pub files_changed: usize,

    /// Number of files that failed.
    pub files_failed: usize,

    /// Total number of hunks in the patch.
    pub total_hunks: usize,

    /// Number of hunks successfully applied.
    pub hunks_applied: usize,

    /// Number of hunks that failed (conflicts, context mismatch).
    pub hunks_failed: usize,

    /// Total number of lines added.
    pub total_lines_added: usize,

    /// Total number of lines removed.
    pub total_lines_removed: usize,
}

impl Default for PatchSummary {
    fn default() -> Self {
        Self {
            total_files: 0,
            files_changed: 0,
            files_failed: 0,
            total_hunks: 0,
            hunks_applied: 0,
            hunks_failed: 0,
            total_lines_added: 0,
            total_lines_removed: 0,
        }
    }
}

impl PatchResult {
    /// Create a successful patch result.
    pub fn success(changed_files: Vec<ChangedFile>) -> Self {
        let summary = PatchSummary {
            total_files: changed_files.len(),
            files_changed: changed_files.len(),
            files_failed: 0,
            total_hunks: changed_files.iter().map(|f| f.hunks_applied).sum(),
            hunks_applied: changed_files.iter().map(|f| f.hunks_applied).sum(),
            hunks_failed: 0,
            total_lines_added: changed_files.iter().map(|f| f.lines_added).sum(),
            total_lines_removed: changed_files.iter().map(|f| f.lines_removed).sum(),
        };

        Self {
            success: true,
            changed_files,
            errors: Vec::new(),
            summary,
        }
    }

    /// Create a failed patch result.
    pub fn failure(errors: Vec<FileOperationError>) -> Self {
        Self {
            success: false,
            changed_files: Vec::new(),
            errors,
            summary: PatchSummary::default(),
        }
    }

    /// Check if the result has any errors.
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty() || !self.success
    }
}

/// Patch applicator for applying patches to files.
pub struct PatchApplicator {
    /// Workspace root for boundary validation.
    workspace_root: PathBuf,
    /// Boundary validator.
    boundary_validator: BoundaryValidator,
}

impl PatchApplicator {
    /// Create a new patch applicator.
    ///
    /// # Errors
    /// Returns error if workspace root cannot be canonicalized.
    pub fn new(workspace_root: impl AsRef<Path>) -> FileOperationResult<Self> {
        let root = workspace_root.as_ref().to_path_buf();
        let validator = BoundaryValidator::new(&root)
            .map_err(|e| FileOperationError::IoError {
                path: root.display().to_string(),
                operation: "initialize patch applicator".to_string(),
                source: std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("Failed to create boundary validator: {}", e),
                ),
            })?;

        Ok(Self {
            workspace_root: root,
            boundary_validator: validator,
        })
    }

    /// Apply a patch to files.
    ///
    /// # Arguments
    /// * `input` - The patch input containing patch content and options
    ///
    /// # Returns
    /// Patch result with changed files and any errors
    pub fn apply(&self, input: &PatchInput) -> PatchResult {
        let mut result = PatchResult {
            success: true,
            changed_files: Vec::new(),
            errors: Vec::new(),
            summary: PatchSummary::default(),
        };

        // Parse patch content
        let file_patches = match self.parse_patch(&input.patch) {
            Ok(patches) => patches,
            Err(e) => {
                result.errors.push(e);
                result.success = false;
                return result;
            }
        };

        result.summary.total_files = file_patches.len();
        result.summary.total_hunks = file_patches
            .iter()
            .map(|fp| fp.hunks.len())
            .sum();

        // Apply each file patch
        for file_patch in file_patches {
            match self.apply_file_patch(&file_patch, input) {
                Ok(changed_file) => {
                    result.changed_files.push(changed_file);
                    result.summary.files_changed += 1;
                }
                Err(e) => {
                    result.errors.push(e);
                    result.summary.files_failed += 1;
                    result.success = false;
                }
            }
        }

        // Update summary
        result.summary.hunks_applied = result
            .changed_files
            .iter()
            .map(|f| f.hunks_applied)
            .sum();
        result.summary.hunks_failed = result.summary.total_hunks - result.summary.hunks_applied;
        result.summary.total_lines_added = result
            .changed_files
            .iter()
            .map(|f| f.lines_added)
            .sum();
        result.summary.total_lines_removed = result
            .changed_files
            .iter()
            .map(|f| f.lines_removed)
            .sum();

        result
    }

    /// Parse patch content into file patches.
    fn parse_patch(&self, content: &PatchContent) -> FileOperationResult<Vec<FilePatch>> {
        match content {
            PatchContent::UnifiedDiff { content } => self.parse_unified_diff(content),
            PatchContent::StructuredHunks { files } => Ok(files.clone()),
        }
    }

    /// Parse unified diff format.
    fn parse_unified_diff(&self, diff: &str) -> FileOperationResult<Vec<FilePatch>> {
        let mut file_patches = Vec::new();
        let mut current_file: Option<FilePatch> = None;
        let mut current_hunk: Option<Hunk> = None;
        let mut in_hunk = false;
        let mut hunk_header_line: Option<String> = None;

        for line in diff.lines() {
            // File header: --- a/path or +++ b/path
            if line.starts_with("--- ") {
                // Save previous file if exists
                if let Some(file) = current_file.take() {
                    file_patches.push(file);
                }

                let path = line
                    .strip_prefix("--- ")
                    .and_then(|s| s.split_whitespace().next())
                    .map(|s| s.strip_prefix("a/").unwrap_or(s).to_string())
                    .ok_or_else(|| FileOperationError::InvalidInput {
                        operation: "parse_unified_diff".to_string(),
                        field: "file_header".to_string(),
                        reason: format!("Invalid file header: {}", line),
                    })?;

                current_file = Some(FilePatch {
                    path,
                    hunks: Vec::new(),
                });
            } else if line.starts_with("+++ ") {
                // File path in +++ line (already handled in ---)
                continue;
            }
            // Hunk header: @@ -old_start,old_count +new_start,new_count @@
            else if line.starts_with("@@ ") {
                // Save previous hunk if exists
                if let Some(hunk) = current_hunk.take() {
                    if let Some(ref mut file) = current_file {
                        file.hunks.push(hunk);
                    }
                }

                let _hunk_header_line = Some(line.to_string());
                in_hunk = true;

                // Parse hunk header
                let hunk = self.parse_hunk_header(line)?;
                current_hunk = Some(hunk);
            } else if in_hunk {
                // Hunk content
                if let Some(ref mut hunk) = current_hunk {
                    if line.starts_with(' ') {
                        // Context line
                        let content = line.strip_prefix(' ').unwrap_or(line).to_string();
                        if hunk.removed_lines.is_empty() && hunk.added_lines.is_empty() {
                            hunk.context_before.push(content);
                        } else {
                            hunk.context_after.push(content);
                        }
                    } else if line.starts_with('-') {
                        // Removed line
                        let content = line.strip_prefix('-').unwrap_or(line).to_string();
                        hunk.removed_lines.push(content);
                    } else if line.starts_with('+') {
                        // Added line
                        let content = line.strip_prefix('+').unwrap_or(line).to_string();
                        hunk.added_lines.push(content);
                    }
                }
            }
        }

        // Save last hunk and file
        if let Some(hunk) = current_hunk.take() {
            if let Some(ref mut file) = current_file {
                file.hunks.push(hunk);
            }
        }

        if let Some(file) = current_file.take() {
            file_patches.push(file);
        }

        Ok(file_patches)
    }

    /// Parse a hunk header line.
    fn parse_hunk_header(&self, line: &str) -> FileOperationResult<Hunk> {
        // Format: @@ -old_start,old_count +new_start,new_count @@
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 3 {
            return Err(FileOperationError::InvalidInput {
                operation: "parse_hunk_header".to_string(),
                field: "hunk_header".to_string(),
                reason: format!("Invalid hunk header format: {}", line),
            });
        }

        let old_part = parts[1].strip_prefix('-').ok_or_else(|| {
            FileOperationError::InvalidInput {
                operation: "parse_hunk_header".to_string(),
                field: "old_range".to_string(),
                reason: format!("Invalid old range: {}", parts[1]),
            }
        })?;

        let new_part = parts[2].strip_prefix('+').ok_or_else(|| {
            FileOperationError::InvalidInput {
                operation: "parse_hunk_header".to_string(),
                field: "new_range".to_string(),
                reason: format!("Invalid new range: {}", parts[2]),
            }
        })?;

        let (old_start, old_count) = self.parse_range(old_part)?;
        let (new_start, new_count) = self.parse_range(new_part)?;

        Ok(Hunk {
            old_start,
            old_count,
            new_start,
            new_count,
            context_before: Vec::new(),
            removed_lines: Vec::new(),
            added_lines: Vec::new(),
            context_after: Vec::new(),
        })
    }

    /// Parse a range (e.g., "5,3" -> (5, 3) or "5" -> (5, 1)).
    fn parse_range(&self, range: &str) -> FileOperationResult<(usize, usize)> {
        if let Some(comma_pos) = range.find(',') {
            let start = range[..comma_pos]
                .parse::<usize>()
                .map_err(|_| FileOperationError::InvalidInput {
                    operation: "parse_range".to_string(),
                    field: "start".to_string(),
                    reason: format!("Invalid start value: {}", range),
                })?;
            let count = range[comma_pos + 1..]
                .parse::<usize>()
                .map_err(|_| FileOperationError::InvalidInput {
                    operation: "parse_range".to_string(),
                    field: "count".to_string(),
                    reason: format!("Invalid count value: {}", range),
                })?;
            Ok((start, count))
        } else {
            let start = range
                .parse::<usize>()
                .map_err(|_| FileOperationError::InvalidInput {
                    operation: "parse_range".to_string(),
                    field: "start".to_string(),
                    reason: format!("Invalid range: {}", range),
                })?;
            Ok((start, 1))
        }
    }

    /// Apply a file patch.
    fn apply_file_patch(
        &self,
        file_patch: &FilePatch,
        input: &PatchInput,
    ) -> FileOperationResult<ChangedFile> {
        // Validate path
        let validated_path = self
            .boundary_validator
            .validate_path(&file_patch.path, false)
            .map_err(FileOperationError::from)?;

        // Read existing file or create empty
        let file_exists = validated_path.exists();
        let current_content = if file_exists {
            fs::read_to_string(&validated_path).map_err(|e| FileOperationError::IoError {
                path: validated_path.display().to_string(),
                operation: "read_file".to_string(),
                source: e,
            })?
        } else {
            if !input.allow_create {
                return Err(FileOperationError::PathNotFound {
                    path: validated_path.display().to_string(),
                    operation: "apply_patch".to_string(),
                });
            }
            String::new()
        };

        let mut lines: Vec<String> = current_content.lines().map(|s| s.to_string()).collect();
        let mut hunks_applied = 0;
        let mut lines_added = 0;
        let mut lines_removed = 0;
        let mut errors = Vec::new();

        // Apply hunks in reverse order to maintain line numbers
        let mut sorted_hunks = file_patch.hunks.clone();
        sorted_hunks.sort_by_key(|h| h.old_start);
        sorted_hunks.reverse();

        for hunk in &sorted_hunks {
            match self.apply_hunk(&mut lines, hunk, &input.options) {
                Ok((added, removed)) => {
                    hunks_applied += 1;
                    lines_added += added;
                    lines_removed += removed;
                }
                Err(e) => {
                    errors.push(e);
                }
            }
        }

        if !errors.is_empty() {
            return Err(errors.into_iter().next().unwrap());
        }

        // Generate diff
        let diff = self.generate_diff(&current_content, &lines.join("\n"), &file_patch.path);

        // Apply changes if not dry-run
        if !input.dry_run {
            fs::write(&validated_path, lines.join("\n")).map_err(|e| {
                FileOperationError::IoError {
                    path: validated_path.display().to_string(),
                    operation: "write_file".to_string(),
                    source: e,
                }
            })?;
        }

        Ok(ChangedFile {
            path: validated_path,
            created: !file_exists,
            deleted: false,
            diff,
            hunks_applied,
            lines_added,
            lines_removed,
        })
    }

    /// Apply a single hunk to file lines.
    fn apply_hunk(
        &self,
        lines: &mut Vec<String>,
        hunk: &Hunk,
        options: &PatchOptions,
    ) -> FileOperationResult<(usize, usize)> {
        // Convert to 0-indexed
        let start_idx = hunk.old_start.saturating_sub(1);
        let end_idx = start_idx + hunk.old_count;

        // Validate context
        if !self.validate_context(lines, hunk, start_idx, options)? {
            return Err(FileOperationError::PatchConflict {
                file: "unknown".to_string(),
                line_number: hunk.old_start,
                expected: format!("{:?}", hunk.context_before),
                actual: format!("{:?}", lines.get(start_idx..start_idx + hunk.context_before.len())),
            });
        }

        // Remove old lines
        let removed_count = hunk.removed_lines.len();
        if end_idx <= lines.len() {
            lines.drain(start_idx..end_idx);
        }

        // Insert new lines
        let added_count = hunk.added_lines.len();
        for (i, new_line) in hunk.added_lines.iter().enumerate() {
            lines.insert(start_idx + i, new_line.clone());
        }

        Ok((added_count, removed_count))
    }

    /// Validate that context matches.
    fn validate_context(
        &self,
        lines: &[String],
        hunk: &Hunk,
        start_idx: usize,
        options: &PatchOptions,
    ) -> FileOperationResult<bool> {
        // Check context before
        if !hunk.context_before.is_empty() {
            let context_start = start_idx.saturating_sub(hunk.context_before.len());
            if context_start + hunk.context_before.len() > lines.len() {
                return Ok(false);
            }

            let actual_context: Vec<&str> = lines[context_start..context_start + hunk.context_before.len()]
                .iter()
                .map(|s| s.as_str())
                .collect();
            let expected_context: Vec<&str> = hunk.context_before.iter().map(|s| s.as_str()).collect();

            if !self.compare_lines(&actual_context, &expected_context, options) {
                return Ok(false);
            }
        }

        // Check context after (if we have removed lines)
        if !hunk.context_after.is_empty() && !hunk.removed_lines.is_empty() {
            let after_start = start_idx + hunk.removed_lines.len();
            if after_start + hunk.context_after.len() > lines.len() {
                return Ok(false);
            }

            let actual_context: Vec<&str> = lines[after_start..after_start + hunk.context_after.len()]
                .iter()
                .map(|s| s.as_str())
                .collect();
            let expected_context: Vec<&str> = hunk.context_after.iter().map(|s| s.as_str()).collect();

            if !self.compare_lines(&actual_context, &expected_context, options) {
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Compare lines with optional whitespace ignoring.
    fn compare_lines(&self, actual: &[&str], expected: &[&str], options: &PatchOptions) -> bool {
        if actual.len() != expected.len() {
            return false;
        }

        for (a, e) in actual.iter().zip(expected.iter()) {
            if options.ignore_whitespace {
                if a.trim() != e.trim() {
                    return false;
                }
            } else if a != e {
                return false;
            }
        }

        true
    }

    /// Generate a unified diff for the changes.
    fn generate_diff(&self, old_content: &str, new_content: &str, path: &str) -> String {
        let old_lines: Vec<&str> = old_content.lines().collect();
        let new_lines: Vec<&str> = new_content.lines().collect();

        let mut diff = format!("--- a/{}\n+++ b/{}\n", path, path);

        // Simple diff generation (could be enhanced with proper diff algorithm)
        let mut old_idx = 0;
        let mut new_idx = 0;

        while old_idx < old_lines.len() || new_idx < new_lines.len() {
            if old_idx < old_lines.len() && new_idx < new_lines.len() {
                if old_lines[old_idx] == new_lines[new_idx] {
                    diff.push_str(&format!(" {}\n", old_lines[old_idx]));
                    old_idx += 1;
                    new_idx += 1;
                } else if old_idx + 1 < old_lines.len()
                    && old_lines[old_idx + 1] == new_lines[new_idx]
                {
                    diff.push_str(&format!("-{}\n", old_lines[old_idx]));
                    old_idx += 1;
                } else {
                    diff.push_str(&format!("+{}\n", new_lines[new_idx]));
                    new_idx += 1;
                }
            } else if old_idx < old_lines.len() {
                diff.push_str(&format!("-{}\n", old_lines[old_idx]));
                old_idx += 1;
            } else {
                diff.push_str(&format!("+{}\n", new_lines[new_idx]));
                new_idx += 1;
            }
        }

        diff
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_patch_input_defaults() {
        let input = PatchInput {
            patch: PatchContent::UnifiedDiff {
                content: "--- a/file.txt\n+++ b/file.txt\n@@ -1,1 +1,1 @@\n-old\n+new".to_string(),
            },
            dry_run: false,
            allow_create: true,
            expected_hash: None,
            options: PatchOptions::default(),
        };

        assert!(!input.dry_run);
        assert!(input.allow_create);
        assert_eq!(input.options.context_lines, 3);
    }

    #[test]
    fn test_patch_result_success() {
        let changed_file = ChangedFile {
            path: PathBuf::from("test.txt"),
            created: false,
            deleted: false,
            diff: "--- a/test.txt\n+++ b/test.txt\n@@ -1,1 +1,1 @@\n-old\n+new".to_string(),
            hunks_applied: 1,
            lines_added: 1,
            lines_removed: 1,
        };

        let result = PatchResult::success(vec![changed_file]);
        assert!(result.success);
        assert_eq!(result.changed_files.len(), 1);
        assert_eq!(result.summary.files_changed, 1);
        assert_eq!(result.summary.total_lines_added, 1);
    }

    #[test]
    fn test_patch_result_failure() {
        let error = FileOperationError::PatchConflict {
            file: "test.txt".to_string(),
            line_number: 10,
            expected: "old".to_string(),
            actual: "different".to_string(),
        };

        let result = PatchResult::failure(vec![error]);
        assert!(!result.success);
        assert!(result.has_errors());
        assert_eq!(result.errors.len(), 1);
    }

    #[test]
    fn test_structured_hunks_format() {
        let hunk = Hunk {
            old_start: 5,
            old_count: 2,
            new_start: 5,
            new_count: 3,
            context_before: vec!["line 3".to_string(), "line 4".to_string()],
            removed_lines: vec!["line 5".to_string(), "line 6".to_string()],
            added_lines: vec!["new line 5".to_string(), "new line 6".to_string(), "new line 7".to_string()],
            context_after: vec!["line 7".to_string()],
        };

        let file_patch = FilePatch {
            path: "test.txt".to_string(),
            hunks: vec![hunk],
        };

        let patch = PatchContent::StructuredHunks {
            files: vec![file_patch],
        };

        match patch {
            PatchContent::StructuredHunks { files } => {
                assert_eq!(files.len(), 1);
                assert_eq!(files[0].hunks.len(), 1);
            }
            _ => panic!("Expected StructuredHunks"),
        }
    }

    #[test]
    fn test_patch_applicator_apply_simple_patch() {
        use tempfile::TempDir;

        let temp = TempDir::new().unwrap();
        let applicator = PatchApplicator::new(temp.path()).unwrap();

        // Create a test file
        let test_file = temp.path().join("test.txt");
        fs::write(&test_file, "line 1\nline 2\nline 3\n").unwrap();

        // Create a patch
        let patch = PatchInput {
            patch: PatchContent::UnifiedDiff {
                content: "--- a/test.txt\n+++ b/test.txt\n@@ -2,1 +2,1 @@\n-line 2\n+line 2 modified\n".to_string(),
            },
            dry_run: false,
            allow_create: true,
            expected_hash: None,
            options: PatchOptions::default(),
        };

        let result = applicator.apply(&patch);
        assert!(result.success);
        assert_eq!(result.changed_files.len(), 1);
        assert_eq!(result.changed_files[0].hunks_applied, 1);

        // Verify file was modified
        let content = fs::read_to_string(&test_file).unwrap();
        assert!(content.contains("line 2 modified"));
    }

    #[test]
    fn test_patch_applicator_dry_run() {
        use tempfile::TempDir;

        let temp = TempDir::new().unwrap();
        let applicator = PatchApplicator::new(temp.path()).unwrap();

        // Create a test file
        let test_file = temp.path().join("test.txt");
        fs::write(&test_file, "original\n").unwrap();

        let original_content = fs::read_to_string(&test_file).unwrap();

        // Create a patch with dry_run
        let patch = PatchInput {
            patch: PatchContent::UnifiedDiff {
                content: "--- a/test.txt\n+++ b/test.txt\n@@ -1,1 +1,1 @@\n-original\n+modified\n".to_string(),
            },
            dry_run: true,
            allow_create: true,
            expected_hash: None,
            options: PatchOptions::default(),
        };

        let result = applicator.apply(&patch);
        assert!(result.success);

        // Verify file was NOT modified
        let content = fs::read_to_string(&test_file).unwrap();
        assert_eq!(content, original_content);
    }

    #[test]
    fn test_patch_applicator_create_file() {
        use tempfile::TempDir;

        let temp = TempDir::new().unwrap();
        let applicator = PatchApplicator::new(temp.path()).unwrap();

        // Create a patch for a new file
        let patch = PatchInput {
            patch: PatchContent::UnifiedDiff {
                content: "--- /dev/null\n+++ b/new.txt\n@@ -0,0 +1,1 @@\n+new content\n".to_string(),
            },
            dry_run: false,
            allow_create: true,
            expected_hash: None,
            options: PatchOptions::default(),
        };

        let result = applicator.apply(&patch);
        assert!(result.success);
        assert_eq!(result.changed_files.len(), 1);
        assert!(result.changed_files[0].created);

        // Verify file was created
        let new_file = temp.path().join("new.txt");
        assert!(new_file.exists());
    }
}
