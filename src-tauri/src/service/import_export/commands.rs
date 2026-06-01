use tauri::State;

use crate::error::AppResult;
use crate::AppState;

use super::service;
use super::types::ImportReport;

#[tauri::command]
pub fn import_customers_csv(
    state: State<'_, AppState>,
    file_path: String,
) -> AppResult<ImportReport> {
    service::import_customers_from_csv(&state.db, &file_path)
}

#[tauri::command]
pub fn export_all_excel(state: State<'_, AppState>, target_path: String) -> AppResult<()> {
    service::export_all_to_excel(&state.db, &target_path)
}
