//! Partikel CLI module

use crate::catalog::CatalogRegistry;
use crate::mcp::McpServer;
use crate::tier0::*;
use clap::{Args, Parser, Subcommand};
use colored::*;
use std::io::Read;

#[derive(Parser, Debug)]
#[command(
    name = "partikel",
    bin_name = "partikel",
    author,
    version,
    about = "Sub-atomic micro-agent runtime & Tier 0 deterministic tool engine"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Execute Tier 0 deterministic tools directly (sub-microsecond, 0 tokens)
    #[command(subcommand)]
    Tier0(Tier0Commands),

    /// Browse, inspect, and query the 35 micro-agent catalog
    Catalog {
        #[command(subcommand)]
        command: Option<CatalogSubcommands>,
    },

    /// Run audit, token budget verification, and benchmarks on catalog
    Eval {
        /// Optional directory containing catalog markdown files
        #[arg(short, long)]
        dir: Option<String>,
    },

    /// Launch Model Context Protocol (MCP) JSON-RPC 2.0 stdio server
    Mcp,

    /// Run a micro-agent specification against inputs (or Tier 0 if available)
    Run {
        /// Name of the micro-agent
        name: String,
        /// Input string or payload
        input: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
pub enum CatalogSubcommands {
    /// List all micro-agents in catalog
    List {
        /// Filter by domain keyword
        #[arg(short, long)]
        domain: Option<String>,
    },
    /// View detailed spec for a micro-agent
    Get {
        /// Name of the micro-agent
        name: String,
    },
    /// Show summary statistics
    Stats,
}

#[derive(Subcommand, Debug)]
pub enum Tier0Commands {
    /// Calculate flow-matching discrete sigma schedule
    Sigmas(SigmasArgs),
    /// Calculate Apple fluid spring physics constants and CSS
    Spring(SpringArgs),
    /// Sanitize field name to Filevine camelCase selector
    Filevine { label: String },
    /// Calculate LoRA delta scaling factor
    Lora(LoraArgs),
    /// Deterministically sort Tailwind CSS classes by box-model specificity
    Tailwind { classes: String },
    /// Check WCAG 2.2 contrast ratio and conformance between two hex colors
    Wcag(WcagArgs),
    /// Parse `git diff --stat` output into structured metrics
    GitStat {
        /// Raw stat text (reads from stdin if omitted)
        text: Option<String>,
    },
    /// Validate standard 5-field cron expression
    Cron { expression: String },
}

#[derive(Args, Debug)]
pub struct SigmasArgs {
    #[arg(short, long, default_value_t = 8)]
    pub steps: usize,
    #[arg(short = 'S', long, default_value_t = 3.0)]
    pub shift: f64,
    #[arg(short, long, default_value_t = 1000)]
    pub timesteps: usize,
}

#[derive(Args, Debug)]
pub struct SpringArgs {
    #[arg(short, long, default_value_t = 0.35)]
    pub response: f64,
    #[arg(short, long, default_value_t = 0.7)]
    pub damping: f64,
    #[arg(short, long, default_value_t = 0.0)]
    pub blend: f64,
}

#[derive(Args, Debug)]
pub struct LoraArgs {
    #[arg(short, long)]
    pub rank: u32,
    #[arg(short, long)]
    pub alpha: f64,
    #[arg(short, long)]
    pub multiplier: Option<f64>,
}

#[derive(Args, Debug)]
pub struct WcagArgs {
    #[arg(long)]
    pub fg: String,
    #[arg(long)]
    pub bg: String,
}

pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Tier0(sub) => handle_tier0(sub),
        Commands::Catalog { command } => handle_catalog(command),
        Commands::Eval { dir } => handle_eval(dir),
        Commands::Mcp => {
            let mut server = McpServer::new();
            server.run_stdio().await?;
        }
        Commands::Run { name, input } => handle_run(&name, input),
    }

    Ok(())
}

