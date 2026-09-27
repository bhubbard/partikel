use crate::error::{PartikelError, Result};

/// Calculate shifted discrete flow matching sigma schedules in nanoseconds.
/// Formula: sigma_t = (shift * t) / (1 + (shift - 1) * t)
pub fn flow_matching_sigmas(
    steps: usize,
    shift: f64,
    num_train_timesteps: usize,
) -> Result<Vec<f64>> {
    if steps == 0 {
        return Err(PartikelError::Tier0("steps must be greater than 0".into()));
    }
    if shift <= 0.0 {
        return Err(PartikelError::Tier0("shift must be positive".into()));
    }

    let mut sigmas = Vec::with_capacity(steps + 1);
    let tt = num_train_timesteps as f64;

    for i in 0..steps {
        let t = 1.0 - (i as f64 / steps as f64) * (1.0 - 1.0 / tt);
        let sigma = (shift * t) / (1.0 + (shift - 1.0) * t);
        // Round to 4 decimal places
        sigmas.push((sigma * 10000.0).round() / 10000.0);
    }
    sigmas.push(0.0);

    Ok(sigmas)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flow_matching_sigmas() {
        let sigmas = flow_matching_sigmas(8, 3.0, 1000).unwrap();
        assert_eq!(sigmas.len(), 9);
        assert!((sigmas[0] - 1.0).abs() < 0.05);
        assert_eq!(sigmas[8], 0.0);
        for i in 0..8 {
            assert!(sigmas[i] > sigmas[i + 1]);
        }
    }
}
