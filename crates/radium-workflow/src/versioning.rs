//! Component version management utilities.

use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

/// Parsed semantic version.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemVer {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub pre: Option<String>,
}

/// What kind of version bump is needed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BumpKind {
    Major,
    Minor,
    Patch,
}

impl SemVer {
    /// Parse a semver string like "1.2.3" or "1.2.3-beta.1".
    ///
    /// # Errors
    ///
    /// Returns an error string if the input is not a valid semver.
    pub fn parse(s: &str) -> Result<Self, String> {
        if s.is_empty() {
            return Err("version string is empty".to_string());
        }

        // Split on '-' to separate pre-release
        let (core, pre) = match s.split_once('-') {
            Some((core, pre)) => (core, Some(pre.to_string())),
            None => (s, None),
        };

        let parts: Vec<&str> = core.split('.').collect();
        if parts.len() != 3 {
            return Err(format!(
                "expected 3 dot-separated components, found {}: {s:?}",
                parts.len()
            ));
        }

        let parse_part = |part: &str, label: &str| -> Result<u32, String> {
            part.parse::<u32>()
                .map_err(|_| format!("invalid {label} component {part:?} in {s:?}"))
        };

        Ok(Self {
            major: parse_part(parts[0], "major")?,
            minor: parse_part(parts[1], "minor")?,
            patch: parse_part(parts[2], "patch")?,
            pre,
        })
    }

    /// Format back to string.
    #[must_use]
    pub fn to_version_string(&self) -> String {
        let base = format!("{}.{}.{}", self.major, self.minor, self.patch);
        match &self.pre {
            Some(pre) => format!("{base}-{pre}"),
            None => base,
        }
    }

    /// Apply a version bump.
    ///
    /// Bumping major resets minor and patch to 0. Bumping minor resets patch to 0.
    /// All bumps clear the pre-release label.
    #[must_use]
    pub fn bump(&self, kind: BumpKind) -> Self {
        match kind {
            BumpKind::Major => Self {
                major: self.major + 1,
                minor: 0,
                patch: 0,
                pre: None,
            },
            BumpKind::Minor => Self {
                major: self.major,
                minor: self.minor + 1,
                patch: 0,
                pre: None,
            },
            BumpKind::Patch => Self {
                major: self.major,
                minor: self.minor,
                patch: self.patch + 1,
                pre: None,
            },
        }
    }

}

impl PartialOrd for SemVer {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SemVer {
    /// Compare two versions.
    ///
    /// Pre-release versions are ordered before the release (e.g. 1.0.0-alpha < 1.0.0),
    /// following the semver spec.
    fn cmp(&self, other: &Self) -> Ordering {
        match self.major.cmp(&other.major) {
            Ordering::Equal => {}
            ord => return ord,
        }
        match self.minor.cmp(&other.minor) {
            Ordering::Equal => {}
            ord => return ord,
        }
        match self.patch.cmp(&other.patch) {
            Ordering::Equal => {}
            ord => return ord,
        }
        // Pre-release handling: no pre-release > has pre-release (1.0.0 > 1.0.0-alpha)
        match (&self.pre, &other.pre) {
            (None, None) => Ordering::Equal,
            (None, Some(_)) => Ordering::Greater,
            (Some(_), None) => Ordering::Less,
            (Some(a), Some(b)) => a.cmp(b),
        }
    }
}

impl std::fmt::Display for SemVer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_version_string())
    }
}

