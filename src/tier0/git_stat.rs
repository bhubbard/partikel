use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitDiffStat {
    pub files_changed: usize,
    pub insertions: usize,
    pub deletions: usize,
    pub files: Vec<GitFileStat>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitFileStat {
    pub path: String,
    pub insertions: usize,
    pub deletions: usize,
}

pub fn parse_git_diff_stat(stat_output: &str) -> GitDiffStat {
    let mut files = Vec::new();
    let mut total_files = 0;
    let mut total_ins = 0;
    let mut total_del = 0;

    for line in stat_output.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        // Summary line: e.g. "3 files changed, 24 insertions(+), 8 deletions(-)"
        if line.contains("changed") || line.contains("insertion") || line.contains("deletion") {
            let parts: Vec<&str> = line.split(',').collect();
            for part in parts {
                let p = part.trim();
                if p.contains("file") {
                    if let Some(num_str) = p.split_whitespace().next() {
                        total_files = num_str.parse().unwrap_or(0);
                    }
                } else if p.contains("insertion") {
                    if let Some(num_str) = p.split_whitespace().next() {
                        total_ins = num_str.parse().unwrap_or(0);
                    }
                } else if p.contains("deletion") {
                    if let Some(num_str) = p.split_whitespace().next() {
                        total_del = num_str.parse().unwrap_or(0);
                    }
                }
            }
            continue;
        }

        // File line: "src/main.rs | 10 ++---"
        if let Some((path_part, change_part)) = line.split_once('|') {
            let path = path_part.trim().to_string();
            let change_str = change_part.trim();

            let mut ins = 0;
            let mut del = 0;
            if let Some(num_str) = change_str.split_whitespace().next() {
                if let Ok(total_count) = num_str.parse::<usize>() {
                    let pluses = change_str.chars().filter(|&c| c == '+').count();
                    let minuses = change_str.chars().filter(|&c| c == '-').count();
                    let plus_minus = pluses + minuses;
                    if let Some(res) = (total_count * pluses).checked_div(plus_minus) {
                        ins = res;
                        del = total_count.saturating_sub(ins);
                    } else {
                        ins = total_count;
                    }
                }
            }

            files.push(GitFileStat {
                path,
                insertions: ins,
                deletions: del,
            });
        }
    }

    if total_files == 0 && !files.is_empty() {
        total_files = files.len();
        total_ins = files.iter().map(|f| f.insertions).sum();
        total_del = files.iter().map(|f| f.deletions).sum();
    }

    GitDiffStat {
        files_changed: total_files,
        insertions: total_ins,
        deletions: total_del,
        files,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_git_diff_stat() {
        let raw = r#"
src/main.rs | 10 +++++-----
src/lib.rs  |  4 ++++
2 files changed, 9 insertions(+), 5 deletions(-)
"#;
        let stat = parse_git_diff_stat(raw);
        assert_eq!(stat.files_changed, 2);
        assert_eq!(stat.insertions, 9);
        assert_eq!(stat.deletions, 5);
        assert_eq!(stat.files.len(), 2);
    }
}
