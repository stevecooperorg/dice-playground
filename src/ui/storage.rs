use super::models::{new_file_id, DiceFile, WorkspaceState};
use super::playground_handoff::{
    decode_payload_json, PendingScriptLoad, LOAD_QUERY_PARAM, PENDING_LOCAL_STORAGE_KEY,
};
use wasm_bindgen::JsValue;
use web_sys::window;

const STORAGE_KEY: &str = "dice_playground_v1";

pub use super::playground_handoff::PENDING_LOCAL_STORAGE_KEY as PENDING_SCRIPT_LOAD_KEY;

/// Add a new file from doc \"Load in playground\" (or similar) and make it active.
pub fn apply_script_load(
    mut ws: WorkspaceState,
    content: String,
    filename: Option<String>,
) -> WorkspaceState {
    let name = filename.unwrap_or_else(|| ws.suggest_new_filename());
    let id = new_file_id();
    ws.files.push(DiceFile {
        id: id.clone(),
        name,
        content,
    });
    ws.active_id = id;
    ws
}

pub fn load_workspace() -> WorkspaceState {
    let ws = load_workspace_from_local_storage();
    workspace_with_pending_script(ws)
}

fn load_workspace_from_local_storage() -> WorkspaceState {
    let Some(window) = window() else {
        return WorkspaceState::new_default();
    };
    let Ok(Some(storage)) = window.local_storage() else {
        return WorkspaceState::new_default();
    };
    match storage.get_item(STORAGE_KEY) {
        Ok(Some(data)) => {
            serde_json::from_str(&data).unwrap_or_else(|_| WorkspaceState::new_default())
        }
        _ => WorkspaceState::new_default(),
    }
}

pub fn workspace_with_pending_script(ws: WorkspaceState) -> WorkspaceState {
    if let Some(pending) = take_pending_script_load() {
        apply_script_load(ws, pending.content, pending.filename)
    } else {
        ws
    }
}

fn take_pending_script_load() -> Option<PendingScriptLoad> {
    if let Some(pending) = take_pending_from_query() {
        clear_load_query_from_location();
        return Some(pending);
    }
    take_pending_from_local_storage()
}

fn take_pending_from_query() -> Option<PendingScriptLoad> {
    let window = window()?;
    let search = window.location().search().ok()?;
    let params = web_sys::UrlSearchParams::new_with_str(&search).ok()?;
    let raw = params.get(LOAD_QUERY_PARAM)?;
    if raw.is_empty() {
        return None;
    }
    decode_payload_json(&raw)
}

fn take_pending_from_local_storage() -> Option<PendingScriptLoad> {
    let window = window()?;
    let storage = window.local_storage().ok()??;
    let raw = storage.get_item(PENDING_LOCAL_STORAGE_KEY).ok()??;
    let _ = storage.remove_item(PENDING_LOCAL_STORAGE_KEY);
    decode_payload_json(&raw)
}

fn clear_load_query_from_location() {
    let Some(window) = window() else {
        return;
    };
    let Ok(location) = window.location().pathname() else {
        return;
    };
    let Ok(history) = window.history() else {
        return;
    };
    let _ = history.replace_state_with_url(&JsValue::NULL, "", Some(&location));
}

pub fn save_workspace(state: &WorkspaceState) -> Result<(), JsValue> {
    let window = window().ok_or_else(|| JsValue::from_str("no window"))?;
    let storage = window
        .local_storage()?
        .ok_or_else(|| JsValue::from_str("no localStorage"))?;
    let json = serde_json::to_string(state).map_err(|e| JsValue::from_str(&e.to_string()))?;
    storage.set_item(STORAGE_KEY, &json)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::ui::models::WorkspaceState;
    use crate::ui::storage::apply_script_load;

    #[test]
    fn apply_script_load_adds_new_active_file() {
        let ws = WorkspaceState::new_default();
        let updated = apply_script_load(ws, "output(\"x\", 1d6)\n".to_owned(), None);
        assert_eq!(updated.files.len(), 2);
        let active = updated.active_file().expect("active");
        assert_eq!(active.content, "output(\"x\", 1d6)\n");
        assert_eq!(active.name, "untitled.dice");
    }

    #[test]
    fn workspace_roundtrip_json() {
        let state = WorkspaceState::new_default();
        let json = serde_json::to_string(&state).expect("serialize");
        let back: WorkspaceState = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.files.len(), state.files.len());
        assert_eq!(back.active_id, state.active_id);
    }
}
