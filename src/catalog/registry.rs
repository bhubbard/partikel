//! Micro-Agent Catalog Registry
//!
//! Indexes, validates, and manages all 35 micro-agent specifications.

use crate::catalog::parser::{parse_spec_file, MicroAgentSpec};
use crate::error::{PartikelError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatalogSummary {
    pub total_agents: usize,
    pub total_tokens_est: usize,
    pub avg_tokens_est: usize,
    pub within_envelope_count: usize,
    pub violations_count: usize,
    pub tier0_equivalents_count: usize,
}

#[derive(Debug, Clone)]
pub struct CatalogRegistry {
    specs: HashMap<String, MicroAgentSpec>,
    catalog_dir: PathBuf,
}

impl CatalogRegistry {
    /// Load registry from a directory containing `.md` files.
    pub fn load_from_dir<P: AsRef<Path>>(dir: P) -> Result<Self> {
        let dir_ref = dir.as_ref();
        if !dir_ref.exists() || !dir_ref.is_dir() {
            return Err(PartikelError::CatalogError(format!(
                "Directory not found: {}",
                dir_ref.display()
            )));
        }

        let mut specs = HashMap::new();
        let entries = std::fs::read_dir(dir_ref).map_err(PartikelError::Io)?;

        for entry in entries {
            let entry = entry.map_err(PartikelError::Io)?;
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) == Some("md") {
                match parse_spec_file(&path) {
                    Ok(spec) => {
                        specs.insert(spec.name.clone(), spec);
                    }
                    Err(e) => {
                        eprintln!("Warning: Failed to parse {}: {}", path.display(), e);
                    }
                }
            }
        }

        Ok(CatalogRegistry {
            specs,
            catalog_dir: dir_ref.to_path_buf(),
        })
    }

    /// Auto-discover catalog directory starting from current working directory or relative path.
    pub fn discover() -> Result<Self> {
        let candidates = [
            PathBuf::from("catalog"),
            PathBuf::from("./catalog"),
            PathBuf::from("../catalog"),
            PathBuf::from("/Users/bhubbard/PROJECTS/partikel/catalog"),
        ];

        for path in &candidates {
            if path.exists() && path.is_dir() {
                if let Ok(reg) = Self::load_from_dir(path) {
                    if !reg.is_empty() {
                        return Ok(reg);
                    }
                }
            }
        }

        Err(PartikelError::CatalogError(
            "Could not locate 'catalog' directory with micro-agent specifications".to_string(),
        ))
    }

    pub fn catalog_dir(&self) -> &Path {
        &self.catalog_dir
    }

    pub fn is_empty(&self) -> bool {
        self.specs.is_empty()
    }

    pub fn len(&self) -> usize {
        self.specs.len()
    }

    pub fn get(&self, name: &str) -> Option<&MicroAgentSpec> {
        self.specs.get(name)
    }

    pub fn list(&self) -> Vec<&MicroAgentSpec> {
        let mut list: Vec<&MicroAgentSpec> = self.specs.values().collect();
        list.sort_by(|a, b| a.name.cmp(&b.name));
        list
    }

    pub fn filter_by_domain(&self, domain_keyword: &str) -> Vec<&MicroAgentSpec> {
        let kw = domain_keyword.to_lowercase();
        self.list()
            .into_iter()
            .filter(|spec| spec.domain.to_lowercase().contains(&kw))
            .collect()
    }

    pub fn summarize(&self) -> CatalogSummary {
        let total = self.specs.len();
        let total_tokens: usize = self.specs.values().map(|s| s.estimated_tokens).sum();
        let avg_tokens = total_tokens.checked_div(total).unwrap_or(0);
        let within_env = self
            .specs
            .values()
            .filter(|s| s.is_within_envelope())
            .count();
        let violations = total - within_env;
        let t0_count = self
            .specs
            .values()
            .filter(|s| s.tier0_equivalent.is_some())
            .count();

        CatalogSummary {
            total_agents: total,
            total_tokens_est: total_tokens,
            avg_tokens_est: avg_tokens,
            within_envelope_count: within_env,
            violations_count: violations,
            tier0_equivalents_count: t0_count,
        }
    }
}
