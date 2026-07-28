//! Beholder is a structural index of a codebase that answers two questions:
//! what is risky, and what depends on what.
//!
//! The crate is organized around the phase boundary described in the README.
//!
//! - [`phase1`] is a pure function of one file's content.
//! - [`store`] persists phase results in a custom git ref so they travel with
//!   the repository.
//!
//! Language support lives in [`lang`] plus the query files beside it. No
//! language has a code path of its own.

pub mod analysis;
pub mod config;
pub mod hash;
pub mod jsonl;
pub mod lang;
pub mod phase1;
pub mod store;
pub mod symbol;
pub mod tier0;
pub mod walk;

pub use analysis::Analysis;
pub use config::Config;
pub use phase1::FileAnalysis;
pub use store::Store;
pub use symbol::Symbol;

/// Version of the record schemas beholder writes.
///
/// Stored results produced under a different schema version are never reused.
pub const SCHEMA_VERSION: u32 = 1;

/// Version of the tool that produced a result.
pub const TOOL_VERSION: &str = env!("CARGO_PKG_VERSION");
