//! Contract between static docs and the playground for \"load script\" navigation.

use serde::{Deserialize, Serialize};

/// Query parameter on `/` carrying a percent-encoded JSON payload.
pub const LOAD_QUERY_PARAM: &str = "dice_playground_load";

/// `localStorage` fallback when the encoded URL would be too long (see `MAX_URL_ENCODED_LEN`).
pub const PENDING_LOCAL_STORAGE_KEY: &str = "dice_playground_pending_load";

/// Match `encodeURIComponent` length guard in prior client script.
pub const MAX_URL_ENCODED_LEN: usize = 7000;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PendingScriptLoad {
    pub content: String,
    pub filename: Option<String>,
}

/// Whether a fenced code block in tutorial/cookbook HTML should get a playground link.
pub fn looks_like_dice_script(text: &str) -> bool {
    let t = text.trim();
    if t.is_empty() {
        return false;
    }
    if t.starts_with("Write a .dice script") {
        return false;
    }
    t.contains("output(")
        || t.contains("dice_pool")
        || contains_ndm_notation(t)
        || t.contains(".p_ge")
        || t.contains(".p_mf")
        || t.contains(".p_at_least")
        || t.contains("bucket(")
        || t.contains("def ")
}

fn contains_ndm_notation(text: &str) -> bool {
    text.as_bytes()
        .windows(3)
        .any(|w| w[0].is_ascii_digit() && w[1] == b'd' && w[2].is_ascii_digit())
}

/// Percent-encode like JavaScript `encodeURIComponent` (UTF-8 bytes).
pub fn encode_uri_component(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for &byte in s.as_bytes() {
        match byte {
            b'A'..=b'Z'
            | b'a'..=b'z'
            | b'0'..=b'9'
            | b'-'
            | b'_'
            | b'.'
            | b'!'
            | b'~'
            | b'*'
            | b'\''
            | b'('
            | b')' => out.push(byte as char),
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}

pub fn payload_json(load: &PendingScriptLoad) -> anyhow::Result<String> {
    Ok(serde_json::to_string(load)?)
}

pub fn decode_payload_json(raw: &str) -> Option<PendingScriptLoad> {
    serde_json::from_str(raw).ok()
}

/// Path and query for opening the playground with script inline (no `localStorage` needed).
pub fn playground_href(load: &PendingScriptLoad) -> anyhow::Result<String> {
    let json = payload_json(load)?;
    let encoded = encode_uri_component(&json);
    Ok(format!("/?{LOAD_QUERY_PARAM}={encoded}"))
}

/// Prefer URL when the encoded query fits in [`MAX_URL_ENCODED_LEN`]; otherwise caller should
/// write [`payload_json`] to [`PENDING_LOCAL_STORAGE_KEY`] and open `/`.
pub fn playground_open_href(content: &str, filename: Option<String>) -> anyhow::Result<String> {
    let load = PendingScriptLoad {
        content: content.to_owned(),
        filename,
    };
    let json = payload_json(&load)?;
    let encoded = encode_uri_component(&json);
    if encoded.len() <= MAX_URL_ENCODED_LEN {
        Ok(format!("/?{LOAD_QUERY_PARAM}={encoded}"))
    } else {
        Ok("/".to_owned())
    }
}

pub fn needs_local_storage_fallback(content: &str, filename: Option<String>) -> bool {
    let load = PendingScriptLoad {
        content: content.to_owned(),
        filename,
    };
    payload_json(&load)
        .ok()
        .map(|json| encode_uri_component(&json).len() > MAX_URL_ENCODED_LEN)
        .unwrap_or(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_payload_json() {
        let load = PendingScriptLoad {
            content: "output(\"x\", 1d6)\n".to_owned(),
            filename: None,
        };
        let json = payload_json(&load).expect("json");
        let back = decode_payload_json(&json).expect("decode");
        assert_eq!(back, load);
    }

    #[test]
    fn looks_like_dice_script_accepts_lesson_snippet() {
        assert!(looks_like_dice_script("output(\"one_d6\", 1d6)"));
    }

    #[test]
    fn looks_like_dice_script_rejects_llm_example_prompt() {
        assert!(!looks_like_dice_script(
            "Write a .dice script for this check:\n- Roll d6"
        ));
    }

    #[test]
    fn href_contains_query_param() {
        let href = playground_open_href("output(\"x\", 1d6)", None).expect("href");
        assert!(href.starts_with("/?dice_playground_load="));
    }
}
