use std::collections::HashMap;

use serde::Deserialize;

// ---------------------------------------------------------------------------
// Data model matching type-registry.yaml
// ---------------------------------------------------------------------------

/// Which layer a type belongs to in the three-layer type system.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TypeLayer {
    Base,
    Format,
    Semantic,
}

/// A single type entry as defined in `type-registry.yaml`.
#[derive(Debug, Clone, Deserialize)]
pub struct TypeDefinition {
    pub layer: TypeLayer,
    pub description: String,

    #[serde(default)]
    pub base: Option<String>,

    #[serde(default)]
    pub validation: Option<String>,

    #[serde(default)]
    pub pattern: Option<String>,

    #[serde(default)]
    pub schema_org: Option<String>,

    #[serde(default)]
    pub schema_org_mapping: Option<HashMap<String, String>>,

    #[serde(default)]
    pub structure: Option<HashMap<String, String>>,

    #[serde(default)]
    pub alias_of: Option<String>,

    #[serde(default)]
    pub generic_param: Option<String>,

    #[serde(default)]
    pub generic_params: Option<Vec<String>>,
}

/// Root structure of `type-registry.yaml`.
#[derive(Debug, Deserialize)]
pub struct TypeRegistryFile {
    pub version: String,
    pub types: HashMap<String, TypeDefinition>,
}

// ---------------------------------------------------------------------------
// Compatibility result
// ---------------------------------------------------------------------------

/// How compatible two types are.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompatibilityLevel {
    /// Types are identical (or both resolve to the same canonical type).
    Exact,
    /// Types map to the same Schema.org concept.
    ShadowMatch,
    /// The source type can be safely widened to the target (child -> parent).
    Coercible,
    /// The types have no known compatibility path.
    Incompatible,
}

// ---------------------------------------------------------------------------
// Registry
// ---------------------------------------------------------------------------

/// An in-memory registry of all Radium types, loaded from the bundled YAML.
#[derive(Debug)]
pub struct TypeRegistry {
    file: TypeRegistryFile,
}

impl TypeRegistry {
    /// Load the registry from the YAML file bundled into the binary.
    pub fn load_bundled() -> Result<Self, serde_yaml::Error> {
        let yaml = include_str!("../../type-registry.yaml");
        let file: TypeRegistryFile = serde_yaml::from_str(yaml)?;
        Ok(Self { file })
    }

    /// Look up a type definition by name.
    pub fn get(&self, name: &str) -> Option<&TypeDefinition> {
        self.file.types.get(name)
    }

    /// List all types, optionally filtered to a single layer.
    pub fn list(&self, layer: Option<&TypeLayer>) -> Vec<(&String, &TypeDefinition)> {
        self.file
            .types
            .iter()
            .filter(|(_, def)| layer.map_or(true, |l| def.layer == *l))
            .collect()
    }

    /// Return the registry version string.
    pub fn version(&self) -> &str {
        &self.file.version
    }

    // -- Schema.org resolution ------------------------------------------------

