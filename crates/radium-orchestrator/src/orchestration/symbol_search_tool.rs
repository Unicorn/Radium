//! Symbol search tool for orchestration.
//!
//! This module provides the `search_symbols` tool that allows agents to search
//! for code symbols (functions, structs, enums, etc.) using tree-sitter AST analysis.

use async_trait::async_trait;
use std::path::PathBuf;
use std::sync::Arc;
use tree_sitter::{Node, Parser};

use super::file_tools::WorkspaceRootProvider;
use super::tool::{Tool, ToolArguments, ToolHandler, ToolParameters, ToolResult};
use crate::error::OrchestrationError;

/// Symbol kind
#[derive(Debug, Clone, PartialEq, Eq)]
enum SymbolKind {
    Function,
    Struct,
    Enum,
    Trait,
    Impl,
    Module,
}

/// A code symbol
#[derive(Debug, Clone)]
struct Symbol {
    name: String,
    kind: SymbolKind,
    file_path: PathBuf,
    line: usize,
    column: usize,
    visibility: Option<String>,
    metadata: Vec<String>,
}

/// Rust analyzer for symbol extraction
struct RustAnalyzer {
    parser: Parser,
}

impl RustAnalyzer {
    fn new() -> Self {
        let mut parser = Parser::new();
        let rust_language = tree_sitter_rust::language();
        parser.set_language(rust_language).expect("Failed to load Rust grammar");
        
        Self {
            parser,
        }
    }

    fn extract_symbols(&mut self, source: &str, file_path: PathBuf) -> std::result::Result<Vec<Symbol>, String> {
        let tree = self.parser.parse(source, None)
            .ok_or_else(|| "Parse error".to_string())?;

        let mut symbols = Vec::new();
        let root = tree.root_node();
        self.extract_from_node(&root, source, &file_path, &mut symbols);
        Ok(symbols)
    }

    fn search_symbols(&mut self, source: &str, file_path: PathBuf, query: &str) -> std::result::Result<Vec<Symbol>, String> {
        let all_symbols = self.extract_symbols(source, file_path)?;
        
        if query == "*" || query.is_empty() {
            Ok(all_symbols)
        } else {
            let query_lower = query.to_lowercase();
            Ok(all_symbols.into_iter()
                .filter(|s| s.name.to_lowercase().contains(&query_lower))
                .collect())
        }
    }

    fn extract_from_node(&self, node: &Node, source: &str, file_path: &PathBuf, symbols: &mut Vec<Symbol>) {
        match node.kind() {
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
                    symbols.push(self.create_struct_symbol(node, name, source, file_path));
                }
            }
            "enum_item" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let name = name_node.utf8_text(source.as_bytes()).unwrap_or("").to_string();
                    symbols.push(self.create_enum_symbol(node, name, source, file_path));
                }
            }
            "trait_item" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let name = name_node.utf8_text(source.as_bytes()).unwrap_or("").to_string();
                    symbols.push(self.create_trait_symbol(node, name, source, file_path));
                }
            }
            "impl_item" => {
                if let Some(trait_node) = node.child_by_field_name("trait") {
                    if let Some(name_node) = trait_node.child_by_field_name("name") {
                        let name = name_node.utf8_text(source.as_bytes()).unwrap_or("").to_string();
                        symbols.push(self.create_impl_symbol(node, name, source, file_path, true));
                    }
                } else if let Some(type_node) = node.child_by_field_name("type") {
                    if let Some(name_node) = type_node.child_by_field_name("name") {
                        let name = name_node.utf8_text(source.as_bytes()).unwrap_or("").to_string();
                        symbols.push(self.create_impl_symbol(node, name, source, file_path, false));
                    }
                }
            }
            _ => {}
        }

        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.extract_from_node(&child, source, file_path, symbols);
            }
        }
    }

    fn create_function_symbol(&self, node: &Node, name: String, source: &str, file_path: &PathBuf) -> Symbol {
        let mut metadata = Vec::new();
        let mut visibility = None;

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

        Symbol {
            name,
            kind: SymbolKind::Function,
            file_path: file_path.clone(),
            line: node.start_position().row + 1,
            column: node.start_position().column + 1,
            visibility: visibility.or_else(|| Some("private".to_string())),
            metadata,
        }
    }

    fn create_struct_symbol(&self, node: &Node, name: String, source: &str, file_path: &PathBuf) -> Symbol {
        let visibility = self.extract_visibility(node, source);
        Symbol {
            name,
            kind: SymbolKind::Struct,
            file_path: file_path.clone(),
            line: node.start_position().row + 1,
            column: node.start_position().column + 1,
            visibility: visibility.or_else(|| Some("private".to_string())),
            metadata: Vec::new(),
        }
    }

    fn create_enum_symbol(&self, node: &Node, name: String, source: &str, file_path: &PathBuf) -> Symbol {
        let visibility = self.extract_visibility(node, source);
        Symbol {
            name,
            kind: SymbolKind::Enum,
            file_path: file_path.clone(),
            line: node.start_position().row + 1,
            column: node.start_position().column + 1,
            visibility: visibility.or_else(|| Some("private".to_string())),
            metadata: Vec::new(),
        }
    }

    fn create_trait_symbol(&self, node: &Node, name: String, source: &str, file_path: &PathBuf) -> Symbol {
        let visibility = self.extract_visibility(node, source);
        Symbol {
            name,
            kind: SymbolKind::Trait,
            file_path: file_path.clone(),
            line: node.start_position().row + 1,
            column: node.start_position().column + 1,
            visibility: visibility.or_else(|| Some("private".to_string())),
            metadata: Vec::new(),
        }
    }

    fn create_impl_symbol(&self, node: &Node, name: String, source: &str, file_path: &PathBuf, is_trait: bool) -> Symbol {
        Symbol {
            name,
            kind: SymbolKind::Impl,
            file_path: file_path.clone(),
            line: node.start_position().row + 1,
            column: node.start_position().column + 1,
            visibility: None,
            metadata: if is_trait { vec!["trait".to_string()] } else { vec![] },
        }
    }

    fn extract_visibility(&self, node: &Node, source: &str) -> Option<String> {
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                if child.kind() == "visibility_modifier" {
                    let vis_text = child.utf8_text(source.as_bytes()).unwrap_or("").trim();
                    if vis_text == "pub" {
                        return Some("public".to_string());
                    }
                }
            }
        }
        None
    }
}