/// A version history entry for a component.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionEntry {
    pub version: String,
    pub released_at: String,
    pub description: Option<String>,
    pub breaking: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- parse valid versions ---

    #[test]
    fn test_parse_simple() {
        let v = SemVer::parse("1.0.0").unwrap();
        assert_eq!(v, SemVer { major: 1, minor: 0, patch: 0, pre: None });
    }

    #[test]
    fn test_parse_multi_digit() {
        let v = SemVer::parse("2.3.4").unwrap();
        assert_eq!(v, SemVer { major: 2, minor: 3, patch: 4, pre: None });
    }

    #[test]
    fn test_parse_pre_release() {
        let v = SemVer::parse("1.0.0-beta.1").unwrap();
        assert_eq!(v, SemVer { major: 1, minor: 0, patch: 0, pre: Some("beta.1".to_string()) });
    }

    #[test]
    fn test_parse_zero_minor() {
        let v = SemVer::parse("0.1.0").unwrap();
        assert_eq!(v, SemVer { major: 0, minor: 1, patch: 0, pre: None });
    }

    // --- parse invalid versions ---

    #[test]
    fn test_parse_empty_string() {
        assert!(SemVer::parse("").is_err());
    }

    #[test]
    fn test_parse_non_numeric() {
        assert!(SemVer::parse("abc").is_err());
    }

    #[test]
    fn test_parse_too_few_parts() {
        assert!(SemVer::parse("1.2").is_err());
    }

    #[test]
    fn test_parse_too_many_parts() {
        assert!(SemVer::parse("1.2.3.4").is_err());
    }

    // --- bump ---

    #[test]
    fn test_bump_major() {
        let v = SemVer::parse("1.2.3").unwrap();
        assert_eq!(v.bump(BumpKind::Major), SemVer { major: 2, minor: 0, patch: 0, pre: None });
    }

    #[test]
    fn test_bump_minor() {
        let v = SemVer::parse("1.2.3").unwrap();
        assert_eq!(v.bump(BumpKind::Minor), SemVer { major: 1, minor: 3, patch: 0, pre: None });
    }

    #[test]
    fn test_bump_patch() {
        let v = SemVer::parse("1.2.3").unwrap();
        assert_eq!(v.bump(BumpKind::Patch), SemVer { major: 1, minor: 2, patch: 4, pre: None });
    }

    #[test]
    fn test_bump_major_clears_pre_release() {
        let v = SemVer::parse("1.2.3-beta.1").unwrap();
        let bumped = v.bump(BumpKind::Major);
        assert_eq!(bumped, SemVer { major: 2, minor: 0, patch: 0, pre: None });
        assert!(bumped.pre.is_none());
    }

    #[test]
    fn test_bump_minor_clears_pre_release() {
        let v = SemVer::parse("1.2.3-beta.1").unwrap();
        let bumped = v.bump(BumpKind::Minor);
        assert!(bumped.pre.is_none());
    }

    #[test]
    fn test_bump_patch_clears_pre_release() {
        let v = SemVer::parse("1.2.3-beta.1").unwrap();
        let bumped = v.bump(BumpKind::Patch);
        assert!(bumped.pre.is_none());
    }

    // --- ordering ---

    #[test]
    fn test_patch_ordering() {
        let a = SemVer::parse("1.0.0").unwrap();
        let b = SemVer::parse("1.0.1").unwrap();
        assert!(a < b);
    }

    #[test]
    fn test_minor_ordering() {
        let a = SemVer::parse("1.0.1").unwrap();
        let b = SemVer::parse("1.1.0").unwrap();
        assert!(a < b);
    }

    #[test]
    fn test_major_ordering() {
        let a = SemVer::parse("1.1.0").unwrap();
        let b = SemVer::parse("2.0.0").unwrap();
        assert!(a < b);
    }

    #[test]
    fn test_full_ordering_chain() {
        let mut versions = vec![
            SemVer::parse("2.0.0").unwrap(),
            SemVer::parse("1.0.0").unwrap(),
            SemVer::parse("1.1.0").unwrap(),
            SemVer::parse("1.0.1").unwrap(),
        ];
        versions.sort();
        assert_eq!(
            versions.iter().map(|v| v.to_version_string()).collect::<Vec<_>>(),
            vec!["1.0.0", "1.0.1", "1.1.0", "2.0.0"]
        );
    }

    #[test]
    fn test_pre_release_less_than_release() {
        let pre = SemVer::parse("1.0.0-alpha").unwrap();
        let release = SemVer::parse("1.0.0").unwrap();
        assert!(pre < release);
    }

    #[test]
    fn test_equal_versions() {
        let a = SemVer::parse("1.2.3").unwrap();
        let b = SemVer::parse("1.2.3").unwrap();
        assert_eq!(a, b);
        assert!(!(a < b));
        assert!(!(a > b));
    }

    // --- display ---

    #[test]
    fn test_display_roundtrip() {
        assert_eq!(SemVer::parse("1.2.3").unwrap().to_version_string(), "1.2.3");
    }

    #[test]
    fn test_display_pre_release() {
        assert_eq!(
            SemVer::parse("1.0.0-beta.1").unwrap().to_version_string(),
            "1.0.0-beta.1"
        );
    }

    #[test]
    fn test_fmt_display() {
        let v = SemVer::parse("3.0.0").unwrap();
        assert_eq!(format!("{v}"), "3.0.0");
    }

    // --- VersionEntry serialization ---

    #[test]
    fn test_version_entry_roundtrip() {
        let entry = VersionEntry {
            version: "1.0.0".to_string(),
            released_at: "2024-01-01T00:00:00Z".to_string(),
            description: Some("Initial release".to_string()),
            breaking: false,
        };
        let json = serde_json::to_string(&entry).unwrap();
        let decoded: VersionEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.version, "1.0.0");
        assert!(!decoded.breaking);
    }

    // --- BumpKind serialization ---

    #[test]
    fn test_bump_kind_serde() {
        assert_eq!(serde_json::to_string(&BumpKind::Major).unwrap(), "\"major\"");
        assert_eq!(serde_json::to_string(&BumpKind::Minor).unwrap(), "\"minor\"");
        assert_eq!(serde_json::to_string(&BumpKind::Patch).unwrap(), "\"patch\"");
    }
}
