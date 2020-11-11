//! The Lemmy server crate
#![recursion_limit = "512"]
#![deny(missing_docs)]

/// The Database migrations that require code
pub mod code_migrations;

/// The API routes
pub mod routes;
