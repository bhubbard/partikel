//! MCP Stdio Server
//!
//! Exposes Partikel's Tier 0 deterministic tools and micro-agent catalog
//! to any Model Context Protocol compliant client (Claude Code, Antigravity, Cursor, Zed).

use crate::catalog::CatalogRegistry;
use crate::mcp::protocol::{CallToolResult, JsonRpcRequest, JsonRpcResponse, TextContent, Tool};
use crate::tier0::{
    calculate_lora_scale, check_contrast, compute_sigmas, compute_spring, parse_git_stat,
    sanitize_filevine_name, sort_tailwind_classes, validate_cron, SpringConfig,
};
use serde_json::{json, Value};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

#[derive(Debug)]
pub struct McpServer {
    registry: Option<CatalogRegistry>,
}

impl Default for McpServer {
    fn default() -> Self {
        Self::new()
    }
}

impl McpServer {
    pub fn new() -> Self {
        let registry = CatalogRegistry::discover().ok();
        Self { registry }
    }

    pub async fn run_stdio(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let stdin = tokio::io::stdin();
        let mut stdout = tokio::io::stdout();
        let reader = BufReader::new(stdin);
        let mut lines = reader.lines();

        while let Some(line) = lines.next_line().await? {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            let request: Result<JsonRpcRequest, _> = serde_json::from_str(line);
            match request {
                Ok(req) => {
                    let response = self.handle_request(req).await;
                    if let Some(resp) = response {
                        let json_str = serde_json::to_string(&resp)?;
                        stdout.write_all(json_str.as_bytes()).await?;
                        stdout.write_all(b"\n").await?;
                        stdout.flush().await?;
                    }
                }
                Err(e) => {
                    let resp = JsonRpcResponse::error(None, -32700, &format!("Parse error: {}", e));
                    let json_str = serde_json::to_string(&resp)?;
                    stdout.write_all(json_str.as_bytes()).await?;
                    stdout.write_all(b"\n").await?;
                    stdout.flush().await?;
                }
            }
        }

        Ok(())
    }

    async fn handle_request(&self, req: JsonRpcRequest) -> Option<JsonRpcResponse> {
        let id = req.id.clone();
        match req.method.as_str() {
            "initialize" => {
                let result = json!({
                    "protocolVersion": "2024-11-05",
                    "capabilities": {
                        "tools": {}
                    },
                    "serverInfo": {
                        "name": "partikel",
                        "version": env!("CARGO_PKG_VERSION")
                    }
                });
                Some(JsonRpcResponse::success(id, result))
            }
            "notifications/initialized" => None,
            "tools/list" => {
                let tools = self.list_tools();
                let result = json!({ "tools": tools });
                Some(JsonRpcResponse::success(id, result))
            }
            "tools/call" => {
                let params = req.params.unwrap_or(Value::Null);
                let tool_name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
                let arguments = params.get("arguments").cloned().unwrap_or(Value::Null);

                let call_result = self.call_tool(tool_name, arguments);
                let result_value = serde_json::to_value(call_result).unwrap_or(json!({
                    "content": [{"type": "text", "text": "Internal serialization error"}],
                    "isError": true
                }));
                Some(JsonRpcResponse::success(id, result_value))
            }
            _ => Some(JsonRpcResponse::error(
                id,
                -32601,
                &format!("Method not found: {}", req.method),
            )),
        }
    }

