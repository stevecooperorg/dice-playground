use serde::{Deserialize, Serialize};

pub fn new_file_id() -> String {
    #[cfg(target_arch = "wasm32")]
    {
        format!("f-{}", js_sys::Math::random())
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::sync::atomic::{AtomicU64, Ordering};
        static N: AtomicU64 = AtomicU64::new(0);
        format!("f-{}", N.fetch_add(1, Ordering::Relaxed))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DiceFile {
    pub id: String,
    pub name: String,
    pub content: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkspaceState {
    pub files: Vec<DiceFile>,
    pub active_id: String,
}

impl WorkspaceState {
    pub fn new_default() -> Self {
        let id = new_file_id();
        Self {
            files: vec![DiceFile {
                id: id.clone(),
                name: "welcome.dice".to_owned(),
                content: "# Welcome\n\nTry **Run** to see the woven report.\n\n```dice\noutput(\"two_d6\", 2d6)\n```\n"
                    .to_owned(),
            }],
            active_id: id,
        }
    }

    pub fn active_file(&self) -> Option<&DiceFile> {
        self.files.iter().find(|f| f.id == self.active_id)
    }

    pub fn active_file_mut(&mut self) -> Option<&mut DiceFile> {
        let id = self.active_id.clone();
        self.files.iter_mut().find(|f| f.id == id)
    }

    /// `untitled.dice`, then `untitled (1).dice`, `untitled (2).dice`, … avoiding existing names.
    pub fn suggest_new_filename(&self) -> String {
        let names: Vec<&str> = self.files.iter().map(|f| f.name.as_str()).collect();
        suggest_untitled_name(&names)
    }
}

pub fn suggest_untitled_name(existing: &[&str]) -> String {
    use std::collections::HashSet;
    let taken: HashSet<&str> = existing.iter().copied().collect();
    if !taken.contains("untitled.dice") {
        return "untitled.dice".to_owned();
    }
    for n in 1.. {
        let name = format!("untitled ({n}).dice");
        if !taken.contains(name.as_str()) {
            return name;
        }
    }
    unreachable!("unbounded untitled suffixes")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn suggest_untitled_skips_taken_names() {
        assert_eq!(suggest_untitled_name(&[]), "untitled.dice");
        assert_eq!(
            suggest_untitled_name(&["untitled.dice"]),
            "untitled (1).dice"
        );
        assert_eq!(
            suggest_untitled_name(&["untitled.dice", "untitled (1).dice"]),
            "untitled (2).dice"
        );
        assert_eq!(
            suggest_untitled_name(&["untitled (1).dice", "other.dice"]),
            "untitled.dice"
        );
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct UiDiagnostic {
    pub line: u32,
    pub column: u32,
    pub message: String,
    pub severity: String,
}
