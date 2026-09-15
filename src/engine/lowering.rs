//! Lower recognized literal tokens to ordinary calls, with an original-source location map.
//! The legacy pool-versus-sum policy is deliberately separate from lexical recognition.

use std::ops::Range;

use super::lex::lex_source;
use super::literals::{DiceLiteral, Literal, PoolSuffix};

#[derive(Debug)]
struct Replacement {
    original: Range<usize>,
    expanded: Range<usize>,
}

pub(crate) struct Expansion {
    pub source: String,
    replacements: Vec<Replacement>,
}

impl Expansion {
    /// Map an expanded byte offset back to the input. Generated call interiors
    /// cover the whole original literal; unchanged text maps byte-for-byte.
    pub fn original_offset(&self, offset: usize, end: bool) -> usize {
        let i = self
            .replacements
            .partition_point(|r| r.expanded.start <= offset);
        let Some(r) = i.checked_sub(1).and_then(|i| self.replacements.get(i)) else {
            return offset;
        };
        if offset < r.expanded.end {
            if end && offset > r.expanded.start {
                r.original.end
            } else {
                r.original.start
            }
        } else {
            r.original.end + (offset - r.expanded.end)
        }
    }
}

pub(crate) fn lower(source: &str, dice: bool, bands: bool) -> Expansion {
    let mut out = String::with_capacity(source.len());
    let mut replacements = Vec::new();
    for token in lex_source(source) {
        let replacement = match token.literal {
            Some(Literal::Dice(d)) if dice => Some(expand_dice(d, &source[token.range.end..])),
            Some(Literal::Band { lo, hi }) if bands => match (lo, hi) {
                (Some(lo), Some(hi)) => Some(format!("through({lo}, {hi})")),
                (Some(lo), None) => Some(format!("at_least({lo})")),
                (None, Some(hi)) => Some(format!("at_most({hi})")),
                (None, None) => None,
            },
            _ => None,
        };
        if let Some(replacement) = replacement {
            let start = out.len();
            out.push_str(&replacement);
            replacements.push(Replacement {
                original: token.range,
                expanded: start..out.len(),
            });
        } else {
            out.push_str(&source[token.range]);
        }
    }
    Expansion {
        source: out,
        replacements,
    }
}

fn expand_dice(dice: DiceLiteral, tail: &str) -> String {
    let DiceLiteral {
        count,
        sides,
        suffix,
    } = dice;
    if let Some((op, n)) = suffix {
        let helper = match op {
            PoolSuffix::DropLowest => "drop_lowest",
            PoolSuffix::DropHighest => "drop_highest",
            PoolSuffix::KeepHighest => "keep_highest",
            PoolSuffix::KeepLowest => "keep_lowest",
        };
        format!("{helper}({count}, {sides}, {n})")
    } else if count == 1 {
        format!("d({sides})")
    } else {
        let pool = format!("dice_pool({count}, {sides})");
        if legacy_auto_sum(tail) {
            format!("sum({pool})")
        } else {
            pool
        }
    }
}

/// Retained compatibility debt, NOT a proposed type/grammar rule. Historically
/// these following characters trigger a sum, even across whitespace/newlines.
/// Thus `2d6` is a pool, `(2d6)` sums, and `2d6.keep(5..)` stays a pool.
fn legacy_auto_sum(tail: &str) -> bool {
    matches!(
        tail.trim_start().chars().next(),
        Some('+' | '-' | '*' | '/' | ')' | ',' | ']' | '>' | '<' | '=')
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retained_context_dependent_auto_sum() {
        for (source, expected) in [
            ("2d6", "dice_pool(2, 6)"),
            ("02d6", "dice_pool(2, 6)"),
            ("output(02d6)", "output(sum(dice_pool(2, 6)))"),
            ("04d6dl1", "drop_lowest(4, 6, 1)"),
            ("(2d6)", "(sum(dice_pool(2, 6)))"),
            ("2d6.keep(5..)", "dice_pool(2, 6).keep(at_least(5))"),
            ("output(2d6)", "output(sum(dice_pool(2, 6)))"),
            ("2d6 # comment\n+ 1", "dice_pool(2, 6) # comment\n+ 1"),
        ] {
            assert_eq!(lower(source, true, true).source, expected);
        }
    }

    #[test]
    fn offsets_inside_and_after_multiple_expansions() {
        let source = "é = [2d6, 1..5]; missing";
        // Use valid Starlark before shorthand; Unicode remains legal in strings.
        let source = source.replacen('é', "\"é\"", 1);
        let expanded = lower(&source, true, true);
        let missing = expanded.source.find("missing").unwrap();
        assert_eq!(
            expanded.original_offset(missing, false),
            source.find("missing").unwrap()
        );
        let inside = expanded.source.find("dice_pool").unwrap();
        assert_eq!(
            expanded.original_offset(inside, false),
            source.find("2d6").unwrap()
        );
        assert_eq!(
            expanded.original_offset(inside, true),
            source.find("2d6").unwrap() + 3
        );
        assert_eq!(
            expanded.original_offset(expanded.source.len(), true),
            source.len()
        );
    }
}
