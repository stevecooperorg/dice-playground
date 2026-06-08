use super::models::WorkspaceState;
use wasm_bindgen::JsValue;
use web_sys::window;

const STORAGE_KEY: &str = "dice_playground_v1";

pub fn load_workspace() -> WorkspaceState {
    let Some(window) = window() else {
        return WorkspaceState::new_default();
    };
    let Ok(Some(storage)) = window.local_storage() else {
        return WorkspaceState::new_default();
    };
    match storage.get_item(STORAGE_KEY) {
        Ok(Some(data)) => serde_json::from_str(&data).unwrap_or_else(|_| WorkspaceState::new_default()),
        _ => WorkspaceState::new_default(),
    }
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

    #[test]
    fn workspace_roundtrip_json() {
        let state = WorkspaceState::new_default();
        let json = serde_json::to_string(&state).expect("serialize");
        let back: WorkspaceState = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.files.len(), state.files.len());
        assert_eq!(back.active_id, state.active_id);
    }
}
