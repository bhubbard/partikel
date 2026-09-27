//! Micro-Agent Catalog
//!
//! Enforces the 4K token ceiling, YAML/Markdown specifications, and
//! fast indexation of micro-agents.

pub mod parser;
pub mod registry;

pub use parser::{parse_spec_content, parse_spec_file, MicroAgentSpec};
pub use registry::{CatalogRegistry, CatalogSummary};