/// Symbol search tool handler
struct SymbolSearchHandler {
    workspace_root: Arc<dyn WorkspaceRootProvider>,
}

#[async_trait]
impl ToolHandler for SymbolSearchHandler {
    async fn execute(&self, args: &ToolArguments) -> crate::error::Result<ToolResult> {
        let workspace_root = self.workspace_root.workspace_root().ok_or_else(|| {
            OrchestrationError::Other("Workspace root not available".to_string())
        })?;

        let query = args.get_string("query").unwrap_or_else(|| "*".to_string());
        let language = args.get_string("language").unwrap_or_else(|| "rust".to_string());

        if language.to_lowercase() != "rust" {
            return Ok(ToolResult::error(format!(
                "Language '{}' not yet supported. Currently only 'rust' is supported.",
                language
            )));
        }

        // Find Rust files
        use super::search_tool;
        let rust_files = search_tool::search_code_internal(
            &workspace_root,
            "",
            0,
            "language:rust",
            1000,
        ).map_err(|e| OrchestrationError::Other(format!("Failed to find Rust files: {}", e)))?;

        let mut all_symbols = Vec::new();
        let mut analyzer = RustAnalyzer::new();

        for file_result in rust_files {
            let file_path = file_result.file_path;
            let source = match std::fs::read_to_string(&file_path) {
                Ok(s) => s,
                Err(_) => continue,
            };

            match analyzer.search_symbols(&source, file_path.clone(), &query) {
                Ok(symbols) => all_symbols.extend(symbols),
                Err(_) => continue,
            }
        }

        if all_symbols.is_empty() {
            return Ok(ToolResult::success(format!("No symbols found matching '{}'", query)));
        }

        // Format results
        let mut output = String::new();
        output.push_str(&format!("# Symbol Search Results ({} found)\n\n", all_symbols.len()));

        for symbol in all_symbols {
            output.push_str(&format!("## {} ({:?})\n", symbol.name, symbol.kind));
            output.push_str(&format!("**File:** {}\n", symbol.file_path.display()));
            output.push_str(&format!("**Location:** {}:{}\n", symbol.line, symbol.column));
            if let Some(ref vis) = symbol.visibility {
                output.push_str(&format!("**Visibility:** {}\n", vis));
            }
            if !symbol.metadata.is_empty() {
                output.push_str(&format!("**Metadata:** {}\n", symbol.metadata.join(", ")));
            }
            output.push_str("\n---\n\n");
        }

        Ok(ToolResult::success(output))
    }
}

/// Create the search_symbols tool
pub fn create_search_symbols_tool(workspace_root: Arc<dyn WorkspaceRootProvider>) -> Tool {
    let parameters = ToolParameters::new()
        .add_property(
            "query",
            "string",
            "Symbol name to search for (use '*' to list all symbols). Default: '*'",
            false,
        )
        .add_property(
            "language",
            "string",
            "Programming language (currently only 'rust' is supported). Default: 'rust'",
            false,
        );

    let handler = Arc::new(SymbolSearchHandler { workspace_root });

    Tool::new(
        "search_symbols",
        "search_symbols",
        "Search for code symbols (functions, structs, enums, traits, impls) using AST analysis. Currently supports Rust only.",
        parameters,
        handler,
    )
}

/// Create all symbol search tools
pub fn create_symbol_search_tools(workspace_root: Arc<dyn WorkspaceRootProvider>) -> Vec<Tool> {
    vec![create_search_symbols_tool(workspace_root)]
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    struct TestWorkspaceRoot {
        root: PathBuf,
    }

    impl WorkspaceRootProvider for TestWorkspaceRoot {
        fn workspace_root(&self) -> Option<PathBuf> {
            Some(self.root.clone())
        }
    }

    #[tokio::test]
    async fn test_search_symbols_tool_creation() {
        let temp_dir = TempDir::new().unwrap();
        let workspace_root = Arc::new(TestWorkspaceRoot {
            root: temp_dir.path().to_path_buf(),
        });

        let tool = create_search_symbols_tool(workspace_root);
        assert_eq!(tool.name, "search_symbols");
    }
}
