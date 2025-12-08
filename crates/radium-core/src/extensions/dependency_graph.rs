//! Extension dependency graph construction and visualization.
//!
//! Provides functionality for building and visualizing extension dependency graphs.

use crate::extensions::structure::Extension;
use std::collections::{HashMap, HashSet};
use thiserror::Error;

/// Dependency graph errors.
#[derive(Debug, Error)]
pub enum DependencyGraphError {
    /// Extension not found.
    #[error("extension not found: {0}")]
    ExtensionNotFound(String),

    /// Circular dependency detected.
    #[error("circular dependency detected: {0}")]
    CircularDependency(String),
}

/// Result type for graph operations.
pub type Result<T> = std::result::Result<T, DependencyGraphError>;

/// Dependency graph node.
#[derive(Debug, Clone)]
pub struct GraphNode {
    /// Extension name.
    pub name: String,
    /// Extension version.
    pub version: String,
    /// Dependencies (extension names).
    pub dependencies: Vec<String>,
}

/// Dependency graph.
#[derive(Debug, Clone)]
pub struct DependencyGraph {
    /// Graph nodes (extension name -> node).
    nodes: HashMap<String, GraphNode>,
    /// Reverse dependencies (extension name -> extensions that depend on it).
    reverse_deps: HashMap<String, Vec<String>>,
}

impl DependencyGraph {
    /// Builds a dependency graph from installed extensions.
    pub fn from_extensions(extensions: &[Extension]) -> Self {
        let mut nodes = HashMap::new();
        let mut reverse_deps: HashMap<String, Vec<String>> = HashMap::new();

        for ext in extensions {
            let deps = ext.manifest.dependencies.clone();
            let node = GraphNode {
                name: ext.name.clone(),
                version: ext.version.clone(),
                dependencies: deps.clone(),
            };
            nodes.insert(ext.name.clone(), node);

            // Build reverse dependencies
            for dep in &deps {
                reverse_deps.entry(dep.clone())
                    .or_insert_with(Vec::new)
                    .push(ext.name.clone());
            }
        }

        Self { nodes, reverse_deps }
    }

    /// Gets a node by name.
    pub fn get_node(&self, name: &str) -> Option<&GraphNode> {
        self.nodes.get(name)
    }

    /// Gets all nodes.
    pub fn nodes(&self) -> impl Iterator<Item = &GraphNode> {
        self.nodes.values()
    }

    /// Gets reverse dependencies (extensions that depend on this one).
    pub fn get_reverse_deps(&self, name: &str) -> &[String] {
        self.reverse_deps.get(name).map(|v| v.as_slice()).unwrap_or(&[])
    }

    /// Detects circular dependencies.
    pub fn detect_cycles(&self) -> Vec<Vec<String>> {
        let mut cycles = Vec::new();
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        let mut path = Vec::new();

        for node_name in self.nodes.keys() {
            if !visited.contains(node_name) {
                self.dfs_cycle(node_name, &mut visited, &mut rec_stack, &mut path, &mut cycles);
            }
        }

        cycles
    }

    fn dfs_cycle(
        &self,
        node: &str,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
        path: &mut Vec<String>,
        cycles: &mut Vec<Vec<String>>,
    ) {
        visited.insert(node.to_string());
        rec_stack.insert(node.to_string());
        path.push(node.to_string());

        if let Some(node_data) = self.nodes.get(node) {
            for dep in &node_data.dependencies {
                if !visited.contains(dep) {
                    self.dfs_cycle(dep, visited, rec_stack, path, cycles);
                } else if rec_stack.contains(dep) {
                    // Found cycle
                    let cycle_start = path.iter().position(|n| n == dep).unwrap();
                    cycles.push(path[cycle_start..].to_vec());
                }
            }
        }

        rec_stack.remove(node);
        path.pop();
    }

    /// Gets topological sort (installation order).
    pub fn topological_sort(&self) -> Result<Vec<String>> {
        // Check for cycles first
        let cycles = self.detect_cycles();
        if !cycles.is_empty() {
            return Err(DependencyGraphError::CircularDependency(
                format!("Circular dependencies detected: {:?}", cycles[0])
            ));
        }

        let mut result = Vec::new();
        let mut visited = HashSet::new();

        for node_name in self.nodes.keys() {
            if !visited.contains(node_name) {
                self.dfs_topological(node_name, &mut visited, &mut result);
            }
        }

        result.reverse();
        Ok(result)
    }

