use crate::error::{PartikelError, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpringMotionConfig {
    pub r#type: String,
    pub stiffness: f64,
    pub damping: f64,
    pub mass: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpringPhysicsResult {
    pub response: f64,
    pub damping_ratio: f64,
    pub stiffness: f64,
    pub damping: f64,
    pub mass: f64,
    pub motion_config: SpringMotionConfig,
    pub css_transition: String,
}

impl SpringPhysicsResult {
    /// Evaluates the analytical second-order harmonic oscillator displacement trajectory `x(t)`
    /// from `0.0` toward `1.0` across `num_samples` evenly spaced points over `duration_s` seconds.
    /// Supports underdamped (`zeta < 1`), critically damped (`zeta == 1`), and overdamped (`zeta > 1`) regimes.
    pub fn sample_trajectory(&self, num_samples: usize, duration_s: f64) -> Vec<f64> {
        if num_samples == 0 {
            return Vec::new();
        }
        let omega_n = (self.stiffness / self.mass.max(1e-6)).sqrt();
        let zeta = self.damping_ratio.max(0.0);
        let mut points = Vec::with_capacity(num_samples);

        for i in 0..num_samples {
            let t = if num_samples > 1 {
                (i as f64 / (num_samples - 1) as f64) * duration_s
            } else {
                0.0
            };
            let displacement = if (zeta - 1.0).abs() < 1e-5 {
                // Critically damped: x(t) = 1 - (1 + omega_n * t) * exp(-omega_n * t)
                1.0 - (1.0 + omega_n * t) * (-omega_n * t).exp()
            } else if zeta < 1.0 {
                // Underdamped: x(t) = 1 - exp(-zeta * omega_n * t) * (cos(omega_d * t) + (zeta / sqrt(1 - zeta^2)) * sin(omega_d * t))
                let omega_d = omega_n * (1.0 - zeta * zeta).sqrt();
                let env = (-zeta * omega_n * t).exp();
                let coeff = zeta / (1.0 - zeta * zeta).sqrt();
                1.0 - env * ((omega_d * t).cos() + coeff * (omega_d * t).sin())
            } else {
                // Overdamped
                let s = (zeta * zeta - 1.0).sqrt();
                let r1 = -omega_n * (zeta - s);
                let r2 = -omega_n * (zeta + s);
                let c1 = r2 / (r2 - r1);
                let c2 = -r1 / (r2 - r1);
                1.0 - (c1 * (r1 * t).exp() + c2 * (r2 * t).exp())
            };
            points.push((displacement * 10000.0).round() / 10000.0);
        }
        points
    }
}

/// Convert designer parameters (response, damping_ratio) into physical spring constants
pub fn spring_physics_convert(
    response: f64,
    damping_ratio: f64,
    mass: f64,
) -> Result<SpringPhysicsResult> {
    if response <= 0.0 {
        return Err(PartikelError::Tier0("response must be positive".into()));
    }
    if damping_ratio < 0.0 {
        return Err(PartikelError::Tier0(
            "damping_ratio must be non-negative".into(),
        ));
    }
    if mass <= 0.0 {
        return Err(PartikelError::Tier0("mass must be positive".into()));
    }

    let omega_n = (2.0 * std::f64::consts::PI) / response;
    let stiffness = ((mass * omega_n * omega_n) * 100.0).round() / 100.0;
    let damping = ((2.0 * mass * omega_n * damping_ratio) * 100.0).round() / 100.0;
    let duration_ms = (response * 1000.0) as u64;

    Ok(SpringPhysicsResult {
        response,
        damping_ratio,
        stiffness,
        damping,
        mass,
        motion_config: SpringMotionConfig {
            r#type: "spring".to_string(),
            stiffness,
            damping,
            mass,
        },
        css_transition: format!("transform {}ms cubic-bezier(0.16, 1, 0.3, 1)", duration_ms),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spring_physics() {
        let res = spring_physics_convert(0.4, 0.85, 1.0).unwrap();
        assert_eq!(res.stiffness, 246.74);
        assert_eq!(res.damping, 26.7);
        assert!(res.css_transition.contains("400ms"));
        let curve = res.sample_trajectory(9, 0.8);
        assert_eq!(curve.len(), 9);
        assert_eq!(curve[0], 0.0);
        assert!((curve[8] - 1.0).abs() < 0.05);
    }
}
