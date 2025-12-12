//! Definition lookup tool for orchestration.
//!
//! This module provides the `find_definition` tool that allows agents to find
//! where symbols are defined in source files.

use async_trait::async_trait;
use std::path::PathBuf;
use std::sync::Arc;
use tree_sitter::{Node, Parser};
use tree_sitter_rust::language as rust_language;
use tree_sitter_typescript::{language_tsx, language_typescript};

use super::file_tools::WorkspaceRootProvider;
use super::tool::{Tool, ToolArguments, ToolHandler, ToolParameters, ToolResult};
use crate::error::{OrchestrationError, Result};

/// Symbol definition
#[derive(Debug, Clone)]
struct Definition {
    name: String,
    file_path: PathBuf,
    line: usize,
    column: usize,
    kind: String,
}

/// Rust definition finder
struct RustDefinitionFinder {
    parser: Parser,
}

impl RustDefinitionFinder {
    fn new() -> Self {
        let mut parser = Parser::new();
        parser.set_language(rust_language()).expect("Failed to load Rust grammar");
        Self { parser }
    }

    fn find_definition(&mut self, source: &str, symbol_name: &str) -> Option<Definition> {
        let tree = self.parser.parse(source, None)?;
        self.search_node(&tree.root_node(), source, symbol_name)
    }

    fn search_node(&self, node: &Node, source: &str, symbol_name: &str) -> Option<Definition> {
        match node.kind() {
            "function_item" | "struct_item" | "enum_item" | "trait_item" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let name = name_node.utf8_text(source.as_bytes()).unwrap_or("").to_string();
                    if name == symbol_name {
                        return Some(Definition {
                            name,
                            file_path: PathBuf::new(), // Will be set by caller
                            line: node.start_position().row + 1,
                            column: node.start_position().column + 1,
                            kind: node.kind().to_string(),
                        });
                    }
                }
            }
            _ => {}
        }

        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                if let Some(def) = self.search_node(&child, source, symbol_name) {
                    return Some(def);
                }
            }
        }
        None
    }
}

/// TypeScript definition finder
struct TypeScriptDefinitionFinder {
    parser: Parser,
    ts_language: tree_sitter::Language,
    tsx_language: tree_sitter::Language,
}

impl TypeScriptDefinitionFinder {
    fn new() -> Self {
        let mut parser = Parser::new();
        let ts_language = language_typescript();
        let tsx_language = language_tsx();
        parser.set_language(ts_language).expect("Failed to load TypeScript grammar");
        
        Self {
            parser,
            ts_language,
            tsx_language,
        }
    }

    fn find_definition(&mut self, source: &str, symbol_name: &str, is_tsx: bool) -> Option<Definition> {
        let language = if is_tsx { self.tsx_language } else { self.ts_language };
        self.parser.set_language(language).ok()?;
        
        let tree = self.parser.parse(source, None)?;
        self.search_node(&tree.root_node(), source, symbol_name)
    }

    fn search_node(&self, node: &Node, source: &str, symbol_name: &str) -> Option<Definition> {
        match node.kind() {
            "function_declaration" | "method_definition" | "class_declaration" 
            | "interface_declaration" | "type_alias_declaration" | "enum_declaration" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let name = name_node.utf8_text(source.as_bytes()).unwrap_or("").to_string();
                    if name == symbol_name {
                        return Some(Definition {
                            name,
                            file_path: PathBuf::new(),
                            line: node.start_position().row + 1,
                            column: node.start_position().column + 1,
                            kind: node.kind().to_string(),
                        });
                    }
                }
            }
            "variable_declarator" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let name = name_node.utf8_text(source.as_bytes()).unwrap_or("").to_string();
                    if name == symbol_name {
                        return Some(Definition {
                            name,
                            file_path: PathBuf::new(),
                            line: node.start_position().row + 1,
                            column: node.start_position().column + 1,
                            kind: "variable".to_string(),
                        });
                    }
                }
            }
            _ => {}
        }

        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                if let Some(def) = self.search_node(&child, source, symbol_name) {
                    return Some(def);
                }
            }
        }
        None
    }
}

/// Definition lookup tool handler
struct FindDefinitionHandler {
    workspace_root: Arc<dyn WorkspaceRootProvider>,
}

#[async_trait]
impl ToolHandler for FindDefinitionHandler {
    async fn execute(&self, args: &ToolArguments) -> Result<ToolResult> {
        let workspace_root = self.workspace_root.workspace_root().ok_or_else(|| {
            OrchestrationError::Other("Workspace root not available".to_string())
        })?;

        let symbol = args.get_string("symbol").ok_or_else(|| {
            OrchestrationError::InvalidToolArguments {
                tool: "find_definition".to_string(),
                reason: "Missing required 'symbol' argument".to_string(),
            }
        })?;

        let file_path_str = args.get_string("file").ok_or_else(|| {
            OrchestrationError::InvalidToolArguments {
                tool: "find_definition".to_string(),
                reason: "Missing required 'file' argument".to_string(),
            }
        })?;

        let file_path = workspace_root.join(&file_path_str);
        let source = std::fs::read_to_string(&file_path)
            .map_err(|e| OrchestrationError::Other(format!("Failed to read file: {}", e)))?;

        // Determine language from file extension
        let ext = file_path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        let definition = match ext.as_str() {
            "rs" => {
                let mut finder = RustDefinitionFinder::new();
                finder.find_definition(&source, &symbol)
            }
            "ts" | "tsx" => {
                let is_tsx = ext == "tsx";
                let mut finder = TypeScriptDefinitionFinder::new();
                finder.find_definition(&source, &symbol, is_tsx)
            }
            _ => {
                return Ok(ToolResult::error(format!(
                    "File type '{}' not supported. Currently only Rust (.rs) and TypeScript (.ts, .tsx) are supported.",
                    ext
                )));
            }
        };

        if let Some(def) = definition {
            let mut output = String::new();
            output.push_str(&format!("# Definition: {}\n\n", def.name));
            output.push_str(&format!("**File:** {}\n", file_path_str));
            output.push_str(&format!("**Location:** {}:{}\n", def.line, def.column));
            output.push_str(&format!("**Kind:** {}\n", def.kind));
            Ok(ToolResult::success(output))
        } else {
            Ok(ToolResult::success(format!(
                "Definition for '{}' not found in {}",
                symbol, file_path_str
            )))
        }
    }
}

/// Create the find_definition tool
pub fn create_find_definition_tool(workspace_root: Arc<dyn WorkspaceRootProvider>) -> Tool {
    let parameters = ToolParameters::new()
        .add_property(
            "symbol",
            "string",
            "Symbol name to find the definition for",
            true,
        )
        .add_property(
            "file",
            "string",
            "File path (relative to workspace root) to search in",
            true,
        );

    let handler = Arc::new(FindDefinitionHandler { workspace_root });

    Tool::new(
        "find_definition",
        "find_definition",
        "Find the definition of a symbol (function, struct, class, etc.) in a file. Supports Rust and TypeScript.",
        parameters,
        handler,
    )
}

/// Create all definition lookup tools
pub fn create_definition_tools(workspace_root: Arc<dyn WorkspaceRootProvider>) -> Vec<Tool> {
    vec![create_find_definition_tool(workspace_root)]
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
    async fn test_find_definition_tool_creation() {
        let temp_dir = TempDir::new().unwrap();
        let workspace_root = Arc::new(TestWorkspaceRoot {
            root: temp_dir.path().to_path_buf(),
        });

        let tool = create_find_definition_tool(workspace_root);
        assert_eq!(tool.name, "find_definition");
    }
}
