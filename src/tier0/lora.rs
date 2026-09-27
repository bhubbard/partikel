use crate::error::{PartikelError, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoRAScaleResult {
    pub rank: usize,
    pub alpha: f64,
    pub scale: f64,
}

pub fn lora_delta_scale(rank: usize, alpha: Option<f64>) -> Result<LoRAScaleResult> {
    if rank == 0 {
        return Err(PartikelError::Tier0("rank must be greater than 0".into()));
    }
    let a = alpha.unwrap_or(rank as f64);
    let scale = ((a / rank as f64) * 10000.0).round() / 10000.0;

    Ok(LoRAScaleResult {
        rank,
        alpha: a,
        scale,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lora_scale() {
        let res = lora_delta_scale(16, Some(32.0)).unwrap();
        assert_eq!(res.scale, 2.0);

        let default_alpha = lora_delta_scale(16, None).unwrap();
        assert_eq!(default_alpha.scale, 1.0);
    }
}