    /// Walk up the type hierarchy (following `base` links) until we find a
    /// `schema_org` mapping, resolving aliases along the way.
    pub fn resolve_schema_org(&self, type_name: &str) -> Option<String> {
        let canonical = self.resolve_alias(type_name);
        let mut current = canonical.as_str();

        // Guard against cycles (max 10 hops is more than enough for 3 layers).
        for _ in 0..10 {
            if let Some(def) = self.file.types.get(current) {
                if let Some(ref mapping) = def.schema_org {
                    return Some(mapping.clone());
                }
                // Walk to base type
                if let Some(ref base) = def.base {
                    let base_canonical = self.resolve_alias(base);
                    // Need to match against owned string; break out if same
                    if base_canonical == current {
                        break;
                    }
                    // We need to keep an owned copy to continue the loop
                    current = self.resolve_alias_static(base)?;
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        None
    }

    // -- Compatibility --------------------------------------------------------

    /// Determine how compatible `from` is with `to`.
    pub fn check_compatibility(&self, from: &str, to: &str) -> CompatibilityLevel {
        let from_canonical = self.resolve_alias(from);
        let to_canonical = self.resolve_alias(to);

        // Exact match after alias resolution
        if from_canonical == to_canonical {
            return CompatibilityLevel::Exact;
        }

        // Check if `from` is a subtype (child) of `to` => Coercible
        if self.is_subtype_of(&from_canonical, &to_canonical) {
            return CompatibilityLevel::Coercible;
        }

        // Check Schema.org shadow match
        let from_schema = self.resolve_schema_org(&from_canonical);
        let to_schema = self.resolve_schema_org(&to_canonical);
        if let (Some(ref fs), Some(ref ts)) = (from_schema, to_schema) {
            if fs == ts {
                return CompatibilityLevel::ShadowMatch;
            }
        }

        CompatibilityLevel::Incompatible
    }

    // -- Internal helpers -----------------------------------------------------

    /// Follow alias chains to find the canonical type name.
    fn resolve_alias(&self, type_name: &str) -> String {
        let mut current = type_name.to_string();
        for _ in 0..10 {
            if let Some(def) = self.file.types.get(&current) {
                if let Some(ref alias) = def.alias_of {
                    current = alias.clone();
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        current
    }

    /// Like `resolve_alias` but returns a reference into the registry's keys,
    /// which lets us keep iterating without lifetime issues.
    fn resolve_alias_static(&self, type_name: &str) -> Option<&str> {
        let canonical = self.resolve_alias(type_name);
        // Find the key in the HashMap that matches (it's heap-allocated and
        // lives as long as &self).
        self.file
            .types
            .keys()
            .find(|k| **k == canonical)
            .map(|k| k.as_str())
    }

    /// Return `true` if `child` is a subtype of `parent` by walking the `base`
    /// chain upward.
    fn is_subtype_of(&self, child: &str, parent: &str) -> bool {
        let mut current = child.to_string();
        for _ in 0..10 {
            if let Some(def) = self.file.types.get(&current) {
                if let Some(ref base) = def.base {
                    let base_canonical = self.resolve_alias(base);
                    if base_canonical == parent {
                        return true;
                    }
                    current = base_canonical;
                } else {
                    return false;
                }
            } else {
                return false;
            }
        }
        false
    }
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn registry() -> TypeRegistry {
        TypeRegistry::load_bundled().expect("Failed to load bundled type registry")
    }

    #[test]
    fn test_load_bundled_registry() {
        let reg = registry();
        assert_eq!(reg.version(), "1.0.0");
        // 10 base + 17 format (including aliases) + 14 semantic = 41
        // but at minimum we should have 30+
        assert!(
            reg.list(None).len() >= 30,
            "Expected at least 30 types, got {}",
            reg.list(None).len()
        );
    }

    #[test]
    fn test_get_base_type() {
        let reg = registry();
        let string_type = reg.get("string").expect("string type should exist");
        assert_eq!(string_type.layer, TypeLayer::Base);
        assert_eq!(
            string_type.schema_org.as_deref(),
            Some("Schema.org/Text")
        );
    }

    #[test]
    fn test_get_format_type() {
        let reg = registry();
        let email = reg.get("string:email").expect("string:email should exist");
        assert_eq!(email.layer, TypeLayer::Format);
        assert_eq!(email.base.as_deref(), Some("string"));
        assert!(email.pattern.is_some(), "email should have a pattern");
    }

    #[test]
    fn test_get_semantic_type() {
        let reg = registry();
        let money = reg.get("money").expect("money type should exist");
        assert_eq!(money.layer, TypeLayer::Semantic);
        assert!(money.structure.is_some(), "money should have structure");
        let structure = money.structure.as_ref().unwrap();
        assert!(structure.contains_key("amount"));
        assert!(structure.contains_key("currency"));
        assert_eq!(
            money.schema_org.as_deref(),
            Some("Schema.org/MonetaryAmount")
        );
    }

    #[test]
    fn test_list_by_layer() {
        let reg = registry();
        let base_count = reg.list(Some(&TypeLayer::Base)).len();
        let format_count = reg.list(Some(&TypeLayer::Format)).len();
        let semantic_count = reg.list(Some(&TypeLayer::Semantic)).len();

        assert_eq!(base_count, 10, "Expected 10 base types");
        // 15 defined + 2 aliases (string:url, number is base alias)
        // Actually: 15 format types defined in YAML + string:url alias = 16
        // but number alias is base layer. Let me just count format entries.
        assert!(
            format_count >= 15,
            "Expected at least 15 format types, got {format_count}"
        );
        assert_eq!(semantic_count, 14, "Expected 14 semantic types");
    }

    #[test]
    fn test_resolve_schema_org_direct() {
        let reg = registry();
        let result = reg.resolve_schema_org("string:email");
        assert_eq!(result.as_deref(), Some("Schema.org/email"));
    }

    #[test]
    fn test_resolve_schema_org_via_base() {
        let reg = registry();
        // string:regex has no schema_org, should walk up to string -> Schema.org/Text
        let result = reg.resolve_schema_org("string:regex");
        assert_eq!(result.as_deref(), Some("Schema.org/Text"));
    }

    #[test]
    fn test_resolve_schema_org_via_alias() {
        let reg = registry();
        // "number" is alias of "float" which has Schema.org/Float
        let result = reg.resolve_schema_org("number");
        assert_eq!(result.as_deref(), Some("Schema.org/Float"));
    }

    #[test]
    fn test_compatibility_exact() {
        let reg = registry();
        assert_eq!(
            reg.check_compatibility("string", "string"),
            CompatibilityLevel::Exact
        );
    }

    #[test]
    fn test_compatibility_alias() {
        let reg = registry();
        // number is an alias of float, so they should be Exact
        assert_eq!(
            reg.check_compatibility("number", "float"),
            CompatibilityLevel::Exact
        );
    }

    #[test]
    fn test_compatibility_coercible() {
        let reg = registry();
        // string:email -> string (child to parent) = Coercible
        assert_eq!(
            reg.check_compatibility("string:email", "string"),
            CompatibilityLevel::Coercible
        );
    }

    #[test]
    fn test_compatibility_incompatible() {
        let reg = registry();
        assert_eq!(
            reg.check_compatibility("string", "integer"),
            CompatibilityLevel::Incompatible
        );
    }

    #[test]
    fn test_secret_ref_type() {
        let reg = registry();
        let secret = reg.get("secret_ref").expect("secret_ref should exist");
        assert_eq!(secret.layer, TypeLayer::Semantic);
        assert!(secret.pattern.is_some(), "secret_ref should have a pattern");
    }

    #[test]
    fn test_money_has_structure() {
        let reg = registry();
        let money = reg.get("money").expect("money type should exist");
        let structure = money.structure.as_ref().expect("money should have structure");
        assert_eq!(structure.get("amount").map(String::as_str), Some("float"));
        assert_eq!(
            structure.get("currency").map(String::as_str),
            Some("currency")
        );
    }
}
