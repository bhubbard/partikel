//! # Partikel (ptk)
//!
//! Sub-atomic micro-agent runtime, native Tier 0 deterministic tool engine,
//! and Model Context Protocol (MCP) server.

pub mod catalog;
pub mod cli;
pub mod error;
pub mod mcp;
pub mod tier0;

pub use catalog::{CatalogRegistry, CatalogSummary, MicroAgentSpec};
pub use error::{PartikelError, Result};
pub use mcp::McpServer;
