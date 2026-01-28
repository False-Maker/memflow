pub fn redact_secrets(input: &str) -> String {
    let mut s = input.to_string();
    s = redact_bearer_tokens(&s);
    s = redact_incorrect_api_key(&s);
    s = redact_sk_like_tokens(&s);
    s = redact_json_key_fields(&s);
    s
}

fn redact_bearer_tokens(input: &str) -> String {
    redact_after_prefix(input, "Bearer ", |c| c.is_whitespace() || c == '"' || c == '\'' || c == '\r' || c == '\n')
}

fn redact_incorrect_api_key(input: &str) -> String {
    redact_after_prefix(input, "Incorrect API key provided:", |c| c.is_whitespace() || c == '"' || c == '\'' || c == '\r' || c == '\n' || c == '.')
}

fn redact_sk_like_tokens(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if i + 3 <= chars.len() && chars[i] == 's' && chars[i + 1] == 'k' && chars[i + 2] == '-' {
            out.push('s');
            out.push('k');
            out.push('-');
            out.push_str("[REDACTED]");
            i += 3;
            while i < chars.len() {
                let c = chars[i];
                if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                    i += 1;
                    continue;
                }
                break;
            }
            continue;
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}

fn redact_json_key_fields(input: &str) -> String {
    let mut s = input.to_string();
    s = redact_json_string_field(&s, "\"api_key\"");
    s = redact_json_string_field(&s, "\"apiKey\"");
    s
}

fn redact_json_string_field(input: &str, key_literal: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut rest = input;
    while let Some(pos) = rest.find(key_literal) {
        out.push_str(&rest[..pos + key_literal.len()]);
        rest = &rest[pos + key_literal.len()..];
        let colon = match rest.find(':') {
            Some(v) => v,
            None => {
                out.push_str(rest);
                return out;
            }
        };
        out.push_str(&rest[..colon + 1]);
        rest = &rest[colon + 1..];
        let trimmed = rest.trim_start();
        let skipped = rest.len() - trimmed.len();
        out.push_str(&rest[..skipped]);
        rest = trimmed;
        if !rest.starts_with('"') {
            continue;
        }
        out.push('"');
        rest = &rest[1..];
        if let Some(end) = rest.find('"') {
            out.push_str("[REDACTED]");
            out.push('"');
            rest = &rest[end + 1..];
        } else {
            out.push_str("[REDACTED]");
            return out;
        }
    }
    out.push_str(rest);
    out
}

fn redact_after_prefix<F>(input: &str, prefix: &str, is_end: F) -> String
where
    F: Fn(char) -> bool,
{
    let mut out = String::with_capacity(input.len());
    let mut i = 0;
    while let Some(pos) = input[i..].find(prefix) {
        let start = i + pos;
        out.push_str(&input[i..start]);
        out.push_str(prefix);
        let mut j = start + prefix.len();
        while j < input.len() {
            let c = input[j..].chars().next().unwrap();
            if c.is_whitespace() {
                out.push(c);
                j += c.len_utf8();
                continue;
            }
            break;
        }
        if j >= input.len() {
            i = j;
            break;
        }
        let mut k = j;
        while k < input.len() {
            let c = input[k..].chars().next().unwrap();
            if is_end(c) {
                break;
            }
            k += c.len_utf8();
        }
        out.push_str("[REDACTED]");
        out.push_str(&input[k..]);
        return out;
    }
    out.push_str(&input[i..]);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redacts_incorrect_api_key_message() {
        let input = r#"Incorrect API key provided: sk-abc123DEF456. You can find your API key"#;
        let out = redact_secrets(input);
        assert!(!out.contains("sk-abc123"));
        assert!(out.contains("[REDACTED]"));
    }

    #[test]
    fn redacts_bearer_token() {
        let input = r#"Authorization: Bearer sk-aaaabbbbccccdddd"#;
        let out = redact_secrets(input);
        assert!(!out.contains("sk-aaaabbbb"));
        assert!(out.contains("Bearer [REDACTED]"));
    }
}
