//! Code analysis tool for extracting code structure
//!
//! This module provides the `analyze_code_structure` tool that analyzes source code
//! to extract functions, types, imports, and other structural elements.
//!
//! Currently uses regex-based parsing. Can be enhanced with tree-sitter AST parsing.

use async_trait::async_trait;
use regex::Regex;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;

use super::file_tools::WorkspaceRootProvider;
use super::tool::{Tool, ToolArguments, ToolHandler, ToolParameters, ToolResult};
use crate::error::{OrchestrationError, Result};

/// Code analysis operation handler
struct CodeAnalysisHandler {
    /// Workspace root provider
    workspace_root: Arc<dyn WorkspaceRootProvider>,
}

#[async_trait]
impl ToolHandler for CodeAnalysisHandler {
    async fn execute(&self, args: &ToolArguments) -> Result<ToolResult> {
        let workspace_root = self.workspace_root.workspace_root().ok_or_else(|| {
            OrchestrationError::Other("Workspace root not available".to_string())
        })?;

        let file_path_str = args.get_string("file_path").ok_or_else(|| {
            OrchestrationError::InvalidToolArguments {
                tool: "analyze_code_structure".to_string(),
                reason: "Missing required 'file_path' argument".to_string(),
            }
        })?;

        let detail_level = args.get_string("detail_level").unwrap_or_else(|| "summary".to_string());

        let file_path = if PathBuf::from(&file_path_str).is_absolute() {
            PathBuf::from(&file_path_str)
        } else {
            workspace_root.join(&file_path_str)
        };

        // Read file
        let code = fs::read_to_string(&file_path).await.map_err(|e| {
            OrchestrationError::Other(format!("Failed to read file {}: {}", file_path.display(), e))
        })?;

        // Detect language
        let language = detect_language(&file_path)?;

        // Analyze structure based on language
        let analysis = match language {
            Language::Rust => analyze_rust_code(&code, &detail_level),
            Language::JavaScript | Language::TypeScript => analyze_js_ts_code(&code, &detail_level),
            Language::Python => analyze_python_code(&code, &detail_level),
            Language::Go => analyze_go_code(&code, &detail_level),
            Language::Other => {
                return Ok(ToolResult::success(format!(
                    "Language detection failed for {}. Supported: .rs, .js, .ts, .py, .go",
                    file_path.display()
                )));
            }
        };

        Ok(ToolResult::success(analysis))
    }
}

/// Programming language detection
#[derive(Debug, Clone, Copy, PartialEq)]
enum Language {
    Rust,
    JavaScript,
    TypeScript,
    Python,
    Go,
    Other,
}

/// Analyze code file and return structure analysis
pub async fn analyze_code_file(
    workspace_root: &Path,
    file_path: &str,
    detail_level: &str,
) -> Result<String> {
    let full_path = workspace_root.join(file_path);

    if !full_path.exists() {
        return Err(OrchestrationError::Other(format!("File not found: {}", file_path)));
    }

    let code = tokio::fs::read_to_string(&full_path)
        .await
        .map_err(|e| OrchestrationError::Other(format!("Failed to read file: {}", e)))?;

    let language = detect_language(&full_path)?;

    let analysis = match language {
        Language::Rust => analyze_rust_code(&code, detail_level),
        Language::JavaScript | Language::TypeScript => analyze_js_ts_code(&code, detail_level),
        Language::Python => analyze_python_code(&code, detail_level),
        Language::Go => analyze_go_code(&code, detail_level),
        Language::Other => {
            return Err(OrchestrationError::Other(
                "Unsupported language for code analysis".to_string(),
            ))
        }
    };

    Ok(analysis)
}

/// Detect language from file extension
fn detect_language(file_path: &Path) -> Result<Language> {
    let ext = file_path
        .extension()
        .and_then(|e| e.to_str())
        .ok_or_else(|| OrchestrationError::Other("No file extension".to_string()))?;

    Ok(match ext {
        "rs" => Language::Rust,
        "js" | "jsx" => Language::JavaScript,
        "ts" | "tsx" => Language::TypeScript,
        "py" => Language::Python,
        "go" => Language::Go,
        _ => Language::Other,
    })
}

