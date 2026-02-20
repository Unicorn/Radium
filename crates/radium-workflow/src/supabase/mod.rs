//! Supabase REST API client module.
//!
//! Provides a typed HTTP client for interacting with Supabase's PostgREST API,
//! used as the persistence layer for the workflow builder service.

pub mod client;
pub mod error;

pub use client::{SupabaseClient, SupabaseConfig};
pub use error::SupabaseError;
