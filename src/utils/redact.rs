/// Redact secrets from a string before logging or including in context packets.
///
/// Replaces common secret patterns with `[REDACTED]`:
/// - API keys (`sk-...`, `Bearer ...`, `api_key=...`, etc.)
/// - Tokens (`token=...`, `auth_token=...`, etc.)
/// - Passwords (`password=...`, `secret=...`, etc.)
/// - Private key blocks (BEGIN/END PEM blocks)
/// - URL credentials (`http://user:pass@host`)
/// - Env-style key=value lines for sensitive keys
pub fn redact_secrets(input: &str) -> String {
    let mut output = input.to_string();

    // Private key blocks
    output = redact_pem_blocks(&output);

    // URL credentials
    output = redact_url_credentials(&output);

    // Line-by-line for key=value secrets
    output = output
        .lines()
        .map(redact_line)
        .collect::<Vec<_>>()
        .join("\n");

    output
}

fn redact_pem_blocks(input: &str) -> String {
    let lines: Vec<&str> = input.lines().collect();
    let mut result = Vec::new();
    let mut in_block = false;

    for line in lines {
        if line.trim_start().starts_with("-----BEGIN ") && line.trim_end().ends_with("-----") {
            in_block = true;
            result.push("[REDACTED PEM BLOCK]");
            continue;
        }
        if line.trim_start().starts_with("-----END ") && line.trim_end().ends_with("-----") {
            in_block = false;
            continue;
        }
        if in_block {
            continue;
        }
        result.push(line);
    }

    result.join("\n")
}

fn redact_url_credentials(input: &str) -> String {
    let mut result = input.to_string();
    // Simple heuristic: find http:// or https:// followed by anything until @
    let schemes = ["http://", "https://"];
    for scheme in &schemes {
        let mut search_start = 0;
        while let Some(start) = result[search_start..].find(scheme) {
            let absolute_start = search_start + start;
            let after_scheme = absolute_start + scheme.len();
            if let Some(at_pos) = result[after_scheme..].find('@') {
                let absolute_at = after_scheme + at_pos;
                // Check there's a colon (user:pass) between scheme and @
                if result[after_scheme..absolute_at].contains(':') {
                    let host_start = absolute_at + 1;
                    let host_end = result[host_start..]
                        .find(['/', ' ', '\n'])
                        .map(|i| host_start + i)
                        .unwrap_or(result.len());
                    let host = result[host_start..host_end].to_string();
                    result.replace_range(absolute_start..host_end, &format!("{}[REDACTED]@{}", scheme, host));
                    search_start = absolute_start + scheme.len() + "[REDACTED]@".len() + host.len();
                    continue;
                }
            }
            search_start = after_scheme;
        }
    }
    result
}


fn redact_line(line: &str) -> String {
    let trimmed = line.trim_start();

    // Skip comments and empty lines
    if trimmed.is_empty() || trimmed.starts_with('#') {
        return line.to_string();
    }

    let lower = trimmed.to_lowercase();

    // Check for sensitive key prefixes
    let sensitive = [
        "api_key",
        "apikey",
        "api_secret",
        "apisecret",
        "auth_token",
        "authtoken",
        "access_token",
        "accesstoken",
        "token",
        "secret",
        "secret_key",
        "secretkey",
        "private_key",
        "privatekey",
        "password",
        "passwd",
        "bearer",
        "authorization",
    ];

    for key in &sensitive {
        if lower.starts_with(key) {
            // Match key=value, key: value, key = value, etc.
            if let Some(sep_pos) = find_separator(trimmed) {
                let key_part = &trimmed[..sep_pos];
                let _sep_and_rest = &trimmed[sep_pos..];
                // Find where value ends (space not in value? just take rest of line)
                return format!("{}=[REDACTED]", key_part.trim_end());
            }
        }
    }

    // Inline secrets in prose-like text
    let mut result = line.to_string();
    result = redact_inline_pattern(&result, "Bearer ", " ");
    result = redact_inline_pattern(&result, "sk-", " \n\t\"',;");
    result = redact_inline_pattern(&result, "sk_live_", " \n\t\"',;");
    result = redact_inline_pattern(&result, "sk_test_", " \n\t\"',;");
    result = redact_inline_pattern(&result, "ghp_", " \n\t\"',;");
    result = redact_inline_pattern(&result, "github_pat_", " \n\t\"',;");

    result
}

fn find_separator(line: &str) -> Option<usize> {
    for (i, c) in line.chars().enumerate() {
        if c == '=' || c == ':' {
            return Some(i);
        }
    }
    None
}

fn redact_inline_pattern(input: &str, prefix: &str, terminators: &str) -> String {
    let mut result = input.to_string();
    let mut search_start = 0;

    while let Some(start) = result[search_start..].find(prefix) {
        let absolute_start = search_start + start;
        let after_prefix = absolute_start + prefix.len();

        // Find end of token
        let end = result[after_prefix..]
            .find(|c: char| terminators.contains(c))
            .map(|i| after_prefix + i)
            .unwrap_or(result.len());

        // Only redact if token looks long enough to be a real secret
        if end - after_prefix >= 8 {
            result.replace_range(absolute_start..end, "[REDACTED]");
            search_start = absolute_start + "[REDACTED]".len();
        } else {
            search_start = after_prefix;
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redact_api_key_inline() {
        let input = "The API key is sk-abc123def456 and it is secret.";
        let out = redact_secrets(input);
        assert!(!out.contains("sk-abc123def456"));
        assert!(out.contains("[REDACTED]"));
    }

    #[test]
    fn test_redact_bearer_token() {
        let input = "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9";
        let out = redact_secrets(input);
        assert!(!out.contains("eyJhbGciOi"));
        assert!(out.contains("[REDACTED]"));
    }

    #[test]
    fn test_redact_env_api_key() {
        let input = "API_KEY=super_secret_value_123\nOTHER=safe";
        let out = redact_secrets(input);
        assert!(out.contains("API_KEY=[REDACTED]"));
        assert!(out.contains("OTHER=safe"));
    }

    #[test]
    fn test_redact_password() {
        let input = "password=myP@ssw0rd\nuser=alice";
        let out = redact_secrets(input);
        assert!(out.contains("password=[REDACTED]"));
        assert!(out.contains("user=alice"));
    }

    #[test]
    fn test_redact_private_key_block() {
        let input =
            "-----BEGIN RSA PRIVATE KEY-----\nMIIEpAIBAAKCAQEA...\n-----END RSA PRIVATE KEY-----";
        let out = redact_secrets(input);
        assert!(!out.contains("MIIEpAIBAAKCAQEA"));
        assert!(out.contains("[REDACTED PEM BLOCK]"));
    }

    #[test]
    fn test_redact_url_credentials() {
        let input = "Database URL: https://admin:secret123@db.example.com:5432/data";
        let out = redact_secrets(input);
        assert!(!out.contains("admin:secret123"));
        assert!(out.contains("https://[REDACTED]@db.example.com:5432/data"));
    }

    #[test]
    fn test_redact_github_token() {
        let input = "ghp_xxxxxxxxxxxxxxxxxxxx\ngithub_pat_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx";
        let out = redact_secrets(input);
        assert!(!out.contains("ghp_xxx"));
        assert!(!out.contains("github_pat_xxx"));
        assert!(out.contains("[REDACTED]"));
    }

    #[test]
    fn test_no_false_positives_short_prefix() {
        let input = "sk-12 short";
        let out = redact_secrets(input);
        // Too short to redact safely
        assert!(out.contains("sk-12"));
    }
}
