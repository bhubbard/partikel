//! Micro-Agent Catalog Parser & Schema Validator
//!
//! Parses micro-agent markdown specifications in `catalog/*.md` and enforces
//! the 4K token ceiling, output contracts, and system prompt boundaries.

use crate::error::{PartikelError, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MicroAgentSpec {
    pub name: String,
    pub domain: String,
    pub target_runtime: String,
    pub budget_str: String,
    pub system_prompt: String,
    pub input_schema: String,
    pub output_contract: String,
    pub verification_harness: String,
    pub estimated_tokens: usize,
    pub tier0_equivalent: Option<String>,
    pub file_path: String,
}

impl MicroAgentSpec {
    pub fn is_within_envelope(&self) -> bool {
        self.estimated_tokens <= 4000
    }
}

/// Parse a micro-agent markdown file.
pub fn parse_spec_file<P: AsRef<Path>>(path: P) -> Result<MicroAgentSpec> {
    let path_ref = path.as_ref();
    let content = std::fs::read_to_string(path_ref).map_err(PartikelError::Io)?;
    parse_spec_content(&content, &path_ref.display().to_string())
}

/// Parse micro-agent markdown content string.
pub fn parse_spec_content(content: &str, file_path: &str) -> Result<MicroAgentSpec> {
    let name = extract_between(content, "# Micro-Agent: `", "`")
        .or_else(|| extract_between(content, "# Micro-Agent: ", "\n"))
        .unwrap_or_else(|| {
            Path::new(file_path)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string()
        });

    let domain = extract_metadata_field(content, "Domain").unwrap_or_else(|| "General".to_string());
    let target_runtime = extract_metadata_field(content, "Target Runtime")
        .unwrap_or_else(|| "Generic SLM".to_string());
    let budget_str =
        extract_metadata_field(content, "Total Budget").unwrap_or_else(|| "Unknown".to_string());

    let system_prompt = extract_section_code_or_text(content, "## System Prompt");
    let input_schema = extract_section_code_or_text(content, "## Input Schema");
    let output_contract = extract_section_code_or_text(content, "## Output Contract");
    let verification_harness = extract_section_raw(content, "## Verification Harness");

    // Token estimation (~4 characters per token in standard LLM tokenizers)
    let total_chars = content.chars().count();
    let estimated_tokens = total_chars.div_ceil(4);

    let tier0_equivalent = match name.as_str() {
        "flow-matching-sigma-calculator" => Some("sigmas".to_string()),
        "spring-physics-converter" => Some("spring".to_string()),
        "filevine-custom-field-sanitizer" => Some("filevine".to_string()),
        "lora-delta-calculator" => Some("lora".to_string()),
        "tailwind-class-sorter" => Some("tailwind".to_string()),
        "layout-shift-cls-healer" => Some("tailwind".to_string()),
        _ => None,
    };

    Ok(MicroAgentSpec {
        name,
        domain,
        target_runtime,
        budget_str,
        system_prompt,
        input_schema,
        output_contract,
        verification_harness,
        estimated_tokens,
        tier0_equivalent,
        file_path: file_path.to_string(),
    })
}

fn extract_between(source: &str, start_tag: &str, end_tag: &str) -> Option<String> {
    if let Some(start_idx) = source.find(start_tag) {
        let after_start = &source[start_idx + start_tag.len()..];
        if let Some(end_idx) = after_start.find(end_tag) {
            return Some(after_start[..end_idx].trim().to_string());
        }
    }
    None
}

fn extract_metadata_field(source: &str, field_name: &str) -> Option<String> {
    let pattern = format!("- **{}**:", field_name);
    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with(&pattern) {
            let val = trimmed[pattern.len()..].trim();
            return Some(val.to_string());
        }
    }
    None
}

fn extract_section_code_or_text(source: &str, section_header: &str) -> String {
    if let Some(header_idx) = source.find(section_header) {
        let after_header = &source[header_idx + section_header.len()..];
        let end_idx = after_header
            .find("\n## ")
            .or_else(|| after_header.find("\n---"))
            .unwrap_or(after_header.len());
        let section = &after_header[..end_idx].trim();

        // Check if wrapped in code block
        if let Some(code_start) = section.find("```") {
            let after_code = &section[code_start + 3..];
            let code_body_start = after_code.find('\n').map(|i| i + 1).unwrap_or(0);
            let code_body = &after_code[code_body_start..];
            if let Some(code_end) = code_body.rfind("```") {
                return code_body[..code_end].trim().to_string();
            }
        }
        return section.to_string();
    }
    String::new()
}

fn extract_section_raw(source: &str, section_header: &str) -> String {
    if let Some(header_idx) = source.find(section_header) {
        let after_header = &source[header_idx + section_header.len()..];
        let end_idx = after_header.find("\n## ").unwrap_or(after_header.len());
        return after_header[..end_idx].trim().to_string();
    }
    String::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"
# Micro-Agent: `flow-matching-sigma-calculator`

- **Domain**: Diffusion & Flow Matching
- **Target Runtime**: apfel-rs
- **Total Budget**: ~210 tokens

---

## System Prompt
```text
You are a mathematical flow matching sigma schedule calculator.
```

---

## Input Schema
```text
steps: 8
```

---

## Output Contract
```json
[0.9990, 0.0000]
```

---

## Verification Harness
- Validator: Monotonicity
"#;

    #[test]
    fn test_parser() {
        let spec = parse_spec_content(SAMPLE, "flow-matching-sigma-calculator.md").unwrap();
        assert_eq!(spec.name, "flow-matching-sigma-calculator");
        assert_eq!(spec.domain, "Diffusion & Flow Matching");
        assert_eq!(
            spec.system_prompt,
            "You are a mathematical flow matching sigma schedule calculator."
        );
        assert_eq!(spec.input_schema, "steps: 8");
        assert_eq!(spec.output_contract, "[0.9990, 0.0000]");
        assert!(spec.is_within_envelope());
        assert_eq!(spec.tier0_equivalent, Some("sigmas".to_string()));
    }
}