    fn list_tools(&self) -> Vec<Tool> {
        vec![
            Tool {
                name: "tier0_sigmas".to_string(),
                description: "Compute shifted discrete sigma schedule for flow matching diffusion (Wan2.1, LTX-2, FLUX, SD3)".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "steps": { "type": "integer", "description": "Number of inference steps (e.g. 8, 20, 30)" },
                        "shift": { "type": "number", "description": "Time shift factor S (default 3.0)" },
                        "timesteps": { "type": "integer", "description": "Total training timesteps (default 1000)" }
                    },
                    "required": ["steps"]
                }),
            },
            Tool {
                name: "tier0_spring".to_string(),
                description: "Compute Apple fluid spring physics constants (omega, zeta) and CSS cubic-bezier transition curves".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "response": { "type": "number", "description": "Response time in seconds (default 0.35)" },
                        "damping": { "type": "number", "description": "Damping ratio between 0.0 and 1.0 (default 0.7)" },
                        "blend": { "type": "number", "description": "Blend duration in seconds (default 0.0)" }
                    }
                }),
            },
            Tool {
                name: "tier0_filevine".to_string(),
                description: "Sanitize arbitrary field labels into valid Filevine alphanumeric camelCase field selectors".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "label": { "type": "string", "description": "Arbitrary field name to sanitize" }
                    },
                    "required": ["label"]
                }),
            },
            Tool {
                name: "tier0_lora".to_string(),
                description: "Compute exact LoRA weight delta scaling factor alpha / rank".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "rank": { "type": "integer", "description": "LoRA rank r (e.g. 16, 32, 64)" },
                        "alpha": { "type": "number", "description": "LoRA alpha parameter (e.g. 16.0, 32.0)" },
                        "multiplier": { "type": "number", "description": "Optional runtime multiplier factor (default 1.0)" }
                    },
                    "required": ["rank", "alpha"]
                }),
            },
            Tool {
                name: "tier0_tailwind".to_string(),
                description: "Deterministic box-model CSS cascade class sorter (Layout -> Sizing -> Typography -> Backgrounds -> Borders -> Effects -> Transitions)".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "classes": { "type": "string", "description": "Whitespace-separated list of Tailwind CSS classes" }
                    },
                    "required": ["classes"]
                }),
            },
            Tool {
                name: "tier0_wcag".to_string(),
                description: "WCAG 2.2 AA and AAA contrast ratio checker & luminance calculator for two hex colors".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "fg": { "type": "string", "description": "Foreground color in hex (#000000 or #000)" },
                        "bg": { "type": "string", "description": "Background color in hex (#ffffff or #fff)" }
                    },
                    "required": ["fg", "bg"]
                }),
            },
            Tool {
                name: "tier0_git_stat".to_string(),
                description: "Fast parser for git diff --stat summary into typed per-file insertions, deletions, and total diff counts".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "stat_text": { "type": "string", "description": "Raw string output of `git diff --stat`" }
                    },
                    "required": ["stat_text"]
                }),
            },
            Tool {
                name: "tier0_cron".to_string(),
                description: "Validate standard 5-field cron expression and generate human-readable explanation".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "expression": { "type": "string", "description": "5-field cron string (e.g. '*/5 * * * *')" }
                    },
                    "required": ["expression"]
                }),
            },
            Tool {
                name: "partikel_catalog_list".to_string(),
                description: "List all micro-agent specifications in the Partikel catalog, including token budgets and runtime targets".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "domain": { "type": "string", "description": "Optional domain filter (e.g. 'Rust', 'Cloudflare', 'Diffusion')" }
                    }
                }),
            },
            Tool {
                name: "partikel_catalog_get".to_string(),
                description: "Retrieve complete specification, system prompt, and input/output contracts for a micro-agent".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "name": { "type": "string", "description": "Exact micro-agent name (e.g. 'rustc-borrow-healer')" }
                    },
                    "required": ["name"]
                }),
            },
        ]
    }

    fn call_tool(&self, name: &str, args: Value) -> CallToolResult {
        match name {
            "tier0_sigmas" => {
                let steps = args.get("steps").and_then(|v| v.as_u64()).unwrap_or(8) as usize;
                let shift = args.get("shift").and_then(|v| v.as_f64()).unwrap_or(3.0);
                let timesteps = args
                    .get("timesteps")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(1000) as usize;
                match compute_sigmas(steps, shift, timesteps) {
                    Ok(res) => CallToolResult {
                        content: vec![TextContent::text(
                            serde_json::to_string_pretty(&res).unwrap_or_default(),
                        )],
                        is_error: None,
                    },
                    Err(e) => CallToolResult {
                        content: vec![TextContent::text(format!("Error: {}", e))],
                        is_error: Some(true),
                    },
                }
            }
            "tier0_spring" => {
                let response = args
                    .get("response")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.35);
                let damping = args.get("damping").and_then(|v| v.as_f64()).unwrap_or(0.7);
                let blend = args.get("blend").and_then(|v| v.as_f64()).unwrap_or(0.0);
                match compute_spring(SpringConfig {
                    response,
                    damping_ratio: damping,
                    blend_duration: blend,
                }) {
                    Ok(res) => CallToolResult {
                        content: vec![TextContent::text(
                            serde_json::to_string_pretty(&res).unwrap_or_default(),
                        )],
                        is_error: None,
                    },
                    Err(e) => CallToolResult {
                        content: vec![TextContent::text(format!("Error: {}", e))],
                        is_error: Some(true),
                    },
                }
            }
            "tier0_filevine" => {
                let label = args.get("label").and_then(|v| v.as_str()).unwrap_or("");
                let res = sanitize_filevine_name(label);
                CallToolResult {
                    content: vec![TextContent::text(
                        serde_json::to_string_pretty(&res).unwrap_or_default(),
                    )],
                    is_error: None,
                }
            }
            "tier0_lora" => {
                let rank = args.get("rank").and_then(|v| v.as_u64()).unwrap_or(16) as u32;
                let alpha = args.get("alpha").and_then(|v| v.as_f64()).unwrap_or(16.0);
                let mult = args.get("multiplier").and_then(|v| v.as_f64());
                match calculate_lora_scale(rank, alpha, mult) {
                    Ok(res) => CallToolResult {
                        content: vec![TextContent::text(
                            serde_json::to_string_pretty(&res).unwrap_or_default(),
                        )],
                        is_error: None,
                    },
                    Err(e) => CallToolResult {
                        content: vec![TextContent::text(format!("Error: {}", e))],
                        is_error: Some(true),
                    },
                }
            }
            "tier0_tailwind" => {
                let classes = args.get("classes").and_then(|v| v.as_str()).unwrap_or("");
                let res = sort_tailwind_classes(classes);
                CallToolResult {
                    content: vec![TextContent::text(
                        serde_json::to_string_pretty(&res).unwrap_or_default(),
                    )],
                    is_error: None,
                }
            }
            "tier0_wcag" => {
                let fg = args.get("fg").and_then(|v| v.as_str()).unwrap_or("#000000");
                let bg = args.get("bg").and_then(|v| v.as_str()).unwrap_or("#ffffff");
                match check_contrast(fg, bg) {
                    Ok(res) => CallToolResult {
                        content: vec![TextContent::text(
                            serde_json::to_string_pretty(&res).unwrap_or_default(),
                        )],
                        is_error: None,
                    },
                    Err(e) => CallToolResult {
                        content: vec![TextContent::text(format!("Error: {}", e))],
                        is_error: Some(true),
                    },
                }
            }
            "tier0_git_stat" => {
                let text = args.get("stat_text").and_then(|v| v.as_str()).unwrap_or("");
                let res = parse_git_stat(text);
                CallToolResult {
                    content: vec![TextContent::text(
                        serde_json::to_string_pretty(&res).unwrap_or_default(),
                    )],
                    is_error: None,
                }
            }
            "tier0_cron" => {
                let expr = args
                    .get("expression")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let res = validate_cron(expr);
                CallToolResult {
                    content: vec![TextContent::text(
                        serde_json::to_string_pretty(&res).unwrap_or_default(),
                    )],
                    is_error: None,
                }
            }
            "partikel_catalog_list" => {
                let domain = args.get("domain").and_then(|v| v.as_str());
                if let Some(reg) = &self.registry {
                    let list = match domain {
                        Some(d) => reg.filter_by_domain(d),
                        None => reg.list(),
                    };
                    let simplified: Vec<Value> = list
                        .into_iter()
                        .map(|s| {
                            json!({
                                "name": s.name,
                                "domain": s.domain,
                                "target_runtime": s.target_runtime,
                                "budget": s.budget_str,
                                "estimated_tokens": s.estimated_tokens,
                                "tier0_equivalent": s.tier0_equivalent
                            })
                        })
                        .collect();
                    CallToolResult {
                        content: vec![TextContent::text(
                            serde_json::to_string_pretty(&simplified).unwrap_or_default(),
                        )],
                        is_error: None,
                    }
                } else {
                    CallToolResult {
                        content: vec![TextContent::text("Catalog registry not loaded")],
                        is_error: Some(true),
                    }
                }
            }
            "partikel_catalog_get" => {
                let name = args.get("name").and_then(|v| v.as_str()).unwrap_or("");
                if let Some(reg) = &self.registry {
                    if let Some(spec) = reg.get(name) {
                        CallToolResult {
                            content: vec![TextContent::text(
                                serde_json::to_string_pretty(spec).unwrap_or_default(),
                            )],
                            is_error: None,
                        }
                    } else {
                        CallToolResult {
                            content: vec![TextContent::text(format!(
                                "Micro-agent '{}' not found in catalog",
                                name
                            ))],
                            is_error: Some(true),
                        }
                    }
                } else {
                    CallToolResult {
                        content: vec![TextContent::text("Catalog registry not loaded")],
                        is_error: Some(true),
                    }
                }
            }
            _ => CallToolResult {
                content: vec![TextContent::text(format!("Tool '{}' not recognized", name))],
                is_error: Some(true),
            },
        }
    }
}