fn handle_tier0(cmd: Tier0Commands) {
    match cmd {
        Tier0Commands::Sigmas(args) => {
            match compute_sigmas(args.steps, args.shift, args.timesteps) {
                Ok(res) => println!("{}", serde_json::to_string_pretty(&res).unwrap()),
                Err(e) => eprintln!("{}: {}", "Error".red().bold(), e),
            }
        }
        Tier0Commands::Spring(args) => {
            match compute_spring(SpringConfig {
                response: args.response,
                damping_ratio: args.damping,
                blend_duration: args.blend,
            }) {
                Ok(res) => println!("{}", serde_json::to_string_pretty(&res).unwrap()),
                Err(e) => eprintln!("{}: {}", "Error".red().bold(), e),
            }
        }
        Tier0Commands::Filevine { label } => {
            let res = sanitize_filevine_name(&label);
            println!("{}", serde_json::to_string_pretty(&res).unwrap());
        }
        Tier0Commands::Lora(args) => {
            match calculate_lora_scale(args.rank, args.alpha, args.multiplier) {
                Ok(res) => println!("{}", serde_json::to_string_pretty(&res).unwrap()),
                Err(e) => eprintln!("{}: {}", "Error".red().bold(), e),
            }
        }
        Tier0Commands::Tailwind { classes } => {
            let res = sort_tailwind_classes(&classes);
            println!("{}", serde_json::to_string_pretty(&res).unwrap());
        }
        Tier0Commands::Wcag(args) => match check_contrast(&args.fg, &args.bg) {
            Ok(res) => println!("{}", serde_json::to_string_pretty(&res).unwrap()),
            Err(e) => eprintln!("{}: {}", "Error".red().bold(), e),
        },
        Tier0Commands::GitStat { text } => {
            let input_text = match text {
                Some(t) => t,
                None => {
                    let mut buffer = String::new();
                    let _ = std::io::stdin().read_to_string(&mut buffer);
                    buffer
                }
            };
            let res = parse_git_stat(&input_text);
            println!("{}", serde_json::to_string_pretty(&res).unwrap());
        }
        Tier0Commands::Cron { expression } => {
            let res = validate_cron(&expression);
            println!("{}", serde_json::to_string_pretty(&res).unwrap());
        }
    }
}

fn handle_catalog(cmd: Option<CatalogSubcommands>) {
    let registry = match CatalogRegistry::discover() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("{}: {}", "Error".red().bold(), e);
            std::process::exit(1);
        }
    };

    match cmd.unwrap_or(CatalogSubcommands::List { domain: None }) {
        CatalogSubcommands::List { domain } => {
            let list = match domain {
                Some(ref d) => registry.filter_by_domain(d),
                None => registry.list(),
            };

            println!(
                "{}",
                format!(
                    "─── PARTIKEL MICRO-AGENT CATALOG ({} specs) ───",
                    list.len()
                )
                .cyan()
                .bold()
            );
            println!(
                "{:<35} {:<30} {:<12} {}",
                "NAME".bold(),
                "DOMAIN".bold(),
                "EST. TOKENS".bold(),
                "TIER 0 FAST-PATH".bold()
            );
            println!("{}", "─".repeat(95).dimmed());

            for s in list {
                let t0 = match &s.tier0_equivalent {
                    Some(t) => format!("✓ tier0_{}", t).green(),
                    None => "—".dimmed(),
                };
                let tokens_color = if s.is_within_envelope() {
                    format!("~{} tok", s.estimated_tokens).green()
                } else {
                    format!("~{} tok [VIOLATION]", s.estimated_tokens)
                        .red()
                        .bold()
                };

                println!(
                    "{:<35} {:<30} {:<12} {}",
                    s.name.yellow(),
                    s.domain,
                    tokens_color,
                    t0
                );
            }
        }
        CatalogSubcommands::Get { name } => {
            if let Some(spec) = registry.get(&name) {
                println!("{}", format!("Micro-Agent: {}", spec.name).cyan().bold());
                println!("Domain:         {}", spec.domain);
                println!("Runtime:        {}", spec.target_runtime);
                println!("Budget:         {}", spec.budget_str);
                println!("Est. Tokens:    {}", spec.estimated_tokens);
                println!("Tier 0 Equiv:   {:?}", spec.tier0_equivalent);
                println!("\n{}", "─── SYSTEM PROMPT ───".yellow());
                println!("{}", spec.system_prompt);
                println!("\n{}", "─── INPUT SCHEMA ───".yellow());
                println!("{}", spec.input_schema);
                println!("\n{}", "─── OUTPUT CONTRACT ───".yellow());
                println!("{}", spec.output_contract);
                println!("\n{}", "─── VERIFICATION HARNESS ───".yellow());
                println!("{}", spec.verification_harness);
            } else {
                eprintln!(
                    "{}: Micro-agent '{}' not found in catalog",
                    "Error".red().bold(),
                    name
                );
            }
        }
        CatalogSubcommands::Stats => {
            let summary = registry.summarize();
            println!("{}", "─── PARTIKEL CATALOG SUMMARY ───".cyan().bold());
            println!(
                "Total Micro-Agents:       {}",
                summary.total_agents.to_string().green().bold()
            );
            println!("Total Estimated Tokens:   {}", summary.total_tokens_est);
            println!(
                "Average Tokens per Agent: ~{} tokens",
                summary.avg_tokens_est
            );
            println!(
                "Within 4K Token Ceiling:  {}/{}",
                summary.within_envelope_count, summary.total_agents
            );
            println!(
                "Ceiling Violations (>4K): {}",
                if summary.violations_count == 0 {
                    "0 (100% compliant)".green()
                } else {
                    summary.violations_count.to_string().red()
                }
            );
            println!(
                "Direct Tier 0 Fast-Paths: {} micro-agents",
                summary.tier0_equivalents_count.to_string().yellow().bold()
            );
        }
    }
}

