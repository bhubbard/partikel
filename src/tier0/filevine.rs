use regex::Regex;
use std::sync::LazyLock;

static NON_ALPHANUMERIC_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"[^a-zA-Z0-9\s]").unwrap()
});

/// Converts human labels into canonical camelCase Filevine custom field codes.
/// 1. Strips non-alphanumeric chars.
/// 2. Converts to camelCase.
/// 3. Truncates to max 40 characters.
pub fn filevine_custom_field_sanitize(label: &str) -> String {
    let cleaned = NON_ALPHANUMERIC_RE.replace_all(label, "");
    let words: Vec<&str> = cleaned.split_whitespace().collect();

    if words.is_empty() {
        return String::new();
    }

    let mut result = words[0].to_lowercase();
    for word in &words[1..] {
        let mut chars = word.chars();
        if let Some(first) = chars.next() {
            result.push_str(&first.to_uppercase().collect::<String>());
            result.push_str(&chars.as_str().to_lowercase());
        }
    }

    if result.len() > 40 {
        result.truncate(40);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filevine_sanitize() {
        assert_eq!(
            filevine_custom_field_sanitize("Client's Primary Insurance Policy #"),
            "clientsPrimaryInsurancePolicy"
        );
        assert_eq!(
            filevine_custom_field_sanitize("Estimated Property Damage ($)"),
            "estimatedPropertyDamage"
        );
    }
}
