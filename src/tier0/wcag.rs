use crate::error::{PartikelError, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WcagContrastResult {
    pub foreground: String,
    pub background: String,
    pub contrast_ratio: f64,
    pub passes_aa_normal: bool,
    pub passes_aa_large: bool,
    pub passes_aaa_normal: bool,
    pub passes_aaa_large: bool,
}

fn parse_hex_color(hex: &str) -> Result<(f64, f64, f64)> {
    let clean = hex.trim_start_matches('#');
    if clean.len() != 6 {
        return Err(PartikelError::Tier0(format!(
            "Invalid 6-character hex color: {}",
            hex
        )));
    }

    let r = u8::from_str_radix(&clean[0..2], 16)
        .map_err(|_| PartikelError::Tier0(format!("Invalid red hex in {}", hex)))?;
    let g = u8::from_str_radix(&clean[2..4], 16)
        .map_err(|_| PartikelError::Tier0(format!("Invalid green hex in {}", hex)))?;
    let b = u8::from_str_radix(&clean[4..6], 16)
        .map_err(|_| PartikelError::Tier0(format!("Invalid blue hex in {}", hex)))?;

    Ok((r as f64 / 255.0, g as f64 / 255.0, b as f64 / 255.0))
}

fn srgb_to_linear(c: f64) -> f64 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

pub fn relative_luminance(r: f64, g: f64, b: f64) -> f64 {
    0.2126 * srgb_to_linear(r) + 0.7152 * srgb_to_linear(g) + 0.0722 * srgb_to_linear(b)
}

pub fn wcag_contrast_check(fg_hex: &str, bg_hex: &str) -> Result<WcagContrastResult> {
    let (r1, g1, b1) = parse_hex_color(fg_hex)?;
    let (r2, g2, b2) = parse_hex_color(bg_hex)?;

    let l1 = relative_luminance(r1, g1, b1);
    let l2 = relative_luminance(r2, g2, b2);

    let (lighter, darker) = if l1 > l2 { (l1, l2) } else { (l2, l1) };
    let ratio = ((lighter + 0.05) / (darker + 0.05) * 100.0).round() / 100.0;

    Ok(WcagContrastResult {
        foreground: fg_hex.to_string(),
        background: bg_hex.to_string(),
        contrast_ratio: ratio,
        passes_aa_normal: ratio >= 4.5,
        passes_aa_large: ratio >= 3.0,
        passes_aaa_normal: ratio >= 7.0,
        passes_aaa_large: ratio >= 4.5,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wcag_contrast() {
        // Pure black on pure white is 21:1
        let bw = wcag_contrast_check("#000000", "#ffffff").unwrap();
        assert_eq!(bw.contrast_ratio, 21.0);
        assert!(bw.passes_aaa_normal);

        // White on white is 1:1
        let ww = wcag_contrast_check("#ffffff", "#ffffff").unwrap();
        assert_eq!(ww.contrast_ratio, 1.0);
        assert!(!ww.passes_aa_normal);
    }
}
