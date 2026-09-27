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
    }
}
