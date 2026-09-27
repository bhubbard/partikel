//! Tier 0 Deterministic Tool: 5-Field Standard Cron Validator & Explainer
//!
//! Validates standard 5-field cron expressions:
//! `minute hour day-of-month month day-of-week`
//! without making LLM calls or running external daemon processes.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CronValidation {
    pub expression: String,
    pub is_valid: bool,
    pub fields: Vec<CronField>,
    pub human_description: String,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CronField {
    pub name: String,
    pub value: String,
    pub min: u32,
    pub max: u32,
    pub is_valid: bool,
    pub error: Option<String>,
}

/// Validates a standard 5-field cron expression.
pub fn validate_cron(expression: &str) -> CronValidation {
    let trimmed = expression.trim();
    let parts: Vec<&str> = trimmed.split_whitespace().collect();

    if parts.len() != 5 {
        return CronValidation {
            expression: expression.to_string(),
            is_valid: false,
            fields: Vec::new(),
            human_description: "Invalid format".to_string(),
            error: Some(format!(
                "Expected 5 fields (minute hour day month day-of-week), found {}",
                parts.len()
            )),
        };
    }

    let field_specs = [
        ("minute", 0, 59),
        ("hour", 0, 23),
        ("day-of-month", 1, 31),
        ("month", 1, 12),
        ("day-of-week", 0, 7), // 0 or 7 = Sunday
    ];

    let mut fields = Vec::with_capacity(5);
    let mut all_valid = true;
    let mut first_error = None;

    for (i, &(name, min, max)) in field_specs.iter().enumerate() {
        let val = parts[i];
        let (valid, err) = validate_field(val, min, max);
        if !valid {
            all_valid = false;
            if first_error.is_none() {
                first_error = Some(format!(
                    "Field '{}' with value '{}': {}",
                    name,
                    val,
                    err.clone().unwrap_or_default()
                ));
            }
        }
        fields.push(CronField {
            name: name.to_string(),
            value: val.to_string(),
            min,
            max,
            is_valid: valid,
            error: err,
        });
    }

    let human_description = if all_valid {
        describe_cron(&parts)
    } else {
        "Invalid cron schedule".to_string()
    };

    CronValidation {
        expression: expression.to_string(),
        is_valid: all_valid,
        fields,
        human_description,
        error: first_error,
    }
}

fn validate_field(field: &str, min: u32, max: u32) -> (bool, Option<String>) {
    for part in field.split(',') {
        if !validate_single_token(part, min, max) {
            return (
                false,
                Some(format!(
                    "Invalid sub-expression '{}' for range {}-{}",
                    part, min, max
                )),
            );
        }
    }
    (true, None)
}

fn validate_single_token(token: &str, min: u32, max: u32) -> bool {
    let token = token.trim();
    if token == "*" {
        return true;
    }

    // Step pattern: */n or 1-5/2
    if let Some((range, step)) = token.split_once('/') {
        let step_num = match step.parse::<u32>() {
            Ok(n) if n > 0 => n,
            _ => return false,
        };
        if range == "*" {
            return step_num <= max;
        }
        return validate_range_or_num(range, min, max);
    }

    // Range: x-y
    if token.contains('-') {
        return validate_range_or_num(token, min, max);
    }

    // Single number
    if let Ok(num) = token.parse::<u32>() {
        return num >= min && num <= max;
    }

    false
}

fn validate_range_or_num(range_str: &str, min: u32, max: u32) -> bool {
    if let Some((start_s, end_s)) = range_str.split_once('-') {
        let start = match start_s.parse::<u32>() {
            Ok(n) => n,
            Err(_) => return false,
        };
        let end = match end_s.parse::<u32>() {
            Ok(n) => n,
            Err(_) => return false,
        };
        start >= min && end <= max && start <= end
    } else if let Ok(num) = range_str.parse::<u32>() {
        num >= min && num <= max
    } else {
        false
    }
}

fn describe_cron(parts: &[&str]) -> String {
    let (m, h, dom, mon, dow) = (parts[0], parts[1], parts[2], parts[3], parts[4]);

    if m == "*" && h == "*" && dom == "*" && mon == "*" && dow == "*" {
        return "Every minute".to_string();
    }
    if m.starts_with("*/") && h == "*" && dom == "*" && mon == "*" && dow == "*" {
        return format!("Every {} minutes", &m[2..]);
    }
    if m == "0" && h == "*" && dom == "*" && mon == "*" && dow == "*" {
        return "Every hour at minute 0".to_string();
    }
    if m == "0" && h.starts_with("*/") && dom == "*" && mon == "*" && dow == "*" {
        return format!("Every {} hours at minute 0", &h[2..]);
    }
    if dom == "*" && mon == "*" && dow == "*" {
        return format!("At {}:{:0>2} daily", h, m);
    }
    format!(
        "At minute {}, hour {}, day-of-month {}, month {}, day-of-week {}",
        m, h, dom, mon, dow
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_standard_crons() {
        let v1 = validate_cron("*/5 * * * *");
        assert!(v1.is_valid);
        assert_eq!(v1.human_description, "Every 5 minutes");

        let v2 = validate_cron("0 0 * * *");
        assert!(v2.is_valid);
        assert_eq!(v2.human_description, "At 0:00 daily");

        let v3 = validate_cron("30 4 1 1 *");
        assert!(v3.is_valid);
    }

    #[test]
    fn test_invalid_cron_fields() {
        let v1 = validate_cron("61 * * * *");
        assert!(!v1.is_valid);
        assert!(v1.error.is_some());

        let v2 = validate_cron("* * * *");
        assert!(!v2.is_valid);
        assert_eq!(v2.fields.len(), 0);

        let v3 = validate_cron("0 25 * * *");
        assert!(!v3.is_valid);
    }
}
