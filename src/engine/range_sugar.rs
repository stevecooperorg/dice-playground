//! Desugar inclusive `..` range syntax into `through` / `at_most` / `at_least` calls.

use anyhow::Result;

/// Rewrite `6..94`, `..6`, and `96..` into IntBand constructors (see `docs/references/stdlib.md`).
pub fn desugar_ranges(source: &str) -> Result<String> {
    let mut out = String::with_capacity(source.len());
    let mut i = 0usize;
    let bytes = source.as_bytes();
    let mut in_string = None::<char>;

    while i < source.len() {
        let Some(ch) = source[i..].chars().next() else {
            break;
        };
        let ch_len = ch.len_utf8();

        if in_string.is_some() {
            out.push(ch);
            if Some(ch) == in_string && !is_escaped(&source[..i]) {
                in_string = None;
            }
            i += ch_len;
            continue;
        }

        if ch == '"' || ch == '\'' {
            in_string = Some(ch);
            out.push(ch);
            i += ch_len;
            continue;
        }

        if ch.is_ascii_digit() {
            let mut digit_end = i + ch_len;
            while digit_end < source.len() && source.as_bytes()[digit_end].is_ascii_digit() {
                digit_end += 1;
            }
            if source[digit_end..].starts_with("..") {
                if let Ok(lo) = source[i..digit_end].parse::<i32>() {
                    if let Some((replacement, len)) =
                        try_parse_range_with_lo(&source[i..], digit_end - i, lo)
                    {
                        out.push_str(&replacement);
                        i += len;
                        continue;
                    }
                }
            }
        }

        if bytes.get(i) == Some(&b'.') && bytes.get(i + 1) == Some(&b'.') {
            if let Some((replacement, len)) = try_parse_range_at(&source[i..]) {
                out.push_str(&replacement);
                i += len;
                continue;
            }
        }

        out.push(ch);
        i += ch_len;
    }
    Ok(out)
}

fn is_escaped(prefix: &str) -> bool {
    let mut slashes = 0usize;
    for c in prefix.chars().rev() {
        if c == '\\' {
            slashes += 1;
        } else {
            break;
        }
    }
    slashes % 2 == 1
}

fn try_parse_range_at(rest: &str) -> Option<(String, usize)> {
    if !rest.starts_with("..") {
        return None;
    }
    let mut pos = 2usize;

    if let Some(hi) = parse_optional_digits(rest, &mut pos) {
        // `..hi`
        if !range_tail_ok(rest, pos) {
            return None;
        }
        return Some((format!("at_most({hi})"), pos));
    }

    None
}

fn try_parse_range_with_lo(rest: &str, lo_byte_len: usize, lo: i32) -> Option<(String, usize)> {
    if !rest[lo_byte_len..].starts_with("..") {
        return None;
    }
    let mut pos = lo_byte_len + 2;
    if let Some(hi) = parse_optional_digits(rest, &mut pos) {
        if !range_tail_ok(rest, pos) {
            return None;
        }
        return Some((format!("through({lo}, {hi})"), pos));
    }
    if !range_tail_ok(rest, pos) {
        return None;
    }
    Some((format!("at_least({lo})"), pos))
}

/// Scan for `digits..` or leading `..digits` at `i` (syntax highlighting).
#[allow(dead_code)]
pub fn try_range_literal_len_at(source: &str, start: usize) -> Option<usize> {
    let rest = &source[start..];
    if rest.starts_with("..") {
        let mut pos = 2;
        if parse_optional_digits(rest, &mut pos).is_some() && range_tail_ok(rest, pos) {
            return Some(pos);
        }
        return None;
    }
    let mut pos = 0usize;
    let lo = parse_digits_run(rest, &mut pos)?;
    let lo_len = pos;
    if !rest[pos..].starts_with("..") {
        return None;
    }
    try_parse_range_with_lo(rest, lo_len, lo).map(|(_, len)| len)
}

fn range_tail_ok(rest: &str, pos: usize) -> bool {
    let tail = rest[pos..].trim_start();
    if tail.is_empty() {
        return true;
    }
    matches!(
        tail.chars().next(),
        Some(',' | ')' | ']' | '}' | ';' | '\n' | '\r')
    )
}

fn parse_optional_digits(s: &str, pos: &mut usize) -> Option<i32> {
    let start = *pos;
    while *pos < s.len() && s.as_bytes()[*pos].is_ascii_digit() {
        *pos += 1;
    }
    if *pos == start {
        return None;
    }
    s[start..*pos].parse().ok()
}

fn parse_digits_run(s: &str, pos: &mut usize) -> Option<i32> {
    let start = *pos;
    if start >= s.len() || !s.as_bytes()[start].is_ascii_digit() {
        return None;
    }
    while *pos < s.len() && s.as_bytes()[*pos].is_ascii_digit() {
        *pos += 1;
    }
    s[start..*pos].parse().ok()
}

/// Desugar dice literals and inclusive ranges.
pub fn desugar_all(path: &str, source: &str) -> Result<String> {
    let with_ranges = desugar_ranges(source)?;
    super::sugar::desugar(path, &with_ranges)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn through_range() {
        let out = desugar_ranges("x.in_band(6..94)").unwrap();
        assert_eq!(out, "x.in_band(through(6, 94))");
    }

    #[test]
    fn at_most_and_at_least() {
        assert_eq!(
            desugar_ranges("bucket(r, s, ..6, 7..9, 10..)").unwrap(),
            "bucket(r, s, at_most(6), through(7, 9), at_least(10))"
        );
    }

    #[test]
    fn does_not_touch_dice_notation() {
        let out = desugar_ranges("2d6 + 3d6").unwrap();
        assert_eq!(out, "2d6 + 3d6");
    }

    #[test]
    fn does_not_desugar_ambiguous_2_dot_d6() {
        let out = desugar_ranges("2..d6").unwrap();
        assert_eq!(out, "2..d6");
    }

    #[test]
    fn respects_strings() {
        let out = desugar_ranges(r#""6..94""#).unwrap();
        assert_eq!(out, r#""6..94""#);
    }
}
