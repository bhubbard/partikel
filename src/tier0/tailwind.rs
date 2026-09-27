use regex::Regex;
use std::sync::OnceLock;

static TAILWIND_PATTERNS: OnceLock<Vec<Regex>> = OnceLock::new();

fn get_patterns() -> &'static Vec<Regex> {
    TAILWIND_PATTERNS.get_or_init(|| {
        vec![
            // Layout / Display
            Regex::new(
                r"^(block|inline-block|inline|flex|inline-flex|grid|inline-grid|hidden|table)$",
            )
            .unwrap(),
            Regex::new(r"^(relative|absolute|fixed|sticky|static)$").unwrap(),
            Regex::new(r"^(top|bottom|left|right|inset)-").unwrap(),
            Regex::new(r"^z-").unwrap(),
            // Flex / Grid
            Regex::new(r"^flex-").unwrap(),
            Regex::new(r"^grid-").unwrap(),
            Regex::new(r"^items-").unwrap(),
            Regex::new(r"^justify-").unwrap(),
            Regex::new(r"^gap-").unwrap(),
            // Box Sizing
            Regex::new(r"^w-").unwrap(),
            Regex::new(r"^min-w-").unwrap(),
            Regex::new(r"^max-w-").unwrap(),
            Regex::new(r"^h-").unwrap(),
            Regex::new(r"^min-h-").unwrap(),
            Regex::new(r"^max-h-").unwrap(),
            // Spacing
            Regex::new(r"^m[trblxy]?-").unwrap(),
            Regex::new(r"^p[trblxy]?-").unwrap(),
            // Typography
            Regex::new(r"^font-").unwrap(),
            Regex::new(r"^text-").unwrap(),
            Regex::new(r"^leading-").unwrap(),
            Regex::new(r"^tracking-").unwrap(),
            // Background & Borders
            Regex::new(r"^bg-").unwrap(),
            Regex::new(r"^border").unwrap(),
            Regex::new(r"^rounded").unwrap(),
            Regex::new(r"^shadow").unwrap(),
            // Interactivity
            Regex::new(r"^cursor-").unwrap(),
            Regex::new(r"^hover:").unwrap(),
            Regex::new(r"^focus:").unwrap(),
            Regex::new(r"^active:").unwrap(),
            Regex::new(r"^transition").unwrap(),
            Regex::new(r"^transform").unwrap(),
        ]
    })
}

pub fn tailwind_class_sort(class_string: &str) -> String {
    let patterns = get_patterns();
    let mut classes: Vec<&str> = class_string.split_whitespace().collect();

    let get_rank = |cls: &str| -> usize {
        for (idx, pat) in patterns.iter().enumerate() {
            if pat.is_match(cls) {
                return idx;
            }
        }
        patterns.len()
    };

    classes.sort_by_key(|&c| get_rank(c));
    classes.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tailwind_sort() {
        let input = "rounded bg-blue-500 flex p-4 text-white hover:bg-blue-600";
        let sorted = tailwind_class_sort(input);
        assert_eq!(
            sorted,
            "flex p-4 text-white bg-blue-500 rounded hover:bg-blue-600"
        );
    }
}
