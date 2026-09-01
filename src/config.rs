//! Reads an optional config file that adjusts the line length limit and
//! turns off individual rules. The format is deliberately not TOML or
//! YAML (no third-party parser is worth pulling in for two settings):
//! blank lines and `#` comments are ignored, everything else has to be
//! `key = value`.

use std::collections::HashSet;
use std::fs;
use std::path::Path;

const CONFIG_FILE_NAME: &str = ".git-history-lint.conf";
const DEFAULT_MAX_LINE_LEN: usize = 72;

#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    pub max_line_len: usize,
    pub disabled_rules: HashSet<String>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            max_line_len: DEFAULT_MAX_LINE_LEN,
            disabled_rules: HashSet::new(),
        }
    }
}

/// Loads config from `.git-history-lint.conf` in the current directory.
/// A missing file is not an error, it just means the defaults apply.
pub fn load() -> Result<Config, String> {
    let path = Path::new(CONFIG_FILE_NAME);
    if !path.exists() {
        return Ok(Config::default());
    }
    let text =
        fs::read_to_string(path).map_err(|e| format!("{}: {}", CONFIG_FILE_NAME, e))?;
    parse(&text).map_err(|e| format!("{}: {}", CONFIG_FILE_NAME, e))
}

fn parse(text: &str) -> Result<Config, String> {
    let mut config = Config::default();
    for (i, raw_line) in text.lines().enumerate() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (key, value) = line
            .split_once('=')
            .ok_or_else(|| format!("line {}: expected `key = value`, got {:?}", i + 1, raw_line))?;
        let key = key.trim();
        let value = value.trim();
        match key {
            "max-line-len" => {
                config.max_line_len = value.parse::<usize>().map_err(|_| {
                    format!(
                        "line {}: max-line-len must be a positive integer, got {:?}",
                        i + 1,
                        value
                    )
                })?;
            }
            "disable" => {
                if value.is_empty() {
                    return Err(format!("line {}: disable needs a rule id", i + 1));
                }
                config.disabled_rules.insert(value.to_string());
            }
            other => {
                return Err(format!("line {}: unknown config key {:?}", i + 1, other));
            }
        }
    }
    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_input_gives_defaults() {
        assert_eq!(parse("").unwrap(), Config::default());
    }

    #[test]
    fn comments_and_blank_lines_are_ignored() {
        let text = "# a comment\n\n   \nmax-line-len = 100\n";
        let config = parse(text).unwrap();
        assert_eq!(config.max_line_len, 100);
    }

    #[test]
    fn disable_can_repeat_for_multiple_rules() {
        let text = "disable = subject-trailing-period\ndisable = trailing-whitespace\n";
        let config = parse(text).unwrap();
        assert_eq!(config.disabled_rules.len(), 2);
        assert!(config.disabled_rules.contains("subject-trailing-period"));
        assert!(config.disabled_rules.contains("trailing-whitespace"));
    }

    #[test]
    fn non_numeric_max_line_len_is_an_error() {
        assert!(parse("max-line-len = seventy").is_err());
    }

    #[test]
    fn unknown_key_is_an_error() {
        assert!(parse("max-lien-len = 80").is_err());
    }

    #[test]
    fn line_without_equals_is_an_error() {
        assert!(parse("max-line-len 80").is_err());
    }

    #[test]
    fn empty_disable_value_is_an_error() {
        assert!(parse("disable = ").is_err());
    }

    #[test]
    fn whitespace_around_key_and_value_is_trimmed() {
        let config = parse("  max-line-len   =   90  ").unwrap();
        assert_eq!(config.max_line_len, 90);
    }
}
