//! Schema similarity scoring using Jaccard similarity of field names

use serde_json::Value;

/// Calculate the Jaccard similarity between two JSON schemas
/// based on their property/field names.
/// Returns a value between 0.0 (no overlap) and 1.0 (identical fields).
pub fn schema_overlap(schema_a: &Value, schema_b: &Value) -> f64 {
    let fields_a = extract_field_names(schema_a);
    let fields_b = extract_field_names(schema_b);

    if fields_a.is_empty() && fields_b.is_empty() {
        return 0.0;
    }

    let intersection = fields_a.iter().filter(|f| fields_b.contains(f)).count();
    let union = {
        let mut all = fields_a;
        for f in &fields_b {
            if !all.contains(f) {
                all.push(f.clone());
            }
        }
        all.len()
    };

    if union == 0 {
        return 0.0;
    }

    #[allow(clippy::cast_precision_loss)]
    let result = intersection as f64 / union as f64;
    result
}

/// Extract field names from a JSON schema (supports JSON Schema format)
fn extract_field_names(schema: &Value) -> Vec<String> {
    let mut fields = Vec::new();

    // JSON Schema: {"type": "object", "properties": {"field1": {...}, "field2": {...}}}
    if let Some(properties) = schema.get("properties").and_then(Value::as_object) {
        for key in properties.keys() {
            fields.push(key.clone());
        }
    }

    // Simple object: {"field1": "string", "field2": "number"}
    if fields.is_empty() {
        if let Some(obj) = schema.as_object() {
            for key in obj.keys() {
                if key != "type" && key != "required" && key != "description" {
                    fields.push(key.clone());
                }
            }
        }
    }

    fields.sort();
    fields
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_identical_schemas() {
        let a = json!({"properties": {"name": {}, "email": {}, "age": {}}});
        let b = json!({"properties": {"name": {}, "email": {}, "age": {}}});
        assert!((schema_overlap(&a, &b) - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_no_overlap() {
        let a = json!({"properties": {"name": {}, "email": {}}});
        let b = json!({"properties": {"color": {}, "size": {}}});
        assert!((schema_overlap(&a, &b)).abs() < f64::EPSILON);
    }

    #[test]
    fn test_partial_overlap() {
        let a = json!({"properties": {"name": {}, "email": {}, "age": {}}});
        let b = json!({"properties": {"name": {}, "email": {}, "phone": {}}});
        // Intersection: name, email (2). Union: name, email, age, phone (4).
        // Jaccard = 2/4 = 0.5
        assert!((schema_overlap(&a, &b) - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_empty_schemas() {
        let a = json!({});
        let b = json!({});
        assert!((schema_overlap(&a, &b)).abs() < f64::EPSILON);
    }

    #[test]
    fn test_one_empty() {
        let a = json!({"properties": {"name": {}}});
        let b = json!({});
        assert!((schema_overlap(&a, &b)).abs() < f64::EPSILON);
    }

    #[test]
    fn test_extract_from_simple_object() {
        let schema = json!({"field1": "string", "field2": "number"});
        let fields = extract_field_names(&schema);
        assert_eq!(fields, vec!["field1", "field2"]);
    }
}
