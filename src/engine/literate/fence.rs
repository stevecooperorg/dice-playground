pub(crate) struct FenceOpener {
    pub tick_count: usize,
    pub executable: bool,
}

pub(crate) fn parse_fence_opener(line: &str) -> Option<FenceOpener> {
    let trimmed = line.trim_end();
    if !trimmed.starts_with('`') {
        return None;
    }
    let tick_count = trimmed.chars().take_while(|&c| c == '`').count();
    if tick_count < 3 {
        return None;
    }
    let rest = &trimmed[tick_count..];
    let info = rest.trim();
    let executable = info.is_empty() || info == "dice";
    Some(FenceOpener {
        tick_count,
        executable,
    })
}

pub(crate) fn is_closing_fence(line: &str, open_ticks: usize) -> bool {
    let trimmed = line.trim_end();
    if !trimmed.starts_with('`') {
        return false;
    }
    let count = trimmed.chars().take_while(|&c| c == '`').count();
    if count < open_ticks {
        return false;
    }
    trimmed[count..].trim().is_empty()
}
