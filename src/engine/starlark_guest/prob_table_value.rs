use std::fmt;

use allocative::Allocative;
use starlark::any::ProvidesStaticType;
use starlark::starlark_simple_value;
use starlark::values::starlark_value;
use starlark::values::{NoSerialize, StarlarkValue};

/// Named rows of independent probabilities (need not sum to 1).
#[derive(Debug, Clone, ProvidesStaticType, NoSerialize, Allocative)]
pub struct StarlarkProbTable {
    pub(crate) rows: Vec<(String, f64)>,
}

impl StarlarkProbTable {
    pub fn new(rows: Vec<(String, f64)>) -> Self {
        Self { rows }
    }

    pub fn rows(&self) -> &[(String, f64)] {
        &self.rows
    }
}

impl fmt::Display for StarlarkProbTable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ProbTable({} rows)", self.rows.len())
    }
}

starlark_simple_value!(StarlarkProbTable);

#[starlark_value(type = "ProbTable")]
impl<'v> StarlarkValue<'v> for StarlarkProbTable {}
