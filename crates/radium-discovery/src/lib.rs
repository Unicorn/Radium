//! Radium Discovery Service — component, service, and project search
//!
//! This crate provides the discovery service API for indexing, searching,
//! and comparing items in a Neo4j graph database with optional embedding-based
//! semantic search.

pub mod api;
pub mod config;
pub mod embeddings;
pub mod graph;
pub mod inference;
pub mod state;
