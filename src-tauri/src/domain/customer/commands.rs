use tauri::State;

use crate::error::AppResult;
use crate::AppState;

use super::service;
use super::types::{CreateCustomerInput, Customer, UpdateCustomerInput};

#[tauri::command]
pub fn customer_create(
    state: State<'_, AppState>,
    payload: CreateCustomerInput,
) -> AppResult<Customer> {
    service::create(&state.db, payload)
}

#[tauri::command]
pub fn customer_update(
    state: State<'_, AppState>,
    payload: UpdateCustomerInput,
) -> AppResult<Customer> {
    service::update(&state.db, payload)
}

#[tauri::command]
pub fn customer_find_by_id(state: State<'_, AppState>, id: String) -> AppResult<Customer> {
    service::find_by_id(&state.db, &id)
}

#[tauri::command]
pub fn customer_list(
    state: State<'_, AppState>,
    include_archived: bool,
) -> AppResult<Vec<Customer>> {
    service::list(&state.db, include_archived)
}

#[tauri::command]
pub fn customer_search(
    state: State<'_, AppState>,
    query: String,
    include_archived: bool,
) -> AppResult<Vec<Customer>> {
    service::search(&state.db, &query, include_archived)
}

#[tauri::command]
pub fn customer_archive(state: State<'_, AppState>, id: String) -> AppResult<Customer> {
    service::archive(&state.db, &id)
}

#[tauri::command]
pub fn customer_unarchive(state: State<'_, AppState>, id: String) -> AppResult<Customer> {
    service::unarchive(&state.db, &id)
}