fn handle_eval(dir: Option<String>) {
    let registry = match dir {
        Some(d) => CatalogRegistry::load_from_dir(d),
        None => CatalogRegistry::discover(),
    };

    let reg = match registry {
        Ok(r) => r,
        Err(e) => {
            eprintln!("{}: {}", "Error".red().bold(), e);
            std::process::exit(1);
        }
    };

    let summary = reg.summarize();
    println!(
        "{}",
        "══════════════════════════════════════════════════════════".cyan()
    );
    println!(
        "{}",
        "           PARTIKEL MICRO-AGENT EVALUATION HARNESS        "
            .cyan()
            .bold()
    );
    println!(
        "{}",
        "══════════════════════════════════════════════════════════".cyan()
    );

    for s in reg.list() {
        let status = if s.is_within_envelope() {
            "PASS [<=4K]".green()
        } else {
            "FAIL [>4K]".red().bold()
        };
        let t0_status = match &s.tier0_equivalent {
            Some(t) => format!("[T0: {}]", t).yellow(),
            None => "".normal(),
        };
        println!(
            " • {:<36} {:<10} {:>5} tokens {}",
            s.name, status, s.estimated_tokens, t0_status
        );
    }

    println!(
        "{}",
        "──────────────────────────────────────────────────────────".dimmed()
    );
    println!(
        "Evaluation completed: {} total agents checked.",
        summary.total_agents
    );
    if summary.violations_count == 0 {
        println!(
            "{}",
            "All micro-agents adhere to the 4K context token ceiling!"
                .green()
                .bold()
        );
    } else {
        println!(
            "{}: {} micro-agents exceed the 4K ceiling.",
            "Warning".yellow().bold(),
            summary.violations_count
        );
    }
}

fn handle_run(name: &str, input: Option<String>) {
    let registry = CatalogRegistry::discover().ok();
    if let Some(reg) = registry {
        if let Some(spec) = reg.get(name) {
            if let Some(ref t0) = spec.tier0_equivalent {
                println!(
                    "{}",
                    format!("⚡ Fast-pathing '{}' to Tier 0 native engine: {}", name, t0)
                        .green()
                        .bold()
                );
                match t0.as_str() {
                    "sigmas" => {
                        let res = compute_sigmas(8, 3.0, 1000).unwrap();
                        println!("{}", serde_json::to_string_pretty(&res).unwrap());
                        return;
                    }
                    "spring" => {
                        let res = compute_spring(SpringConfig::default()).unwrap();
                        println!("{}", serde_json::to_string_pretty(&res).unwrap());
                        return;
                    }
                    "filevine" => {
                        let label = input.unwrap_or_else(|| "Accident Date & Time".to_string());
                        let res = sanitize_filevine_name(&label);
                        println!("{}", serde_json::to_string_pretty(&res).unwrap());
                        return;
                    }
                    "lora" => {
                        let res = calculate_lora_scale(16, 16.0, None).unwrap();
                        println!("{}", serde_json::to_string_pretty(&res).unwrap());
                        return;
                    }
                    "tailwind" => {
                        let classes = input.unwrap_or_else(|| "text-red-500 flex p-4".to_string());
                        let res = sort_tailwind_classes(&classes);
                        println!("{}", serde_json::to_string_pretty(&res).unwrap());
                        return;
                    }
                    _ => {}
                }
            }

            println!(
                "{}",
                format!("Micro-agent '{}' ready for execution.", spec.name)
                    .cyan()
                    .bold()
            );
            println!("System Prompt:\n{}", spec.system_prompt.dimmed());
            if let Some(inp) = input {
                println!("Input:\n{}", inp);
            }
        } else {
            eprintln!(
                "{}: Micro-agent '{}' not found in catalog",
                "Error".red().bold(),
                name
            );
        }
    } else {
        eprintln!("{}: Catalog directory not found", "Error".red().bold());
    }
}