/// Analyze Rust code structure
fn analyze_rust_code(code: &str, detail_level: &str) -> String {
    let mut output = String::new();
    output.push_str("# Rust Code Analysis\n\n");

    // Extract imports/uses
    let use_regex = Regex::new(r"(?m)^use\s+([^;]+);").unwrap();
    let uses: Vec<_> = use_regex
        .captures_iter(code)
        .map(|cap| cap[1].trim().to_string())
        .collect();

    if !uses.is_empty() {
        output.push_str("## Imports\n\n");
        for use_stmt in &uses {
            output.push_str(&format!("- `use {};`\n", use_stmt));
        }
        output.push('\n');
    }

    // Extract structs
    let struct_regex = if detail_level == "detailed" {
        Regex::new(r"(?m)^(?:pub\s+)?struct\s+(\w+)(?:<[^>]+>)?(?:\s*\{[^\}]*\})?").unwrap()
    } else {
        Regex::new(r"(?m)^(?:pub\s+)?struct\s+(\w+)").unwrap()
    };

    let structs: Vec<_> = struct_regex
        .captures_iter(code)
        .map(|cap| cap[1].to_string())
        .collect();

    if !structs.is_empty() {
        output.push_str("## Structs\n\n");
        for struct_name in &structs {
            output.push_str(&format!("- `struct {}`\n", struct_name));
        }
        output.push('\n');
    }

    // Extract enums
    let enum_regex = Regex::new(r"(?m)^(?:pub\s+)?enum\s+(\w+)").unwrap();
    let enums: Vec<_> = enum_regex
        .captures_iter(code)
        .map(|cap| cap[1].to_string())
        .collect();

    if !enums.is_empty() {
        output.push_str("## Enums\n\n");
        for enum_name in &enums {
            output.push_str(&format!("- `enum {}`\n", enum_name));
        }
        output.push('\n');
    }

    // Extract functions
    let fn_regex = if detail_level == "detailed" {
        Regex::new(r"(?m)^(?:\s*)(?:pub\s+)?(?:async\s+)?fn\s+(\w+)(<[^>]+>)?\s*\(([^\)]*)\)(?:\s*->\s*([^\{]+))?").unwrap()
    } else {
        Regex::new(r"(?m)^(?:\s*)(?:pub\s+)?(?:async\s+)?fn\s+(\w+)").unwrap()
    };

    let functions: Vec<_> = fn_regex.captures_iter(code).collect();

    if !functions.is_empty() {
        output.push_str(&format!("## Functions ({})\n\n", functions.len()));
        for cap in &functions {
            let name = &cap[1];
            if detail_level == "detailed" {
                let params = cap.get(3).map(|m| m.as_str()).unwrap_or("");
                let return_type = cap.get(4).map(|m| m.as_str().trim()).unwrap_or("()");
                output.push_str(&format!("- `fn {}({}) -> {}`\n", name, params, return_type));
            } else {
                output.push_str(&format!("- `{}`\n", name));
            }
        }
        output.push('\n');
    }

    // Extract traits
    let trait_regex = Regex::new(r"(?m)^(?:pub\s+)?trait\s+(\w+)").unwrap();
    let traits: Vec<_> = trait_regex
        .captures_iter(code)
        .map(|cap| cap[1].to_string())
        .collect();

    if !traits.is_empty() {
        output.push_str("## Traits\n\n");
        for trait_name in &traits {
            output.push_str(&format!("- `trait {}`\n", trait_name));
        }
        output.push('\n');
    }

    // Count lines
    let total_lines = code.lines().count();
    let code_lines = code.lines().filter(|l| !l.trim().is_empty() && !l.trim().starts_with("//")).count();
    output.push_str(&format!("## Statistics\n\n- Total lines: {}\n- Code lines (approx): {}\n", total_lines, code_lines));

    output
}

