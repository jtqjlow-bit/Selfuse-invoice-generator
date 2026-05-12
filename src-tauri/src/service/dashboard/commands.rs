use tauri::State;

use crate::error::AppResult;
use crate::AppState;

use super::service;
use super::types::DashboardData;

#[tauri::command]
pub fn dashboard_get_data(state: State<'_, AppState>) -> AppResult<DashboardData> {
    service::get_dashboard_data(&state.db)
}
