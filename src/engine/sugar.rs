//! Desugars tabletop dice literals into Starlark expressions.

use anyhow::Context;

/// If `source` contains dice sugar, expand it; otherwise return `source` unchanged.
pub fn desugar_if_needed(path: &str, source: &str) -> anyhow::Result<String> {
    desugar(path, source)
}

fn next_char(rest: &str) -> Option<(char, usize)> {
    let ch = rest.chars().next()?;
    Some((ch, ch.len_utf8()))
}

/// Byte length of a dice literal at the start of `rest`, if any (for syntax highlighting).
pub fn dice_literal_len_at(rest: &str, prev: Option<char>) -> Option<usize> {
    try_parse_dice_expr(rest, prev).map(|(_, len)| len)
}

/// Replace tabletop dice literals (`4d6`, `4d6dl1`, `4d6kh2`, …) with Starlark stdlib calls.
pub fn desugar(_path: &str, source: &str) -> anyhow::Result<String> {
    let mut out = String::with_capacity(source.len());
    let mut i = 0usize;
    while i < source.len() {
        let prev = source.get(..i).and_then(|s| s.chars().next_back());
        if let Some((expr, len)) = try_parse_dice_expr(&source[i..], prev) {
            out.push_str(&expr);
            i += len;
        } else {
            let Some((ch, ch_len)) = next_char(&source[i..]) else {
                break;
            };
            out.push(ch);
            i += ch_len;
        }
    }
    Ok(out)
}

fn try_parse_dice_expr(rest: &str, prev: Option<char>) -> Option<(String, usize)> {
    let mut pos = 0usize;
    let count = match parse_digits(rest, &mut pos) {
        Some(c) => c,
        None => {
            if let Some(p) = prev {
                if p.is_ascii_alphanumeric() || p == '_' {
                    return None;
                }
            }
            1
        }
    };
    if rest.as_bytes().get(pos)? != &b'd' && rest.as_bytes().get(pos)? != &b'D' {
        return None;
    }
    pos += 1;
    let sides = parse_digits(rest, &mut pos)?;
    let expanded = if let Some((op, n)) = parse_pool_suffix(rest, &mut pos) {
        match op {
            PoolSuffix::DropLowest => format!("drop_lowest({count}, {sides}, {n})"),
            PoolSuffix::DropHighest => format!("drop_highest({count}, {sides}, {n})"),
            PoolSuffix::KeepHighest => format!("keep_highest({count}, {sides}, {n})"),
            PoolSuffix::KeepLowest => format!("keep_lowest({count}, {sides}, {n})"),
        }
    } else if count == 1 {
        format!("d({sides})")
    } else {
        format!("roll_pool({count}, {sides})")
    };
    let needs_sum = |tail: &str| -> bool {
        let tail = tail.trim_start();
        if tail.is_empty() {
            return false;
        }
        matches!(
            tail.chars().next(),
            Some('+' | '-' | '*' | '/' | ')' | ',' | ']' | '>' | '<' | '=')
        )
    };
    let expanded = if expanded.starts_with("roll_pool(") && needs_sum(&rest[pos..]) {
        format!("sum({expanded})")
    } else {
        expanded
    };
    Some((expanded, pos))
}

enum PoolSuffix {
    DropLowest,
    DropHighest,
    KeepHighest,
    KeepLowest,
}

fn parse_pool_suffix(rest: &str, pos: &mut usize) -> Option<(PoolSuffix, i32)> {
    let tail = &rest[*pos..];
    let (op, skip): (PoolSuffix, usize) = if tail.starts_with("dl") || tail.starts_with("DL") {
        (PoolSuffix::DropLowest, 2)
    } else if tail.starts_with("dh") || tail.starts_with("DH") {
        (PoolSuffix::DropHighest, 2)
    } else if tail.starts_with("kh") || tail.starts_with("KH") {
        (PoolSuffix::KeepHighest, 2)
    } else if tail.starts_with("kl") || tail.starts_with("KL") {
        (PoolSuffix::KeepLowest, 2)
    } else {
        return None;
    };
    *pos += skip;
    let n = parse_digits(rest, pos)?;
    if n <= 0 {
        return None;
    }
    Some((op, n))
}

fn parse_digits(s: &str, pos: &mut usize) -> Option<i32> {
    let start = *pos;
    while *pos < s.len() && s.as_bytes()[*pos].is_ascii_digit() {
        *pos += 1;
    }
    if *pos == start {
        return None;
    }
    s[start..*pos].parse().context("digit run").ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn desugar_4d6dl1_in_output() {
        let src = r#"output("x", 4d6dl1)"#;
        let out = desugar("t.star", src).unwrap();
        assert!(out.contains("drop_lowest(4, 6, 1)"));
    }

    #[test]
    fn desugar_2d10() {
        let out = desugar("t.star", "2d10").unwrap();
        assert_eq!(out, "roll_pool(2, 10)");
    }

    #[test]
    fn desugar_2d10_plus_auto_sum() {
        let out = desugar("t.star", "2d10 + 3").unwrap();
        assert_eq!(out, "sum(roll_pool(2, 10)) + 3");
    }

    #[test]
    fn desugar_8d6_times_ten_auto_sum() {
        let out = desugar("t.star", "8d6 * 10").unwrap();
        assert_eq!(out, "sum(roll_pool(8, 6)) * 10");
    }

    #[test]
    fn desugar_8d6_floor_div_two_auto_sum() {
        let out = desugar("t.star", "8d6 // 2").unwrap();
        assert_eq!(out, "sum(roll_pool(8, 6)) // 2");
    }

    #[test]
    fn desugar_1d4_times_ten() {
        let out = desugar("t.star", "1d4 * 10").unwrap();
        assert_eq!(out, "d(4) * 10");
    }

    #[test]
    fn desugar_output_wraps_pool_sum() {
        let out = desugar("t.star", r#"output("x", 2d6)"#).unwrap();
        assert!(out.contains("sum(roll_pool(2, 6))"));
    }

    #[test]
    fn desugar_pool_suffixes() {
        assert!(desugar("t.star", "4d6dh1")
            .unwrap()
            .contains("drop_highest(4, 6, 1)"));
        assert!(desugar("t.star", "4d6kh2")
            .unwrap()
            .contains("keep_highest(4, 6, 2)"));
        assert!(desugar("t.star", "3d12kl1")
            .unwrap()
            .contains("keep_lowest(3, 12, 1)"));
    }

    #[test]
    fn invalid_dl_without_digits_fails_parse_as_text() {
        let out = desugar("t.star", "xd6").unwrap();
        assert_eq!(out, "xd6");
    }

    #[test]
    fn does_not_desugar_inside_identifier() {
        let out = desugar("t.star", r#"output("two_d6", d(6))"#).unwrap();
        assert!(out.contains(r#""two_d6""#));
    }
}
