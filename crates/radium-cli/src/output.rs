use serde::Serialize;

/// Pretty-print a serializable value as JSON to stdout.
#[allow(dead_code)]
pub fn print_json<T: Serialize>(value: &T) {
    match serde_json::to_string_pretty(value) {
        Ok(json) => println!("{json}"),
        Err(e) => print_error(&format!("Failed to serialize output: {e}")),
    }
}

/// Print an error message as JSON to stderr.
pub fn print_error(message: &str) {
    let error = serde_json::json!({
        "error": message
    });
    eprintln!("{}", serde_json::to_string_pretty(&error).unwrap_or_else(|_| {
        format!("{{\"error\": \"{message}\"}}")
    }));
}