    fn dfs_topological(
        &self,
        node: &str,
        visited: &mut HashSet<String>,
        result: &mut Vec<String>,
    ) {
        visited.insert(node.to_string());

        if let Some(node_data) = self.nodes.get(node) {
            for dep in &node_data.dependencies {
                if !visited.contains(dep) {
                    self.dfs_topological(dep, visited, result);
                }
            }
        }

        result.push(node.to_string());
    }

    /// Gets dependencies for a specific extension (transitive).
    pub fn get_transitive_deps(&self, name: &str) -> Result<HashSet<String>> {
        let mut deps = HashSet::new();
        let mut to_process = vec![name.to_string()];

        while let Some(current) = to_process.pop() {
            if let Some(node) = self.nodes.get(&current) {
                for dep in &node.dependencies {
                    if !deps.contains(dep) {
                        deps.insert(dep.clone());
                        to_process.push(dep.clone());
                    }
                }
            }
        }

        Ok(deps)
    }

    /// Exports graph to DOT format.
    pub fn to_dot(&self) -> String {
        let mut dot = String::from("digraph ExtensionDependencies {\n");
        dot.push_str("  rankdir=LR;\n");
        dot.push_str("  node [shape=box];\n\n");

        for node in self.nodes.values() {
            dot.push_str(&format!("  \"{}\" [label=\"{}\\n{}\"];\n", 
                node.name, node.name, node.version));

            for dep in &node.dependencies {
                dot.push_str(&format!("  \"{}\" -> \"{}\";\n", node.name, dep));
            }
        }

        dot.push_str("}\n");
        dot
    }

    /// Exports graph to JSON format.
    pub fn to_json(&self) -> serde_json::Value {
        use serde_json::json;

        let nodes: Vec<_> = self.nodes.values().map(|n| {
            json!({
                "name": n.name,
                "version": n.version,
                "dependencies": n.dependencies,
            })
        }).collect();

        json!({
            "nodes": nodes,
            "edges": self.get_all_edges(),
        })
    }

    fn get_all_edges(&self) -> Vec<serde_json::Value> {
        let mut edges = Vec::new();
        for node in self.nodes.values() {
            for dep in &node.dependencies {
                edges.push(serde_json::json!({
                    "from": node.name,
                    "to": dep,
                }));
            }
        }
        edges
    }

    /// Formats graph as ASCII tree.
    pub fn to_ascii_tree(&self, root: Option<&str>) -> String {
        let mut output = String::new();

        if let Some(root_name) = root {
            if let Some(node) = self.nodes.get(root_name) {
                output.push_str(&self.format_node_tree(node, "", true, &mut HashSet::new()));
            }
        } else {
            // Find root nodes (nodes with no dependencies or no reverse deps)
            let root_nodes: Vec<_> = self.nodes.values()
                .filter(|n| n.dependencies.is_empty() || 
                    self.reverse_deps.get(&n.name).map(|v| v.is_empty()).unwrap_or(true))
                .collect();

            for (i, node) in root_nodes.iter().enumerate() {
                let is_last = i == root_nodes.len() - 1;
                output.push_str(&self.format_node_tree(node, "", is_last, &mut HashSet::new()));
            }
        }

        output
    }

    fn format_node_tree(
        &self,
        node: &GraphNode,
        prefix: &str,
        is_last: bool,
        visited: &mut HashSet<String>,
    ) -> String {
        if visited.contains(&node.name) {
            return format!("{}└── {} (circular)\n", prefix, node.name);
        }
        visited.insert(node.name.clone());

        let mut output = String::new();
        let connector = if is_last { "└── " } else { "├── " };
        output.push_str(&format!("{}{}{} v{}\n", prefix, connector, node.name, node.version));

        let new_prefix = if is_last {
            format!("{}    ", prefix)
        } else {
            format!("{}│   ", prefix)
        };

        let deps: Vec<_> = node.dependencies.iter()
            .filter_map(|dep| self.nodes.get(dep))
            .collect();

        for (i, dep) in deps.iter().enumerate() {
            let is_last_dep = i == deps.len() - 1;
            output.push_str(&self.format_node_tree(dep, &new_prefix, is_last_dep, visited));
        }

        visited.remove(&node.name);
        output
    }
}

