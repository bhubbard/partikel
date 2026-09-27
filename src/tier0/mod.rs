//! Tier 0 Deterministic Engine
//!
//! Sub-microsecond deterministic operations running pure native Rust algorithms
//! (zero LLM token calls, zero hallucination risk, zero runtime overhead).

pub mod cron;
pub mod filevine;
pub mod git_stat;
pub mod lora;
pub mod sigmas;
pub mod spring;
pub mod tailwind;
pub mod wcag;

pub use cron::{validate_cron, CronField, CronValidation};
pub use filevine::filevine_custom_field_sanitize;
pub use git_stat::{parse_git_diff_stat, GitDiffStat, GitFileStat};
pub use lora::{lora_delta_scale, LoRAScaleResult};
pub use sigmas::flow_matching_sigmas;
pub use spring::{spring_physics_convert, SpringMotionConfig, SpringPhysicsResult};
pub use tailwind::tailwind_class_sort;
pub use wcag::{relative_luminance, wcag_contrast_check, WcagContrastResult};

// Friendly aliases matching MCP and CLI conventions
pub use filevine_custom_field_sanitize as sanitize_filevine_name;
pub use flow_matching_sigmas as compute_sigmas;
pub use parse_git_diff_stat as parse_git_stat;
pub use tailwind_class_sort as sort_tailwind_classes;
pub use wcag_contrast_check as check_contrast;

/// Helper wrapper for spring physics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SpringConfig {
    pub response: f64,
    pub damping_ratio: f64,
    pub blend_duration: f64,
}

impl Default for SpringConfig {
    fn default() -> Self {
        Self {
            response: 0.35,
            damping_ratio: 0.7,
            blend_duration: 0.0,
        }
    }
}

pub fn compute_spring(cfg: SpringConfig) -> crate::error::Result<SpringPhysicsResult> {
    spring_physics_convert(cfg.response, cfg.damping_ratio, 1.0)
}

/// Helper wrapper for LoRA scale calculation
pub fn calculate_lora_scale(
    rank: u32,
    alpha: f64,
    multiplier: Option<f64>,
) -> crate::error::Result<LoRAScaleResult> {
    let mut res = lora_delta_scale(rank as usize, Some(alpha))?;
    if let Some(m) = multiplier {
        res.scale = ((res.scale * m) * 10000.0).round() / 10000.0;
    }
    Ok(res)
}
