//! Schema change detection and automated version bump calculation.
//!
//! Compares two component schema versions and determines:
//! - What fields were added, removed, or modified
//! - Whether the change is breaking (major), additive (minor), or cosmetic (patch)
//! - The recommended semver bump

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::versioning::BumpKind;

/// A single detected change between two schema versions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaChange {
    /// The JSON path to the changed field (e.g., "input_schema.fields.timeout_ms")
    pub path: String,
    /// What kind of change this is
    pub kind: ChangeKind,
    /// Human-readable description
    pub description: String,
}

/// The kind of a detected schema change.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChangeKind {
    /// A new field was added (minor if optional, major if required without default)
    FieldAdded,
    /// An existing field was removed (always major — breaking)
    FieldRemoved,
    /// A field's type changed (always major — breaking)
    TypeChanged,
    /// A field's default value changed (patch)
    DefaultChanged,
    /// A field became required that was previously optional (major — breaking)
    BecameRequired,
    /// A field became optional that was previously required (minor — relaxation)
    BecameOptional,
    /// Description or other metadata changed (patch)
    MetadataChanged,
    /// An enum variant was added (minor)
    VariantAdded,
    /// An enum variant was removed (major — breaking)
    VariantRemoved,
}

/// Result of comparing two schema versions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeReport {
    /// All individual changes detected between the two schemas.
    pub changes: Vec<SchemaChange>,
    /// The minimum semver bump required to publish the new schema version.
    pub recommended_bump: BumpKind,
    /// Whether any of the detected changes are breaking.
    pub breaking: bool,
    /// A short human-readable summary of the most significant changes.
    pub summary: String,
}

impl ChangeKind {
    /// What semver bump level does this kind of change require?
    #[must_use]
    pub fn bump_level(self) -> BumpKind {
        match self {
            ChangeKind::FieldRemoved
            | ChangeKind::TypeChanged
            | ChangeKind::BecameRequired
            | ChangeKind::VariantRemoved => BumpKind::Major,

            ChangeKind::FieldAdded
            | ChangeKind::BecameOptional
            | ChangeKind::VariantAdded => BumpKind::Minor,

            ChangeKind::DefaultChanged | ChangeKind::MetadataChanged => BumpKind::Patch,
        }
    }

