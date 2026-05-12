use tauri::State;

use crate::error::AppResult;
use crate::AppState;

use super::service;
use super::types::DocType;

#[tauri::command]
pub fn numbering_peek(state: State<'_, AppState>, doc: DocType) -> AppResult<String> {
    service::peek(&state.db, doc)
}

#[tauri::command]
pub fn numbering_set_override(
    state: State<'_, AppState>,
    doc: DocType,
    year: i32,
    seq: i64,
) -> AppResult<()> {
    service::set_override(&state.db, doc, year, seq)
}
