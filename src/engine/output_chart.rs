//! Which `output()` results get inline charts in woven HTML (shared with UI graph tab).

use crate::engine::starlark_guest::OutputEntry;

/// Max `prob_table` rows before inline bar chart is omitted (modifier grids).
pub const MAX_TABLE_CHART_ROWS: usize = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChartKind {
    DieRoll,
    Outcomes,
    Prob,
    Table,
}

impl ChartKind {
    pub fn as_str(self) -> &'static str {
        match self {
            ChartKind::DieRoll => "dieroll",
            ChartKind::Outcomes => "outcomes",
            ChartKind::Prob => "prob",
            ChartKind::Table => "table",
        }
    }
}

pub fn output_entry_name(entry: &OutputEntry) -> &str {
    match entry {
        OutputEntry::DieRoll { name, .. }
        | OutputEntry::Prob { name, .. }
        | OutputEntry::Outcomes { name, .. }
        | OutputEntry::Table { name, .. } => name,
    }
}

pub fn chart_kind_for_entry(entry: &OutputEntry) -> Option<ChartKind> {
    match entry {
        OutputEntry::DieRoll { entries, .. } if !entries.is_empty() => Some(ChartKind::DieRoll),
        OutputEntry::Outcomes { entries, .. } if !entries.is_empty() => Some(ChartKind::Outcomes),
        OutputEntry::Prob { .. } => Some(ChartKind::Prob),
        OutputEntry::Table { entries, .. } if !entries.is_empty() && entries.len() <= MAX_TABLE_CHART_ROWS => {
            Some(ChartKind::Table)
        }
        _ => None,
    }
}

pub fn output_entry_supports_chart(entry: &OutputEntry) -> bool {
    chart_kind_for_entry(entry).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn large_prob_table_skipped() {
        let entries: Vec<(String, f64)> = (0..40)
            .map(|i| (format!("row {i}"), 0.01))
            .collect();
        let entry = OutputEntry::Table {
            name: "grid".into(),
            entries,
        };
        assert!(!output_entry_supports_chart(&entry));
    }

    #[test]
    fn d6_supports_dieroll_kind() {
        let entry = OutputEntry::DieRoll {
            name: "d6".into(),
            entries: vec![(1, 1.0 / 6.0)],
            mean: 1.0,
        };
        assert_eq!(chart_kind_for_entry(&entry), Some(ChartKind::DieRoll));
    }
}
