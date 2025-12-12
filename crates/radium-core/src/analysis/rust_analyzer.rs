//! Rust-specific symbol extraction using tree-sitter.

use std::path::PathBuf;
use tree_sitter::Node;
use crate::analysis::symbols::{Symbol, SymbolKind, SymbolSearchResult};
use crate::analysis::tree_sitter::TreeSitterParser;

/// Rust code analyzer for symbol extraction.
pub struct RustAnalyzer {
    parser: TreeSitterParser,
}

impl RustAnalyzer {
    /// Create a new Rust analyzer.
    pub fn new() -> Self {
        Self {
            parser: TreeSitterParser::new(),
        }
    }

    /// Extract all symbols from a Rust source file.
    pub fn extract_symbols(&mut self, source: &str, file_path: PathBuf) -> Result<Vec<Symbol>, String> {
        let tree = self.parser.parse_rust(source)
            .ok_or_else(|| "Parse error".to_string())?;

        let mut symbols = Vec::new();
        let root = tree.root_node();
        
        self.extract_from_node(root, source, &file_path, &mut symbols);

        Ok(symbols)
    }

    /// Search for symbols matching a query.
    pub fn search_symbols(&mut self, source: &str, file_path: PathBuf, query: &str) -> Result<SymbolSearchResult, String> {
        let all_symbols = self.extract_symbols(source, file_path)?;
        
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
    fn extract_from_node(&self, node: Node, source: &str, file_path: &PathBuf, symbols: &mut Vec<Symbol>) {
        let node_kind = node.kind();
        
        match node_kind {
            "function_item" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let name = name_node.utf8_text(source.as_bytes()).unwrap_or("").to_string();
                    let symbol = self.create_function_symbol(node, name, source, file_path);
                    symbols.push(symbol);
                }
            }
            "struct_item" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let name = name_node.utf8_text(source.as_bytes()).unwrap_or("").to_string();
                    let symbol = self.create_struct_symbol(node, name, source, file_path);
                    symbols.push(symbol);
                }
            }
            "enum_item" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let name = name_node.utf8_text(source.as_bytes()).unwrap_or("").to_string();
                    let symbol = self.create_enum_symbol(node, name, source, file_path);
                    symbols.push(symbol);
                }
            }
            "trait_item" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let name = name_node.utf8_text(source.as_bytes()).unwrap_or("").to_string();
                    let symbol = self.create_trait_symbol(node, name, source, file_path);
                    symbols.push(symbol);
                }
            }
            "impl_item" => {
                // Extract trait name if present, or type name
                if let Some(trait_node) = node.child_by_field_name("trait") {
                    if let Some(name_node) = trait_node.child_by_field_name("name") {
                        let name = name_node.utf8_text(source.as_bytes()).unwrap_or("").to_string();
                        let symbol = self.create_impl_symbol(node, name, source, file_path, true);
                        symbols.push(symbol);
                    }
                } else if let Some(type_node) = node.child_by_field_name("type") {
                    if let Some(name_node) = type_node.child_by_field_name("name") {
                        let name = name_node.utf8_text(source.as_bytes()).unwrap_or("").to_string();
                        let symbol = self.create_impl_symbol(node, name, source, file_path, false);
                        symbols.push(symbol);
                    }
                }
            }
            "module" => {
                // Extract module name if present
                if let Some(name_node) = node.child_by_field_name("name") {
                    let name = name_node.utf8_text(source.as_bytes()).unwrap_or("").to_string();
                    let symbol = Symbol::new(
                        name,
                        SymbolKind::Module,
                        file_path.clone(),
                        node.start_position().row + 1,
                        node.start_position().column + 1,
                    );
                    symbols.push(symbol);
                }
            }
            _ => {}
        }

        // Recursively process children
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.extract_from_node(child, source, file_path, symbols);
            }
        }
    }

    fn create_function_symbol(&self, node: Node, name: String, source: &str, file_path: &PathBuf) -> Symbol {
        let mut metadata = Vec::new();
        let mut visibility = None;

        // Check for visibility modifiers
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                match child.kind() {
                    "visibility_modifier" => {
                        let vis_text = child.utf8_text(source.as_bytes()).unwrap_or("").trim();
                        if vis_text == "pub" {
                            visibility = Some("public".to_string());
                        }
                    }
                    "async" => metadata.push("async".to_string()),
                    "const" => metadata.push("const".to_string()),
                    "unsafe" => metadata.push("unsafe".to_string()),
                    _ => {}
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

    fn create_struct_symbol(&self, node: Node, name: String, source: &str, file_path: &PathBuf) -> Symbol {
        let mut visibility = None;

        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                if child.kind() == "visibility_modifier" {
                    let vis_text = child.utf8_text(source.as_bytes()).unwrap_or("").trim();
                    if vis_text == "pub" {
                        visibility = Some("public".to_string());
                    }
                }
            }
        }

        Symbol::new(
            name,
            SymbolKind::Struct,
            file_path.clone(),
            node.start_position().row + 1,
            node.start_position().column + 1,
        )
        .with_visibility(visibility.unwrap_or_else(|| "private".to_string()))
    }

    fn create_enum_symbol(&self, node: Node, name: String, source: &str, file_path: &PathBuf) -> Symbol {
        let mut visibility = None;

        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                if child.kind() == "visibility_modifier" {
                    let vis_text = child.utf8_text(source.as_bytes()).unwrap_or("").trim();
                    if vis_text == "pub" {
                        visibility = Some("public".to_string());
                    }
                }
            }
        }

        Symbol::new(
            name,
            SymbolKind::Enum,
            file_path.clone(),
            node.start_position().row + 1,
            node.start_position().column + 1,
        )
        .with_visibility(visibility.unwrap_or_else(|| "private".to_string()))
    }

    fn create_trait_symbol(&self, node: Node, name: String, source: &str, file_path: &PathBuf) -> Symbol {
        let mut visibility = None;

        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                if child.kind() == "visibility_modifier" {
                    let vis_text = child.utf8_text(source.as_bytes()).unwrap_or("").trim();
                    if vis_text == "pub" {
                        visibility = Some("public".to_string());
                    }
                }
            }
        }

        Symbol::new(
            name,
            SymbolKind::Trait,
            file_path.clone(),
            node.start_position().row + 1,
            node.start_position().column + 1,
        )
        .with_visibility(visibility.unwrap_or_else(|| "private".to_string()))
    }

    fn create_impl_symbol(&self, node: Node, name: String, _source: &str, file_path: &PathBuf, is_trait: bool) -> Symbol {
        Symbol::new(
            name,
            SymbolKind::Impl,
            file_path.clone(),
            node.start_position().row + 1,
            node.start_position().column + 1,
        )
        .with_metadata(if is_trait { vec!["trait".to_string()] } else { vec![] })
    }
}

impl Default for RustAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_function() {
        let mut analyzer = RustAnalyzer::new();
        let source = "pub fn test() {}";
        let symbols = analyzer.extract_symbols(source, PathBuf::from("test.rs")).unwrap();
        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "test");
        assert_eq!(symbols[0].kind, SymbolKind::Function);
        assert_eq!(symbols[0].visibility, Some("public".to_string()));
    }

    #[test]
    fn test_extract_struct() {
        let mut analyzer = RustAnalyzer::new();
        let source = "struct User { name: String }";
        let symbols = analyzer.extract_symbols(source, PathBuf::from("test.rs")).unwrap();
        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "User");
        assert_eq!(symbols[0].kind, SymbolKind::Struct);
    }
}
