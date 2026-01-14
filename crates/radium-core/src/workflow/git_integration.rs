//! Git integration utilities for workflow execution.
//!
//! This module provides functionality to extract git commits from execution output
//! and query git repositories for commit information.

use super::report_generator::CommitInfo;
use chrono::{DateTime, Utc};
use regex::Regex;
use std::path::Path;
use std::process::Command;

/// Extract git commit hashes from agent output text.
///
/// This function searches for git commit hashes (40-character hex strings)
/// in the output text and returns them as a vector.
///
/// # Arguments
///
/// * `output` - The text output to search for commit hashes
///
/// # Returns
///
/// A vector of commit hash strings found in the output
pub fn extract_commits_from_output(output: &str) -> Vec<String> {
    let mut commits = Vec::new();

    // Regex for full commit hash (40 hex characters)
    let commit_regex = Regex::new(r"\b([0-9a-f]{40})\b").unwrap();

    // Also check for common git output patterns
    let commit_line_regex = Regex::new(r"(?:commit|Commit:?)\s+([0-9a-f]{40})").unwrap();

    // Extract from explicit commit lines first
    for cap in commit_line_regex.captures_iter(output) {
        if let Some(hash) = cap.get(1) {
            let hash_str = hash.as_str().to_string();
            if !commits.contains(&hash_str) {
                commits.push(hash_str);
            }
        }
    }

    // Then extract any standalone hashes
    for cap in commit_regex.captures_iter(output) {
        if let Some(hash) = cap.get(1) {
            let hash_str = hash.as_str().to_string();
            if !commits.contains(&hash_str) {
                commits.push(hash_str);
            }
        }
    }

    commits
}

/// Get git commit information for specific commit hashes.
///
/// # Arguments
///
/// * `repo_path` - Path to the git repository
/// * `commit_hashes` - List of commit hashes to query
///
/// # Returns
///
/// A vector of CommitInfo for the requested commits
pub fn get_commit_info(repo_path: &Path, commit_hashes: &[String]) -> Vec<CommitInfo> {
    let mut commit_infos = Vec::new();

    for hash in commit_hashes {
        if let Ok(info) = get_single_commit_info(repo_path, hash) {
            commit_infos.push(info);
        }
    }

    commit_infos
}

/// Get commit information for a single commit.
fn get_single_commit_info(repo_path: &Path, commit_hash: &str) -> Result<CommitInfo, String> {
    // Format: hash|author|timestamp|message
    let output = Command::new("git")
        .current_dir(repo_path)
        .args(&[
            "log",
            "-1",
            "--format=%H|%an|%aI|%s",
            commit_hash,
        ])
        .output()
        .map_err(|e| format!("Failed to run git command: {}", e))?;

    if !output.status.success() {
        return Err(format!("Git command failed for commit {}", commit_hash));
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    let parts: Vec<&str> = output_str.trim().split('|').collect();

    if parts.len() < 4 {
        return Err(format!("Invalid git output for commit {}", commit_hash));
    }

    let timestamp = DateTime::parse_from_rfc3339(parts[2])
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now());

    Ok(CommitInfo {
        hash: parts[0].to_string(),
        author: parts[1].to_string(),
        timestamp,
        message: parts[3].to_string(),
        url: None, // Could be constructed from repo config
    })
}

/// Get recent commits from a git repository since a specific time.
///
/// # Arguments
///
/// * `repo_path` - Path to the git repository
/// * `since` - Only get commits after this time
///
/// # Returns
///
/// A vector of CommitInfo for commits since the specified time
pub fn get_recent_commits(repo_path: &Path, since: DateTime<Utc>) -> Vec<CommitInfo> {
    let since_str = since.to_rfc3339();

    let output = Command::new("git")
        .current_dir(repo_path)
        .args(&[
            "log",
            &format!("--since={}", since_str),
            "--format=%H|%an|%aI|%s",
        ])
        .output();

    let output = match output {
        Ok(o) if o.status.success() => o,
        _ => return Vec::new(),
    };

    let output_str = String::from_utf8_lossy(&output.stdout);
    let mut commits = Vec::new();

    for line in output_str.lines() {
        let parts: Vec<&str> = line.split('|').collect();
        if parts.len() < 4 {
            continue;
        }

        let timestamp = DateTime::parse_from_rfc3339(parts[2])
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());

        commits.push(CommitInfo {
            hash: parts[0].to_string(),
            author: parts[1].to_string(),
            timestamp,
            message: parts[3].to_string(),
            url: None,
        });
    }

    commits
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_commits_from_output() {
        let output = r#"
        Commit: 1234567890abcdef1234567890abcdef12345678

        Made some changes and committed them.
        The commit hash is 0987654321fedcba0987654321fedcba09876543.
        "#;

        let commits = extract_commits_from_output(output);
        assert_eq!(commits.len(), 2);
        assert!(commits.contains(&"1234567890abcdef1234567890abcdef12345678".to_string()));
        assert!(commits.contains(&"0987654321fedcba0987654321fedcba09876543".to_string()));
    }

    #[test]
    fn test_extract_commits_deduplicate() {
        let output = r#"
        commit 1234567890abcdef1234567890abcdef12345678
        The same commit: 1234567890abcdef1234567890abcdef12345678
        "#;

        let commits = extract_commits_from_output(output);
        assert_eq!(commits.len(), 1);
    }

    #[test]
    fn test_extract_no_commits() {
        let output = "No commits here, just regular text.";
        let commits = extract_commits_from_output(output);
        assert_eq!(commits.len(), 0);
    }
}
