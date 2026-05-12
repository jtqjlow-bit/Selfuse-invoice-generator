use std::path::PathBuf;

use tauri::State;

use crate::error::AppResult;
use crate::AppState;

use super::service;

#[tauri::command]
pub fn backup_export_zip(
    state: State<'_, AppState>,
    target_path: String,
) -> AppResult<()> {
    service::export_zip(&state.db, &state.data_dir, &PathBuf::from(target_path))
}

#[tauri::command]
pub fn backup_restore_zip(
    state: State<'_, AppState>,
    zip_path: String,
) -> AppResult<()> {
    service::restore_zip(&state.data_dir, &PathBuf::from(zip_path))
}
