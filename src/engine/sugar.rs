//! Compatibility APIs for lowering tabletop literals to ordinary Starlark calls.
//! Recognition is shared with highlighting; strings, comments, and whole native
//! tokens are never rewritten. See `docs/tutorial/13-keep-and-drop.dice`.

/// Expand dice and inclusive integer-band shorthand, preserving all other text.
///
/// ```
/// use dice_playground::engine::desugar_if_needed;
/// assert_eq!(desugar_if_needed("x.dice", "2d6.keep(5..)").unwrap(),
///            "dice_pool(2, 6).keep(at_least(5))");
/// ```
pub fn desugar_if_needed(_path: &str, source: &str) -> anyhow::Result<String> {
    Ok(super::lowering::lower(source, true, true).source)
}

/// Byte length of a complete dice spelling at a known token boundary.
/// This compatibility helper does not track strings/comments; use `lex_source`
/// or `lex_document` to highlight source safely.
///
/// ```
/// use dice_playground::engine::dice_literal_len_at;
/// assert_eq!(dice_literal_len_at("2d6 + 1", None), Some(3));
/// ```
pub fn dice_literal_len_at(rest: &str, prev: Option<char>) -> Option<usize> {
    super::literals::dice_at(rest, prev).map(|(_, len)| len)
}

/// Expand only dice shorthand; keep the historical context-sensitive auto-sum policy.
///
/// ```
/// use dice_playground::engine::desugar;
/// assert_eq!(desugar("x.dice", "4d6dl1").unwrap(), "drop_lowest(4, 6, 1)");
/// ```
pub fn desugar(_path: &str, source: &str) -> anyhow::Result<String> {
    Ok(super::lowering::lower(source, true, false).source)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lexical_regressions_are_preserved() {
        for source in [
            r#"output("4d6dl1", d(6))"#,
            "foo4d6",
            "0x2d6",
            "1.5d6",
            "4d6dl",
            "4d6dl0",
            "4d6foo",
            "4d6kh",
            "4d6dL1",
            "4d6.5",
            "# don't lower 2d6 or 1..5",
            "'4d6dl1'",
            "r'4d6dl1'",
            "'''4d6\n1..5'''",
            "\"unterminated 4d6",
            "2..d6",
            "0x2..6",
            "1..5.5",
            "1..5e2",
            "1...5",
            "..",
            "2147483648d6",
        ] {
            assert_eq!(
                desugar_if_needed("t.dice", source).unwrap(),
                source,
                "{source:?}"
            );
        }
        assert_eq!(
            desugar_if_needed("t.dice", "# don't\n1..5\n2d6").unwrap(),
            "# don't\nthrough(1, 5)\ndice_pool(2, 6)"
        );
    }

    #[test]
    fn desugar_4d6dl1_in_output() {
        let src = r#"output("x", 4d6dl1)"#;
        let out = desugar("t.star", src).unwrap();
        assert!(out.contains("drop_lowest(4, 6, 1)"));
    }

    #[test]
    fn desugar_2d10() {
        let out = desugar("t.star", "2d10").unwrap();
        assert_eq!(out, "dice_pool(2, 10)");
    }

    #[test]
    fn desugar_2d10_plus_auto_sum() {
        let out = desugar("t.star", "2d10 + 3").unwrap();
        assert_eq!(out, "sum(dice_pool(2, 10)) + 3");
    }

    #[test]
    fn desugar_8d6_times_ten_auto_sum() {
        let out = desugar("t.star", "8d6 * 10").unwrap();
        assert_eq!(out, "sum(dice_pool(8, 6)) * 10");
    }

    #[test]
    fn desugar_8d6_floor_div_two_auto_sum() {
        let out = desugar("t.star", "8d6 // 2").unwrap();
        assert_eq!(out, "sum(dice_pool(8, 6)) // 2");
    }

    #[test]
    fn desugar_1d4_times_ten() {
        let out = desugar("t.star", "1d4 * 10").unwrap();
        assert_eq!(out, "d(4) * 10");
    }

    #[test]
    fn desugar_output_wraps_pool_sum() {
        let out = desugar("t.star", r#"output("x", 2d6)"#).unwrap();
        assert!(out.contains("sum(dice_pool(2, 6))"));
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
