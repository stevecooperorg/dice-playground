//! Small, allocation-free recognizers for the two extensions to Starlark.
//! Native token boundaries are checked by `lex`; these functions only inspect spelling.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Literal {
    Dice(DiceLiteral),
    Band { lo: Option<i32>, hi: Option<i32> },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DiceLiteral {
    pub count: i32,
    pub sides: i32,
    pub suffix: Option<(PoolSuffix, i32)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PoolSuffix {
    DropLowest,
    DropHighest,
    KeepHighest,
    KeepLowest,
}

fn word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

fn start_boundary(prev: Option<char>) -> bool {
    !prev.is_some_and(|c| word_char(c) || c == '.')
}

pub(crate) fn dice_at(rest: &str, prev: Option<char>) -> Option<(Literal, usize)> {
    if !start_boundary(prev) {
        return None;
    }
    let mut pos = 0;
    let count = if rest.as_bytes().first()?.is_ascii_digit() {
        digits(rest, &mut pos)?
    } else {
        1
    };
    if !matches!(rest.as_bytes().get(pos), Some(b'd' | b'D')) {
        return None;
    }
    pos += 1;
    let sides = digits(rest, &mut pos)?;
    let suffix = match rest.get(pos..pos + 2) {
        Some("dl" | "DL") => Some(PoolSuffix::DropLowest),
        Some("dh" | "DH") => Some(PoolSuffix::DropHighest),
        Some("kh" | "KH") => Some(PoolSuffix::KeepHighest),
        Some("kl" | "KL") => Some(PoolSuffix::KeepLowest),
        _ => None,
    };
    let suffix = if let Some(op) = suffix {
        pos += 2;
        let n = digits(rest, &mut pos)?;
        if n == 0 {
            return None;
        }
        Some((op, n))
    } else {
        None
    };
    if rest[pos..].chars().next().is_some_and(word_char)
        || rest[pos..].starts_with("..")
        || rest[pos..]
            .strip_prefix('.')
            .is_some_and(|s| s.starts_with(|c: char| c.is_ascii_digit()))
    {
        return None;
    }
    Some((
        Literal::Dice(DiceLiteral {
            count,
            sides,
            suffix,
        }),
        pos,
    ))
}

pub(crate) fn band_at(rest: &str, prev: Option<char>) -> Option<(Literal, usize)> {
    if !start_boundary(prev) {
        return None;
    }
    let mut pos = 0;
    let lo = if rest.starts_with(|c: char| c.is_ascii_digit()) {
        Some(digits(rest, &mut pos)?)
    } else {
        None
    };
    if !rest[pos..].starts_with("..") {
        return None;
    }
    pos += 2;
    let hi = if rest[pos..].starts_with(|c: char| c.is_ascii_digit()) {
        Some(digits(rest, &mut pos)?)
    } else {
        None
    };
    if lo.is_none() && hi.is_none() {
        return None;
    }
    // Bands remain unsigned integer shorthand, not a general infix operator.
    // Do not trim newlines: a standalone band followed by another statement is valid.
    let tail = rest[pos..].trim_start_matches([' ', '\t']);
    if !tail.is_empty()
        && !matches!(
            tail.chars().next(),
            Some(',' | ')' | ']' | '}' | ';' | '\n' | '\r' | '#')
        )
    {
        return None;
    }
    Some((Literal::Band { lo, hi }, pos))
}

fn digits(source: &str, pos: &mut usize) -> Option<i32> {
    let start = *pos;
    while source.as_bytes().get(*pos).is_some_and(u8::is_ascii_digit) {
        *pos += 1;
    }
    source[start..*pos].parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dice_spelling_and_full_boundaries() {
        for s in ["d6", "D20", "2d6", "4D6DL1", "4d6kh2", "4d6KL2", "4d6dh1"] {
            assert_eq!(dice_at(s, None).map(|(_, len)| len), Some(s.len()), "{s}");
        }
        for s in [
            "4d6dl",
            "4d6dl0",
            "4d6dL1",
            "4d6foo",
            "4d6_",
            "4d6é",
            "2147483648d6",
            "4d2147483648",
            "4d6dl2147483648",
            "4d6..",
            "4d6.5",
        ] {
            assert_eq!(dice_at(s, None), None, "{s}");
        }
        for prev in ['a', '4', '_', '.', 'é'] {
            assert_eq!(dice_at("4d6", Some(prev)), None);
        }
    }

    #[test]
    fn bands_are_integer_literals_not_an_operator() {
        for s in ["1..5", "..5", "5..", "0..2147483647"] {
            assert_eq!(band_at(s, None).map(|(_, len)| len), Some(s.len()), "{s}");
        }
        for s in [
            "1.5",
            "1...5",
            "..",
            "1..5.5",
            "1..5e2",
            "1..x",
            "1..2 + 3",
            "..2147483648",
            "2147483648..",
        ] {
            assert_eq!(band_at(s, None), None, "{s}");
        }
        assert_eq!(band_at("1..5\nx = 1", None).map(|(_, len)| len), Some(4));
        assert_eq!(
            band_at("1..5 # don't change this", None).map(|(_, len)| len),
            Some(4)
        );
    }
}