/// Analyze JavaScript/TypeScript code structure
fn analyze_js_ts_code(code: &str, _detail_level: &str) -> String {
    let mut output = String::new();
    output.push_str("# JavaScript/TypeScript Code Analysis\n\n");

    // Extract imports
    let import_regex = Regex::new(r#"(?m)^import\s+(.+?)\s+from\s+['"]([^'"]+)['"];?"#).unwrap();
    let imports: Vec<_> = import_regex.captures_iter(code).collect();

    if !imports.is_empty() {
        output.push_str("## Imports\n\n");
        for cap in imports {
            output.push_str(&format!("- `import {} from '{}'`\n", &cap[1], &cap[2]));
        }
        output.push('\n');
    }

    // Extract classes
    let class_regex = Regex::new(r"(?m)^(?:export\s+)?class\s+(\w+)").unwrap();
    let classes: Vec<_> = class_regex
        .captures_iter(code)
        .map(|cap| cap[1].to_string())
        .collect();

    if !classes.is_empty() {
        output.push_str("## Classes\n\n");
        for class_name in &classes {
            output.push_str(&format!("- `class {}`\n", class_name));
        }
        output.push('\n');
    }

    // Extract functions
    let fn_regex = Regex::new(r"(?m)^(?:export\s+)?(?:async\s+)?function\s+(\w+)").unwrap();
    let arrow_fn_regex = Regex::new(r"(?m)^(?:export\s+)?const\s+(\w+)\s*=\s*(?:async\s*)?\([^\)]*\)\s*=>").unwrap();

    let mut functions: Vec<String> = fn_regex
        .captures_iter(code)
        .map(|cap| cap[1].to_string())
        .collect();

    functions.extend(
        arrow_fn_regex
            .captures_iter(code)
            .map(|cap| cap[1].to_string()),
    );

    if !functions.is_empty() {
        output.push_str(&format!("## Functions ({})\n\n", functions.len()));
        for func_name in &functions {
            output.push_str(&format!("- `{}`\n", func_name));
        }
        output.push('\n');
    }

    // Extract interfaces (TypeScript)
    let interface_regex = Regex::new(r"(?m)^(?:export\s+)?interface\s+(\w+)").unwrap();
    let interfaces: Vec<_> = interface_regex
        .captures_iter(code)
        .map(|cap| cap[1].to_string())
        .collect();

    if !interfaces.is_empty() {
        output.push_str("## Interfaces\n\n");
        for interface_name in &interfaces {
            output.push_str(&format!("- `interface {}`\n", interface_name));
        }
        output.push('\n');
    }

    let total_lines = code.lines().count();
    let code_lines = code.lines().filter(|l| !l.trim().is_empty() && !l.trim().starts_with("//")).count();
    output.push_str(&format!("## Statistics\n\n- Total lines: {}\n- Code lines (approx): {}\n", total_lines, code_lines));

    output
}

/// Analyze Python code structure
fn analyze_python_code(code: &str, _detail_level: &str) -> String {
    let mut output = String::new();
    output.push_str("# Python Code Analysis\n\n");

    // Extract imports
    let import_regex = Regex::new(r"(?m)^(?:from\s+(\S+)\s+)?import\s+(.+)").unwrap();
    let imports: Vec<_> = import_regex.captures_iter(code).collect();

    if !imports.is_empty() {
        output.push_str("## Imports\n\n");
        for cap in imports {
            if let Some(from_module) = cap.get(1) {
                output.push_str(&format!("- `from {} import {}`\n", from_module.as_str(), &cap[2]));
            } else {
                output.push_str(&format!("- `import {}`\n", &cap[2]));
            }
        }
        output.push('\n');
    }

    // Extract classes
    let class_regex = Regex::new(r"(?m)^class\s+(\w+)").unwrap();
    let classes: Vec<_> = class_regex
        .captures_iter(code)
        .map(|cap| cap[1].to_string())
        .collect();

    if !classes.is_empty() {
        output.push_str("## Classes\n\n");
        for class_name in &classes {
            output.push_str(&format!("- `class {}`\n", class_name));
        }
        output.push('\n');
    }

    // Extract functions
    let fn_regex = Regex::new(r"(?m)^def\s+(\w+)").unwrap();
    let functions: Vec<_> = fn_regex
        .captures_iter(code)
        .map(|cap| cap[1].to_string())
        .collect();

    if !functions.is_empty() {
        output.push_str(&format!("## Functions ({})\n\n", functions.len()));
        for func_name in &functions {
            output.push_str(&format!("- `{}`\n", func_name));
        }
        output.push('\n');
    }

    let total_lines = code.lines().count();
    let code_lines = code.lines().filter(|l| !l.trim().is_empty() && !l.trim().starts_with("#")).count();
    output.push_str(&format!("## Statistics\n\n- Total lines: {}\n- Code lines (approx): {}\n", total_lines, code_lines));

    output
}

/// Analyze Go code structure
fn analyze_go_code(code: &str, _detail_level: &str) -> String {
    let mut output = String::new();
    output.push_str("# Go Code Analysis\n\n");

    // Extract imports
    let import_regex = Regex::new(r#"(?m)import\s+(?:"([^"]+)"|([^\s]+))"#).unwrap();
    let imports: Vec<_> = import_regex
        .captures_iter(code)
        .map(|cap| cap.get(1).or(cap.get(2)).unwrap().as_str().to_string())
        .collect();

    if !imports.is_empty() {
        output.push_str("## Imports\n\n");
        for import in &imports {
            output.push_str(&format!("- `import \"{}\"`\n", import));
        }
        output.push('\n');
    }

    // Extract structs
    let struct_regex = Regex::new(r"(?m)^type\s+(\w+)\s+struct").unwrap();
    let structs: Vec<_> = struct_regex
        .captures_iter(code)
        .map(|cap| cap[1].to_string())
        .collect();

    if !structs.is_empty() {
        output.push_str("## Structs\n\n");
        for struct_name in &structs {
            output.push_str(&format!("- `type {} struct`\n", struct_name));
        }
        output.push('\n');
    }

    // Extract functions
    let fn_regex = Regex::new(r"(?m)^func\s+(?:\([^\)]+\)\s+)?(\w+)").unwrap();
    let functions: Vec<_> = fn_regex
        .captures_iter(code)
        .map(|cap| cap[1].to_string())
        .collect();

    if !functions.is_empty() {
        output.push_str(&format!("## Functions ({})\n\n", functions.len()));
        for func_name in &functions {
            output.push_str(&format!("- `{}`\n", func_name));
        }
        output.push('\n');
    }

    let total_lines = code.lines().count();
    let code_lines = code.lines().filter(|l| !l.trim().is_empty() && !l.trim().starts_with("//")).count();
    output.push_str(&format!("## Statistics\n\n- Total lines: {}\n- Code lines (approx): {}\n", total_lines, code_lines));

    output
}

/// Create the analyze_code_structure tool
pub fn create_code_analysis_tool(workspace_root: Arc<dyn WorkspaceRootProvider>) -> Tool {
    let parameters = ToolParameters::new()
        .add_property(
            "file_path",
            "string",
            "Path to the file to analyze (relative to workspace root or absolute)",
            true,
        )
        .add_property(
            "detail_level",
            "string",
            "Detail level: 'summary' (names only) or 'detailed' (with signatures)",
            false,
        );

    let handler = Arc::new(CodeAnalysisHandler { workspace_root });

    Tool::new(
        "analyze_code_structure",
        "analyze_code_structure",
        "Parse code file to extract functions, classes, types, imports (supports Rust, JS/TS, Python, Go)",
        parameters,
        handler,
    )
}

/// Create all code analysis tools
pub fn create_code_analysis_tools(workspace_root: Arc<dyn WorkspaceRootProvider>) -> Vec<Tool> {
    vec![create_code_analysis_tool(workspace_root)]
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

    #[test]
    fn test_detect_language() {
        assert_eq!(detect_language(Path::new("test.rs")).unwrap(), Language::Rust);
        assert_eq!(detect_language(Path::new("test.js")).unwrap(), Language::JavaScript);
        assert_eq!(detect_language(Path::new("test.ts")).unwrap(), Language::TypeScript);
        assert_eq!(detect_language(Path::new("test.py")).unwrap(), Language::Python);
        assert_eq!(detect_language(Path::new("test.go")).unwrap(), Language::Go);
    }

    #[test]
    fn test_analyze_rust_code() {
        let code = r#"
use std::collections::HashMap;

pub struct MyStruct {
    field: String,
}

pub fn my_function(arg: i32) -> String {
    "test".to_string()
}

pub async fn async_function() -> Result<()> {
    Ok(())
}
"#;

        let result = analyze_rust_code(code, "summary");
        assert!(result.contains("Imports"));
        assert!(result.contains("std::collections::HashMap"));
        assert!(result.contains("Structs"));
        assert!(result.contains("MyStruct"));
        assert!(result.contains("Functions"));
        assert!(result.contains("my_function"));
        assert!(result.contains("async_function"));
    }

    #[tokio::test]
    async fn test_code_analysis_tool() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.rs");

        let code = r#"
pub fn hello() {
    println!("Hello, world!");
}

pub struct TestStruct {
    value: i32,
}
"#;

        tokio::fs::write(&file_path, code).await.unwrap();

        let workspace_root = Arc::new(TestWorkspaceRoot {
            root: temp_dir.path().to_path_buf(),
        });

        let tool = create_code_analysis_tool(workspace_root);
        let args = ToolArguments::new(serde_json::json!({
            "file_path": "test.rs",
            "detail_level": "summary"
        }));

        let result = tool.execute(&args).await.unwrap();
        assert!(result.success);
        assert!(result.output.contains("hello"));
        assert!(result.output.contains("TestStruct"));
    }
}
