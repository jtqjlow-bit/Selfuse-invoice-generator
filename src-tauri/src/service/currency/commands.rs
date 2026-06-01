use tauri::State;

use crate::error::AppResult;
use crate::AppState;

use super::service;
use super::types::ExchangeRate;

#[tauri::command]
pub fn currency_get_rate(state: State<'_, AppState>, from: String, to: String) -> AppResult<f64> {
    service::get_rate(&state.db, &from, &to)
}

#[tauri::command]
pub fn currency_convert(
    state: State<'_, AppState>,
    amount: f64,
    from: String,
    to: String,
) -> AppResult<f64> {
    service::convert(&state.db, amount, &from, &to)
}

#[tauri::command]
pub fn currency_refresh(state: State<'_, AppState>) -> AppResult<u32> {
    service::refresh(&state.db)
}

#[tauri::command]
pub fn currency_list_cached(state: State<'_, AppState>) -> AppResult<Vec<ExchangeRate>> {
    service::list_cached(&state.db)
}
