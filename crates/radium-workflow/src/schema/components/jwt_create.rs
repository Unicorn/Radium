//! JWT Create component schema
//!
//! The JWT Create component signs and generates JSON Web Tokens (JWTs) from a
//! provided set of claims and signing configuration. This is a pure (stateless,
//! side-effect-free) component -- it does NOT embed ComponentBehaviors.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use validator::Validate;

/// The signing algorithm to use when creating the JWT.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum JwtAlgorithm {
    /// HMAC using SHA-256 (default).
    #[default]
    Hs256,
    /// HMAC using SHA-384.
    Hs384,
    /// HMAC using SHA-512.
    Hs512,
    /// RSASSA-PKCS1-v1_5 using SHA-256.
    Rs256,
    /// RSASSA-PKCS1-v1_5 using SHA-384.
    Rs384,
    /// RSASSA-PKCS1-v1_5 using SHA-512.
    Rs512,
    /// ECDSA using P-256 and SHA-256.
    Es256,
    /// ECDSA using P-384 and SHA-384.
    Es384,
}

/// JWT Create component input.
///
/// Pure tier -- no retry, rate limit, or other I/O behaviors.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "snake_case")]
pub struct JwtCreateInput {
    /// The signing algorithm to use.
    #[serde(default)]
    pub algorithm: JwtAlgorithm,

    /// Secret reference for the HMAC signing key (used with HS* algorithms).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secret_ref: Option<String>,

    /// Secret reference for the RSA or EC private key (used with RS*/ES* algorithms).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub private_key_ref: Option<String>,

    /// Arbitrary claims to embed in the JWT payload.
    #[serde(default)]
    pub claims: HashMap<String, serde_json::Value>,

    /// The `iss` (issuer) claim.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issuer: Option<String>,

    /// The `sub` (subject) claim.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,

    /// The `aud` (audience) claim.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audience: Option<String>,

    /// Number of seconds from now until the token expires (`exp` claim).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_in_seconds: Option<u64>,

    /// Unix timestamp for the `nbf` (not-before) claim.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub not_before: Option<u64>,

    /// Additional key/value pairs to include in the JWT header.
    #[serde(default)]
    pub additional_headers: HashMap<String, String>,
}

impl Default for JwtCreateInput {
    fn default() -> Self {
        Self {
            algorithm: JwtAlgorithm::default(),
            secret_ref: None,
            private_key_ref: None,
            claims: HashMap::new(),
            issuer: None,
            subject: None,
            audience: None,
            expires_in_seconds: None,
            not_before: None,
            additional_headers: HashMap::new(),
        }
    }
}

/// JWT Create component output.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct JwtCreateOutput {
    /// The signed JWT string.
    pub token: String,

    /// The decoded JWT header as a JSON value.
    pub header: serde_json::Value,

    /// The decoded JWT claims payload as a JSON value.
    pub claims: serde_json::Value,
}

impl Default for JwtCreateOutput {
    fn default() -> Self {
        Self {
            token: String::new(),
            header: serde_json::Value::Object(serde_json::Map::new()),
            claims: serde_json::Value::Object(serde_json::Map::new()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_input_with_defaults() {
        let input = JwtCreateInput::default();
        assert_eq!(input.algorithm, JwtAlgorithm::Hs256);
        assert!(input.secret_ref.is_none());
        assert!(input.private_key_ref.is_none());
        assert!(input.claims.is_empty());
        assert!(input.issuer.is_none());
        assert!(input.subject.is_none());
        assert!(input.audience.is_none());
        assert!(input.expires_in_seconds.is_none());
        assert!(input.not_before.is_none());
        assert!(input.additional_headers.is_empty());
    }

    #[test]
    fn test_full_config_deserialization() {
        let yaml = r#"
algorithm: rs256
private_key_ref: "secrets/my-rsa-key"
claims:
  role: "admin"
  tenant_id: "acme-corp"
issuer: "https://auth.example.com"
subject: "user-42"
audience: "https://api.example.com"
expires_in_seconds: 3600
not_before: 1700000000
additional_headers:
  kid: "key-2024-01"
"#;
        let input: JwtCreateInput = serde_yaml::from_str(yaml).expect("deserialize");

        assert_eq!(input.algorithm, JwtAlgorithm::Rs256);
        assert!(input.secret_ref.is_none());
        assert_eq!(
            input.private_key_ref.as_deref(),
            Some("secrets/my-rsa-key")
        );
        assert_eq!(input.claims.get("role"), Some(&json!("admin")));
        assert_eq!(input.claims.get("tenant_id"), Some(&json!("acme-corp")));
        assert_eq!(
            input.issuer.as_deref(),
            Some("https://auth.example.com")
        );
        assert_eq!(input.subject.as_deref(), Some("user-42"));
        assert_eq!(
            input.audience.as_deref(),
            Some("https://api.example.com")
        );
        assert_eq!(input.expires_in_seconds, Some(3600));
        assert_eq!(input.not_before, Some(1_700_000_000));
        assert_eq!(
            input.additional_headers.get("kid").map(String::as_str),
            Some("key-2024-01")
        );
    }

    #[test]
    fn test_output_serialize_deserialize() {
        let output = JwtCreateOutput {
            token: "eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiJ1c2VyLTEifQ.signature".to_string(),
            header: json!({"alg": "HS256", "typ": "JWT"}),
            claims: json!({"sub": "user-1", "iat": 1700000000}),
        };

        let yaml = serde_yaml::to_string(&output).expect("serialize");
        let restored: JwtCreateOutput = serde_yaml::from_str(&yaml).expect("deserialize");

        assert_eq!(restored.token, output.token);
        assert_eq!(restored.header["alg"], "HS256");
        assert_eq!(restored.header["typ"], "JWT");
        assert_eq!(restored.claims["sub"], "user-1");
        assert_eq!(restored.claims["iat"], 1_700_000_000_i64);

        // Default output has empty token and empty objects for header/claims.
        let default_out = JwtCreateOutput::default();
        assert_eq!(default_out.token, "");
        assert!(default_out.header.is_object());
        assert!(default_out.claims.is_object());
    }

    #[test]
    fn test_algorithm_default() {
        // Verify the derived Default for JwtAlgorithm resolves to Hs256.
        let algo = JwtAlgorithm::default();
        assert_eq!(algo, JwtAlgorithm::Hs256);

        // Verify round-trip serialization for every variant.
        let variants = [
            (JwtAlgorithm::Hs256, "hs256"),
            (JwtAlgorithm::Hs384, "hs384"),
            (JwtAlgorithm::Hs512, "hs512"),
            (JwtAlgorithm::Rs256, "rs256"),
            (JwtAlgorithm::Rs384, "rs384"),
            (JwtAlgorithm::Rs512, "rs512"),
            (JwtAlgorithm::Es256, "es256"),
            (JwtAlgorithm::Es384, "es384"),
        ];

        for (variant, expected_str) in variants {
            let serialized = serde_json::to_string(&variant).expect("serialize");
            // serde_json wraps strings in quotes.
            assert_eq!(serialized, format!("\"{}\"", expected_str));

            let deserialized: JwtAlgorithm =
                serde_json::from_str(&serialized).expect("deserialize");
            assert_eq!(deserialized, variant);
        }
    }
}
