//! TypeScript-specific symbol extraction using tree-sitter.

use std::path::PathBuf;
use tree_sitter::{Node, Parser};
use tree_sitter_typescript::{language_tsx, language_typescript};
use crate::analysis::symbols::{Symbol, SymbolKind, SymbolSearchResult};

/// TypeScript code analyzer for symbol extraction.
pub struct TypeScriptAnalyzer {
    parser: Parser,
    ts_language: tree_sitter::Language,
    tsx_language: tree_sitter::Language,
}

impl TypeScriptAnalyzer {
    /// Create a new TypeScript analyzer.
    pub fn new() -> Self {
        let mut parser = Parser::new();
        let ts_language = language_typescript();
        let tsx_language = language_tsx();
        
        // Default to TypeScript (can switch per file)
        parser.set_language(ts_language).expect("Failed to load TypeScript grammar");

        Self {
            parser,
            ts_language,
            tsx_language,
        }
    }

    /// Extract all symbols from a TypeScript/TSX source file.
    pub fn extract_symbols(&mut self, source: &str, file_path: PathBuf, is_tsx: bool) -> Result<Vec<Symbol>, String> {
        let language = if is_tsx { self.tsx_language } else { self.ts_language };
        self.parser.set_language(language).map_err(|e| format!("Failed to set language: {:?}", e))?;
        
        let tree = self.parser.parse(source, None)
            .ok_or_else(|| "Parse error".to_string())?;

        let mut symbols = Vec::new();
        let root = tree.root_node();
        self.extract_from_node(&root, source, &file_path, &mut symbols);

        Ok(symbols)
    }

    /// Search for symbols matching a query.
    pub fn search_symbols(&mut self, source: &str, file_path: PathBuf, query: &str, is_tsx: bool) -> Result<SymbolSearchResult, String> {
        let all_symbols = self.extract_symbols(source, file_path, is_tsx)?;
        
        let query_lower = query.to_lowercase();
        let matching: Vec<Symbol> = if query == "*" || query.is_empty() {
            all_symbols
        } else {
            all_symbols.into_iter()
                .filter(|s| s.name.to_lowercase().contains(&query_lower))
                .collect()
        };

        Ok(SymbolSearchResult {
            total: matching.len(),
            symbols: matching,
        })
    }