    /// Is this a breaking change?
    #[must_use]
    pub fn is_breaking(self) -> bool {
        matches!(self.bump_level(), BumpKind::Major)
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Aggregate the highest bump level seen so far.
fn max_bump(a: BumpKind, b: BumpKind) -> BumpKind {
    match (a, b) {
        (BumpKind::Major, _) | (_, BumpKind::Major) => BumpKind::Major,
        (BumpKind::Minor, _) | (_, BumpKind::Minor) => BumpKind::Minor,
        _ => BumpKind::Patch,
    }
}

/// Compare a single field array (e.g. `input_schema.fields`) between old and new schemas.
///
/// `section` is used to construct readable paths such as `"input_schema.fields.timeout_ms"`.
fn compare_field_arrays(
    old_fields: &[Value],
    new_fields: &[Value],
    section: &str,
    changes: &mut Vec<SchemaChange>,
) {
    // Index both arrays by field name for O(n) lookups.
    let old_by_name: std::collections::HashMap<&str, &Value> = old_fields
        .iter()
        .filter_map(|f| f.get("name").and_then(Value::as_str).map(|n| (n, f)))
        .collect();

    let new_by_name: std::collections::HashMap<&str, &Value> = new_fields
        .iter()
        .filter_map(|f| f.get("name").and_then(Value::as_str).map(|n| (n, f)))
        .collect();

    // Fields present in old but missing in new → removed (major).
    for name in old_by_name.keys() {
        if !new_by_name.contains_key(name) {
            changes.push(SchemaChange {
                path: format!("{section}.{name}"),
                kind: ChangeKind::FieldRemoved,
                description: format!("Field '{name}' was removed from {section}"),
            });
        }
    }

    // Fields present in new — compare against old.
    for (name, new_field) in &new_by_name {
        let path = format!("{section}.{name}");
        match old_by_name.get(name) {
            None => {
                // Brand-new field.  Classify by required + default presence.
                let required = new_field
                    .get("required")
                    .and_then(Value::as_bool)
                    .unwrap_or(false);
                let has_default = new_field.get("default").is_some();

                // A required field with no default forces all existing consumers to
                // provide it → breaking (major).  We record FieldAdded for the field
                // itself and emit a synthetic BecameRequired change so the aggregated
                // bump reaches Major.  Optional or defaulted additions are Minor.
                let breaking_note = if required && !has_default {
                    " (required, no default — breaking)"
                } else {
                    " (optional or has default)"
                };

                changes.push(SchemaChange {
                    path: path.clone(),
                    kind: ChangeKind::FieldAdded,
                    description: format!(
                        "Field '{name}' was added to {section}{breaking_note}"
                    ),
                });

                // For a required-no-default addition, also push a BecameRequired marker
                // so the aggregated bump reaches Major.
                if required && !has_default {
                    changes.push(SchemaChange {
                        path: path.clone(),
                        kind: ChangeKind::BecameRequired,
                        description: format!(
                            "New field '{name}' in {section} is required without a default value"
                        ),
                    });
                }
            }
            Some(old_field) => {
                // Field exists in both — compare individual attributes.
                compare_existing_field(old_field, new_field, name, &path, section, changes);
            }
        }
    }
}

/// Compare two versions of the same field and emit granular changes.
fn compare_existing_field(
    old: &Value,
    new: &Value,
    name: &str,
    path: &str,
    section: &str,
    changes: &mut Vec<SchemaChange>,
) {
    // Type change (use either rustType or typescriptType as discriminant).
    let old_type = old
        .get("rustType")
        .or_else(|| old.get("type"))
        .and_then(Value::as_str);
    let new_type = new
        .get("rustType")
        .or_else(|| new.get("type"))
        .and_then(Value::as_str);

    if let (Some(ot), Some(nt)) = (old_type, new_type) {
        if ot != nt {
            changes.push(SchemaChange {
                path: path.to_string(),
                kind: ChangeKind::TypeChanged,
                description: format!(
                    "Field '{name}' in {section} changed type from '{ot}' to '{nt}'"
                ),
            });
        }
    }

    // Required / optional transition.
    let old_required = old.get("required").and_then(Value::as_bool).unwrap_or(false);
    let new_required = new.get("required").and_then(Value::as_bool).unwrap_or(false);

    match (old_required, new_required) {
        (false, true) => {
            changes.push(SchemaChange {
                path: path.to_string(),
                kind: ChangeKind::BecameRequired,
                description: format!("Field '{name}' in {section} became required"),
            });
        }
        (true, false) => {
            changes.push(SchemaChange {
                path: path.to_string(),
                kind: ChangeKind::BecameOptional,
                description: format!("Field '{name}' in {section} became optional"),
            });
        }
        _ => {}
    }

    // Default value change.
    let old_default = old.get("default");
    let new_default = new.get("default");

    if old_default != new_default {
        changes.push(SchemaChange {
            path: path.to_string(),
            kind: ChangeKind::DefaultChanged,
            description: format!("Field '{name}' in {section} default value changed"),
        });
    }

    // Description / metadata change (description key).
    let old_desc = old.get("description").and_then(Value::as_str).unwrap_or("");
    let new_desc = new.get("description").and_then(Value::as_str).unwrap_or("");

    if old_desc != new_desc {
        changes.push(SchemaChange {
            path: path.to_string(),
            kind: ChangeKind::MetadataChanged,
            description: format!("Field '{name}' in {section} description changed"),
        });
    }

    // Enum variants comparison (variants key, array of strings).
    let old_variants: Option<Vec<&str>> = old
        .get("variants")
        .and_then(Value::as_array)
        .map(|arr| arr.iter().filter_map(Value::as_str).collect());
    let new_variants: Option<Vec<&str>> = new
        .get("variants")
        .and_then(Value::as_array)
        .map(|arr| arr.iter().filter_map(Value::as_str).collect());

    if let (Some(ov), Some(nv)) = (old_variants, new_variants) {
        let old_set: std::collections::HashSet<&str> = ov.into_iter().collect();
        let new_set: std::collections::HashSet<&str> = nv.into_iter().collect();

        for variant in old_set.difference(&new_set) {
            changes.push(SchemaChange {
                path: format!("{path}.variants.{variant}"),
                kind: ChangeKind::VariantRemoved,
                description: format!(
                    "Enum variant '{variant}' was removed from field '{name}' in {section}"
                ),
            });
        }
        for variant in new_set.difference(&old_set) {
            changes.push(SchemaChange {
                path: format!("{path}.variants.{variant}"),
                kind: ChangeKind::VariantAdded,
                description: format!(
                    "Enum variant '{variant}' was added to field '{name}' in {section}"
                ),
            });
        }
    }
}

/// Compare top-level metadata scalars between two schemas.
fn compare_top_level_metadata(
    old: &Value,
    new: &Value,
    changes: &mut Vec<SchemaChange>,
) {
    for key in &["description", "category"] {
        let old_val = old.get(key).and_then(Value::as_str).unwrap_or("");
        let new_val = new.get(key).and_then(Value::as_str).unwrap_or("");
        if old_val != new_val {
            changes.push(SchemaChange {
                path: (*key).to_string(),
                kind: ChangeKind::MetadataChanged,
                description: format!("Top-level '{key}' changed"),
            });
        }
    }
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Compare two component schemas (as [`serde_json::Value`]) and produce a [`ChangeReport`].
///
/// The schemas should be JSON objects.  The function compares:
/// - Top-level metadata fields (`description`, `category`)
/// - `input_schema.fields` array
/// - `output_schema.fields` array
/// - `config_fields` array
///
/// Returns a [`ChangeReport`] describing all detected differences and the
/// minimum semver bump required to publish the new version.
pub fn compare_schemas(old: &Value, new: &Value) -> ChangeReport {
    let mut changes: Vec<SchemaChange> = Vec::new();

    // --- Top-level metadata ---
    compare_top_level_metadata(old, new, &mut changes);

    // Helper to extract a field array from a nested schema key.
    let empty: Vec<Value> = Vec::new();

    // --- input_schema.fields ---
    let old_input: &[Value] = old
        .get("input_schema")
        .and_then(|s| s.get("fields"))
        .and_then(Value::as_array)
        .map_or(empty.as_slice(), Vec::as_slice);
    let new_input: &[Value] = new
        .get("input_schema")
        .and_then(|s| s.get("fields"))
        .and_then(Value::as_array)
        .map_or(empty.as_slice(), Vec::as_slice);

    compare_field_arrays(old_input, new_input, "input_schema.fields", &mut changes);

    // --- output_schema.fields ---
    let old_output: &[Value] = old
        .get("output_schema")
        .and_then(|s| s.get("fields"))
        .and_then(Value::as_array)
        .map_or(empty.as_slice(), Vec::as_slice);
    let new_output: &[Value] = new
        .get("output_schema")
        .and_then(|s| s.get("fields"))
        .and_then(Value::as_array)
        .map_or(empty.as_slice(), Vec::as_slice);

    compare_field_arrays(old_output, new_output, "output_schema.fields", &mut changes);

    // --- config_fields (flat array, not nested under a sub-object) ---
    let old_config: &[Value] = old
        .get("config_fields")
        .and_then(Value::as_array)
        .map_or(empty.as_slice(), Vec::as_slice);
    let new_config: &[Value] = new
        .get("config_fields")
        .and_then(Value::as_array)
        .map_or(empty.as_slice(), Vec::as_slice);

    compare_field_arrays(old_config, new_config, "config_fields", &mut changes);

    // --- Aggregate ---
    let breaking = changes.iter().any(|c| c.kind.is_breaking());
    let recommended_bump = if changes.is_empty() {
        BumpKind::Patch
    } else {
        changes
            .iter()
            .fold(BumpKind::Patch, |acc, c| max_bump(acc, c.kind.bump_level()))
    };

    let summary = build_summary(&changes, recommended_bump, breaking);

    ChangeReport {
        changes,
        recommended_bump,
        breaking,
        summary,
    }
}

/// Build a concise summary string from the aggregated changes.
fn build_summary(changes: &[SchemaChange], bump: BumpKind, breaking: bool) -> String {
    if changes.is_empty() {
        return "No schema changes detected.".to_string();
    }

    let major_count = changes
        .iter()
        .filter(|c| c.kind.is_breaking())
        .count();
    let minor_count = changes
        .iter()
        .filter(|c| matches!(c.kind.bump_level(), BumpKind::Minor) && !c.kind.is_breaking())
        .count();
    let patch_count = changes
        .iter()
        .filter(|c| matches!(c.kind.bump_level(), BumpKind::Patch))
        .count();

    let bump_label = match bump {
        BumpKind::Major => "major",
        BumpKind::Minor => "minor",
        BumpKind::Patch => "patch",
    };

    let breaking_note = if breaking { " (breaking)" } else { "" };

    format!(
        "{} change(s) detected: {major_count} breaking, {minor_count} additive, \
         {patch_count} cosmetic. Recommended bump: {bump_label}{breaking_note}.",
        changes.len(),
    )
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // Helper: build a minimal schema object.
    fn schema(
        description: &str,
        input_fields: serde_json::Value,
        output_fields: serde_json::Value,
    ) -> Value {
        json!({
            "description": description,
            "input_schema": { "fields": input_fields },
            "output_schema": { "fields": output_fields },
        })
    }

    fn field(
        name: &str,
        rust_type: &str,
        required: bool,
        default: Option<&str>,
    ) -> Value {
        let mut f = json!({
            "name": name,
            "rustType": rust_type,
            "required": required,
            "description": "",
        });
        if let Some(d) = default {
            f["default"] = json!(d);
        }
        f
    }

    // -----------------------------------------------------------------------
    // No changes
    // -----------------------------------------------------------------------

    #[test]
    fn test_no_changes_empty_schemas() {
        let s = json!({});
        let report = compare_schemas(&s, &s);
        assert!(report.changes.is_empty());
        assert_eq!(report.recommended_bump, BumpKind::Patch);
        assert!(!report.breaking);
    }

    #[test]
    fn test_no_changes_identical_schema() {
        let s = schema(
            "My component",
            json!([field("timeout_ms", "u64", false, Some("5000"))]),
            json!([field("success", "bool", true, None)]),
        );
        let report = compare_schemas(&s, &s);
        assert!(report.changes.is_empty());
        assert_eq!(report.recommended_bump, BumpKind::Patch);
        assert!(!report.breaking);
    }

    // -----------------------------------------------------------------------
    // Description / metadata only
    // -----------------------------------------------------------------------

    #[test]
    fn test_description_change_is_patch() {
        let old = schema("Old description", json!([]), json!([]));
        let new = schema("New description", json!([]), json!([]));
        let report = compare_schemas(&old, &new);

        assert_eq!(report.changes.len(), 1);
        assert_eq!(report.changes[0].kind, ChangeKind::MetadataChanged);
        assert_eq!(report.recommended_bump, BumpKind::Patch);
        assert!(!report.breaking);
    }

    // -----------------------------------------------------------------------
    // Field added — optional
    // -----------------------------------------------------------------------

    #[test]
    fn test_new_optional_field_is_minor() {
        let old = schema("desc", json!([]), json!([]));
        let new = schema(
            "desc",
            json!([field("new_field", "String", false, None)]),
            json!([]),
        );
        let report = compare_schemas(&old, &new);

        let added: Vec<_> = report
            .changes
            .iter()
            .filter(|c| c.kind == ChangeKind::FieldAdded)
            .collect();
        assert!(!added.is_empty(), "expected a FieldAdded change");
        assert_eq!(report.recommended_bump, BumpKind::Minor);
        assert!(!report.breaking);
    }

    #[test]
    fn test_new_optional_field_with_default_is_minor() {
        let old = schema("desc", json!([]), json!([]));
        let new = schema(
            "desc",
            json!([field("new_field", "String", false, Some("hello"))]),
            json!([]),
        );
        let report = compare_schemas(&old, &new);
        assert_eq!(report.recommended_bump, BumpKind::Minor);
        assert!(!report.breaking);
    }

    // -----------------------------------------------------------------------
    // Field added — required, no default → major / breaking
    // -----------------------------------------------------------------------

    #[test]
    fn test_new_required_field_no_default_is_major() {
        let old = schema("desc", json!([]), json!([]));
        let new = schema(
            "desc",
            json!([field("required_field", "String", true, None)]),
            json!([]),
        );
        let report = compare_schemas(&old, &new);

        assert!(
            report.changes.iter().any(|c| c.kind == ChangeKind::BecameRequired),
            "expected a BecameRequired synthetic change"
        );
        assert_eq!(report.recommended_bump, BumpKind::Major);
        assert!(report.breaking);
    }

    // -----------------------------------------------------------------------
    // Field removed → major / breaking
    // -----------------------------------------------------------------------

    #[test]
    fn test_field_removed_is_major() {
        let old = schema(
            "desc",
            json!([field("old_field", "String", false, None)]),
            json!([]),
        );
        let new = schema("desc", json!([]), json!([]));
        let report = compare_schemas(&old, &new);

        assert!(
            report.changes.iter().any(|c| c.kind == ChangeKind::FieldRemoved)
        );
        assert_eq!(report.recommended_bump, BumpKind::Major);
        assert!(report.breaking);
    }

    // -----------------------------------------------------------------------
    // Field type changed → major / breaking
    // -----------------------------------------------------------------------

    #[test]
    fn test_type_changed_is_major() {
        let old = schema(
            "desc",
            json!([field("count", "u32", true, None)]),
            json!([]),
        );
        let new = schema(
            "desc",
            json!([field("count", "String", true, None)]),
            json!([]),
        );
        let report = compare_schemas(&old, &new);

        assert!(
            report.changes.iter().any(|c| c.kind == ChangeKind::TypeChanged)
        );
        assert_eq!(report.recommended_bump, BumpKind::Major);
        assert!(report.breaking);
    }

    // -----------------------------------------------------------------------
    // Field became required → major / breaking
    // -----------------------------------------------------------------------

    #[test]
    fn test_became_required_is_major() {
        let old = schema(
            "desc",
            json!([field("timeout_ms", "u64", false, None)]),
            json!([]),
        );
        let new = schema(
            "desc",
            json!([field("timeout_ms", "u64", true, None)]),
            json!([]),
        );
        let report = compare_schemas(&old, &new);

        assert!(
            report.changes.iter().any(|c| c.kind == ChangeKind::BecameRequired)
        );
        assert_eq!(report.recommended_bump, BumpKind::Major);
        assert!(report.breaking);
    }

    // -----------------------------------------------------------------------
    // Field became optional → minor
    // -----------------------------------------------------------------------

    #[test]
    fn test_became_optional_is_minor() {
        let old = schema(
            "desc",
            json!([field("timeout_ms", "u64", true, None)]),
            json!([]),
        );
        let new = schema(
            "desc",
            json!([field("timeout_ms", "u64", false, None)]),
            json!([]),
        );
        let report = compare_schemas(&old, &new);

        assert!(
            report.changes.iter().any(|c| c.kind == ChangeKind::BecameOptional)
        );
        assert_eq!(report.recommended_bump, BumpKind::Minor);
        assert!(!report.breaking);
    }

    // -----------------------------------------------------------------------
    // Default value changed → patch
    // -----------------------------------------------------------------------

    #[test]
    fn test_default_changed_is_patch() {
        let old = schema(
            "desc",
            json!([field("retries", "u32", false, Some("3"))]),
            json!([]),
        );
        let new = schema(
            "desc",
            json!([field("retries", "u32", false, Some("5"))]),
            json!([]),
        );
        let report = compare_schemas(&old, &new);

        assert!(
            report.changes.iter().any(|c| c.kind == ChangeKind::DefaultChanged)
        );
        assert_eq!(report.recommended_bump, BumpKind::Patch);
        assert!(!report.breaking);
    }

    // -----------------------------------------------------------------------
    // Multiple changes — highest bump wins
    // -----------------------------------------------------------------------

    #[test]
    fn test_multiple_changes_highest_bump_wins() {
        // patch change: description
        // minor change: new optional field
        // major change: field removed
        let old = schema(
            "Old desc",
            json!([
                field("existing", "String", false, None),
                field("to_remove", "String", false, None),
            ]),
            json!([]),
        );
        let new = schema(
            "New desc",
            json!([
                field("existing", "String", false, None),
                field("new_optional", "u64", false, None),
            ]),
            json!([]),
        );
        let report = compare_schemas(&old, &new);

        assert!(report.changes.iter().any(|c| c.kind == ChangeKind::MetadataChanged));
        assert!(report.changes.iter().any(|c| c.kind == ChangeKind::FieldAdded));
        assert!(report.changes.iter().any(|c| c.kind == ChangeKind::FieldRemoved));
        assert_eq!(report.recommended_bump, BumpKind::Major);
        assert!(report.breaking);
    }

    // -----------------------------------------------------------------------
    // Mixed breaking and non-breaking — breaking wins
    // -----------------------------------------------------------------------

    #[test]
    fn test_mixed_breaking_nonbreaking_major_wins() {
        let old = schema(
            "desc",
            json!([
                field("keep", "String", false, Some("default")),
                field("goodbye", "bool", true, None),
            ]),
            json!([]),
        );
        // Remove a field (major) but also add an optional one (minor).
        let new = schema(
            "desc",
            json!([
                field("keep", "String", false, Some("default")),
                field("hello", "u32", false, None),
            ]),
            json!([]),
        );
        let report = compare_schemas(&old, &new);
        assert!(report.breaking);
        assert_eq!(report.recommended_bump, BumpKind::Major);
    }

    // -----------------------------------------------------------------------
    // Output schema and config_fields are also compared
    // -----------------------------------------------------------------------

    #[test]
    fn test_output_schema_field_removed_is_major() {
        let old = schema(
            "desc",
            json!([]),
            json!([field("result", "Value", false, None)]),
        );
        let new = schema("desc", json!([]), json!([]));
        let report = compare_schemas(&old, &new);

        assert!(report.changes.iter().any(|c| {
            c.kind == ChangeKind::FieldRemoved && c.path.contains("output_schema")
        }));
        assert_eq!(report.recommended_bump, BumpKind::Major);
    }

    #[test]
    fn test_config_fields_compared() {
        let old = json!({
            "description": "desc",
            "config_fields": [field("endpoint", "String", true, None)],
        });
        let new = json!({
            "description": "desc",
            "config_fields": [],
        });
        let report = compare_schemas(&old, &new);

        assert!(report.changes.iter().any(|c| {
            c.kind == ChangeKind::FieldRemoved && c.path.contains("config_fields")
        }));
        assert_eq!(report.recommended_bump, BumpKind::Major);
    }

    // -----------------------------------------------------------------------
    // ChangeKind helpers
    // -----------------------------------------------------------------------

    #[test]
    fn test_change_kind_bump_levels() {
        assert_eq!(ChangeKind::FieldRemoved.bump_level(), BumpKind::Major);
        assert_eq!(ChangeKind::TypeChanged.bump_level(), BumpKind::Major);
        assert_eq!(ChangeKind::BecameRequired.bump_level(), BumpKind::Major);
        assert_eq!(ChangeKind::VariantRemoved.bump_level(), BumpKind::Major);

        assert_eq!(ChangeKind::FieldAdded.bump_level(), BumpKind::Minor);
        assert_eq!(ChangeKind::BecameOptional.bump_level(), BumpKind::Minor);
        assert_eq!(ChangeKind::VariantAdded.bump_level(), BumpKind::Minor);

        assert_eq!(ChangeKind::DefaultChanged.bump_level(), BumpKind::Patch);
        assert_eq!(ChangeKind::MetadataChanged.bump_level(), BumpKind::Patch);
    }

    #[test]
    fn test_change_kind_is_breaking() {
        assert!(ChangeKind::FieldRemoved.is_breaking());
        assert!(ChangeKind::TypeChanged.is_breaking());
        assert!(ChangeKind::BecameRequired.is_breaking());
        assert!(ChangeKind::VariantRemoved.is_breaking());

        assert!(!ChangeKind::FieldAdded.is_breaking());
        assert!(!ChangeKind::BecameOptional.is_breaking());
        assert!(!ChangeKind::VariantAdded.is_breaking());
        assert!(!ChangeKind::DefaultChanged.is_breaking());
        assert!(!ChangeKind::MetadataChanged.is_breaking());
    }

    // -----------------------------------------------------------------------
    // Enum variant changes
    // -----------------------------------------------------------------------

    #[test]
    fn test_variant_added_is_minor() {
        let old_field = json!({
            "name": "status",
            "rustType": "StatusEnum",
            "required": true,
            "description": "",
            "variants": ["pending", "running"],
        });
        let new_field = json!({
            "name": "status",
            "rustType": "StatusEnum",
            "required": true,
            "description": "",
            "variants": ["pending", "running", "cancelled"],
        });
        let old = json!({
            "input_schema": { "fields": [old_field] },
        });
        let new = json!({
            "input_schema": { "fields": [new_field] },
        });
        let report = compare_schemas(&old, &new);

        assert!(report.changes.iter().any(|c| c.kind == ChangeKind::VariantAdded));
        assert!(!report.breaking);
        assert_eq!(report.recommended_bump, BumpKind::Minor);
    }

    #[test]
    fn test_variant_removed_is_major() {
        let old_field = json!({
            "name": "status",
            "rustType": "StatusEnum",
            "required": true,
            "description": "",
            "variants": ["pending", "running", "cancelled"],
        });
        let new_field = json!({
            "name": "status",
            "rustType": "StatusEnum",
            "required": true,
            "description": "",
            "variants": ["pending", "running"],
        });
        let old = json!({ "input_schema": { "fields": [old_field] } });
        let new = json!({ "input_schema": { "fields": [new_field] } });
        let report = compare_schemas(&old, &new);

        assert!(report.changes.iter().any(|c| c.kind == ChangeKind::VariantRemoved));
        assert!(report.breaking);
        assert_eq!(report.recommended_bump, BumpKind::Major);
    }

    // -----------------------------------------------------------------------
    // Summary string
    // -----------------------------------------------------------------------

    #[test]
    fn test_summary_no_changes() {
        let s = json!({});
        let report = compare_schemas(&s, &s);
        assert_eq!(report.summary, "No schema changes detected.");
    }

    #[test]
    fn test_summary_with_changes() {
        let old = schema("Old desc", json!([]), json!([]));
        let new = schema("New desc", json!([field("x", "u32", false, None)]), json!([]));
        let report = compare_schemas(&old, &new);
        assert!(report.summary.contains("change(s) detected"));
        assert!(report.summary.contains("minor"));
    }
}
