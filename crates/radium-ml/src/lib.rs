//! Machine Learning integration for Radium.
//!
//! This crate provides ML-based routing capabilities using models like Arch-Router.

pub mod arch_router;

pub use arch_router::{
    ArchRouterConfig, ArchRouterEngine, ArchRouterResult, InferenceBackend, Message,
    RouteDefinition, RoutingDecision,
};