    /// Extract symbols from a tree-sitter node recursively.
    fn extract_from_node(&self, node: &Node, source: &str, file_path: &PathBuf, symbols: &mut Vec<Symbol>) {
        match node.kind() {
            "function_declaration" | "method_definition" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let name = name_node.utf8_text(source.as_bytes()).unwrap_or("").to_string();
                    let symbol = self.create_function_symbol(node, name, source, file_path);
                    symbols.push(symbol);
                }
            }
            "arrow_function" => {
                // Arrow functions might be in variable declarations
                if let Some(parent) = node.parent() {
                    if parent.kind() == "variable_declarator" {
                        if let Some(name_node) = parent.child_by_field_name("name") {
                            let name = name_node.utf8_text(source.as_bytes()).unwrap_or("").to_string();
                            let symbol = self.create_function_symbol(node, name, source, file_path);
                            symbols.push(symbol);
                        }
                    }
                }
            }
            "class_declaration" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let name = name_node.utf8_text(source.as_bytes()).unwrap_or("").to_string();
                    symbols.push(self.create_class_symbol(node, name, source, file_path));
                }
            }
            "interface_declaration" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let name = name_node.utf8_text(source.as_bytes()).unwrap_or("").to_string();
                    symbols.push(self.create_interface_symbol(node, name, source, file_path));
                }
            }
            "type_alias_declaration" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let name = name_node.utf8_text(source.as_bytes()).unwrap_or("").to_string();
                    symbols.push(self.create_type_symbol(node, name, source, file_path));
                }
            }
            "enum_declaration" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let name = name_node.utf8_text(source.as_bytes()).unwrap_or("").to_string();
                    symbols.push(self.create_enum_symbol(node, name, source, file_path));
                }
            }
            _ => {}
        }

        // Recursively process children
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.extract_from_node(&child, source, file_path, symbols);
            }
        }
    }

    fn create_function_symbol(&self, node: &Node, name: String, _source: &str, file_path: &PathBuf) -> Symbol {
        let mut metadata = Vec::new();
        let mut visibility = None;

        // Check for export and async in parent nodes
        let mut current = node.parent();
        while let Some(n) = current {
            if n.kind() == "export_statement" {
                visibility = Some("exported".to_string());
            }
            if n.kind() == "async" {
                metadata.push("async".to_string());
            }
            current = n.parent();
        }
        
        // Check for async keyword in the node itself
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                if child.kind() == "async" {
                    metadata.push("async".to_string());
                }
            }
        }

        Symbol::new(
            name,
            SymbolKind::Function,
            file_path.clone(),
            node.start_position().row + 1,
            node.start_position().column + 1,
        )
        .with_visibility(visibility.unwrap_or_else(|| "private".to_string()))
        .with_metadata(metadata)
    }

    fn create_class_symbol(&self, node: &Node, name: String, _source: &str, file_path: &PathBuf) -> Symbol {
        let visibility = self.extract_visibility(node);
        Symbol::new(
            name,
            SymbolKind::Type, // Using Type for class
            file_path.clone(),
            node.start_position().row + 1,
            node.start_position().column + 1,
        )
        .with_visibility(visibility.unwrap_or_else(|| "private".to_string()))
        .with_metadata(vec!["class".to_string()])
    }

    fn create_interface_symbol(&self, node: &Node, name: String, _source: &str, file_path: &PathBuf) -> Symbol {
        let visibility = self.extract_visibility(node);
        Symbol::new(
            name,
            SymbolKind::Type,
            file_path.clone(),
            node.start_position().row + 1,
            node.start_position().column + 1,
        )
        .with_visibility(visibility.unwrap_or_else(|| "private".to_string()))
        .with_metadata(vec!["interface".to_string()])
    }

    fn create_type_symbol(&self, node: &Node, name: String, _source: &str, file_path: &PathBuf) -> Symbol {
        let visibility = self.extract_visibility(node);
        Symbol::new(
            name,
            SymbolKind::Type,
            file_path.clone(),
            node.start_position().row + 1,
            node.start_position().column + 1,
        )
        .with_visibility(visibility.unwrap_or_else(|| "private".to_string()))
        .with_metadata(vec!["type".to_string()])
    }

    fn create_enum_symbol(&self, node: &Node, name: String, _source: &str, file_path: &PathBuf) -> Symbol {
        let visibility = self.extract_visibility(node);
        Symbol::new(
            name,
            SymbolKind::Enum,
            file_path.clone(),
            node.start_position().row + 1,
            node.start_position().column + 1,
        )
        .with_visibility(visibility.unwrap_or_else(|| "private".to_string()))
    }

    fn extract_visibility(&self, node: &Node) -> Option<String> {
        // Check parent nodes for export
        let mut current = node.parent();
        while let Some(n) = current {
            if n.kind() == "export_statement" {
                return Some("exported".to_string());
            }
            current = n.parent();
        }
        None
    }
}

impl Default for TypeScriptAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_function() {
        let mut analyzer = TypeScriptAnalyzer::new();
        let source = "export function test() {}";
        let symbols = analyzer.extract_symbols(source, PathBuf::from("test.ts"), false).unwrap();
        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "test");
        assert_eq!(symbols[0].kind, SymbolKind::Function);
    }

    #[test]
    fn test_extract_class() {
        let mut analyzer = TypeScriptAnalyzer::new();
        let source = "export class User { constructor() {} }";
        let symbols = analyzer.extract_symbols(source, PathBuf::from("test.ts"), false).unwrap();
        assert!(symbols.iter().any(|s| s.name == "User"));
    }
}
