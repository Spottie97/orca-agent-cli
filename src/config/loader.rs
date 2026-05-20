use std::env;
use std::path::Path;

use anyhow::{Context, Result};

use super::schema::Config;

pub fn load(path: &Path) -> Result<Config> {
    let raw = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read config file: {}", path.display()))?;
    let substituted = substitute_env_vars(&raw)?;
    let config: Config = serde_yaml::from_str(&substituted)
        .with_context(|| format!("Failed to parse config file: {}", path.display()))?;
    Ok(config)
}

fn substitute_env_vars(input: &str) -> Result<String> {
    let mut result = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '$' && chars.peek() == Some(&'{') {
            chars.next(); // consume '{'
            let mut var_name = String::new();
            let mut default_value = None;

            while let Some(&ch) = chars.peek() {
                if ch == '}' {
                    chars.next(); // consume '}'
                    break;
                } else if ch == ':' && chars.clone().nth(1) == Some('-') {
                    chars.next(); // consume ':'
                    chars.next(); // consume '-'
                    let mut default = String::new();
                    while let Some(&ch2) = chars.peek() {
                        if ch2 == '}' {
                            break;
                        }
                        default.push(ch2);
                        chars.next();
                    }
                    default_value = Some(default);
                } else {
                    var_name.push(ch);
                    chars.next();
                }
            }

            let value = if let Some(default) = default_value {
                env::var(&var_name).unwrap_or(default)
            } else {
                env::var(&var_name)
                    .with_context(|| format!("Environment variable '{}' not found", var_name))?
            };
            result.push_str(&value);
        } else {
            result.push(c);
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_substitute_simple() {
        env::set_var("TEST_VAR", "hello");
        let result = substitute_env_vars("${TEST_VAR}").unwrap();
        assert_eq!(result, "hello");
    }

    #[test]
    fn test_substitute_with_default() {
        env::remove_var("MISSING_VAR");
        let result = substitute_env_vars("${MISSING_VAR:-default_value}").unwrap();
        assert_eq!(result, "default_value");
    }

    #[test]
    fn test_substitute_missing_without_default() {
        env::remove_var("DEFINITELY_MISSING_VAR_12345");
        let result = substitute_env_vars("${DEFINITELY_MISSING_VAR_12345}");
        assert!(result.is_err());
    }
}
