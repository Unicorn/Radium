//! Definition lookup for symbols.

use std::path::PathBuf;
use crate::analysis::symbols::Symbol;
use crate::analysis::rust_analyzer::RustAnalyzer;
use crate::analysis::typescript_analyzer::TypeScriptAnalyzer;

/// Find the definition of a symbol in a file.
pub fn find_definition(
    source: &str,
    file_path: PathBuf,
    symbol_name: &str,
    language: &str,
) -> Result<Option<Symbol>, String> {
    match language.to_lowercase().as_str() {
        "rust" => {
            let mut analyzer = RustAnalyzer::new();
            let symbols = analyzer.extract_symbols(source, file_path.clone())?;
            Ok(symbols.into_iter()
                .find(|s| s.name == symbol_name))
        }
        "typescript" | "ts" => {
            let is_tsx = file_path.extension()
                .and_then(|e| e.to_str())
                .map(|e| e == "tsx")
                .unwrap_or(false);
            let mut analyzer = TypeScriptAnalyzer::new();
            let symbols = analyzer.extract_symbols(source, file_path.clone(), is_tsx)?;
            Ok(symbols.into_iter()
                .find(|s| s.name == symbol_name))
        }
        _ => Err(format!("Language '{}' not supported for definition lookup", language))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_rust_definition() {
        let source = "pub fn calculate(x: i32) -> i32 { x * 2 }";
        let result = find_definition(source, PathBuf::from("test.rs"), "calculate", "rust").unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().name, "calculate");
    }

    #[test]
    fn test_find_typescript_definition() {
        let source = "export function calculate(x: number): number { return x * 2; }";
        let result = find_definition(source, PathBuf::from("test.ts"), "calculate", "typescript").unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().name, "calculate");
    }
}
