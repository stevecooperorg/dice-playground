//! Regression tests for inclusive integer-band lowering, shared with dice lexing.

#[cfg(test)]
fn desugar_ranges(source: &str) -> anyhow::Result<String> {
    Ok(super::lowering::lower(source, false, true).source)
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
    fn scale_with_range_bands() {
        assert_eq!(
            desugar_ranges(r#"scale().step("FAIL", ..14).step("PASS", 15..)"#).unwrap(),
            r#"scale().step("FAIL", at_most(14)).step("PASS", at_least(15))"#
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
