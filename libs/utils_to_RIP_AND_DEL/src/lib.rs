//! # The Utils Library
#![forbid(unsafe_code)]

/// Config utilities based on config-rs
pub mod config;

/// Error helpers for GraphQL
pub mod errors;

/// Pagination utils
pub mod pagination;

/// Ordering utils
pub mod ordering;

#[macro_use]
extern crate anyhow;
