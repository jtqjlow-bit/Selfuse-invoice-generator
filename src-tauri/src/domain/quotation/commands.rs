use tauri::State;

use crate::error::AppResult;
use crate::AppState;

use super::service;
use super::types::{
    CreateQuotationInput, Quotation, QuotationWithLines, UpdateQuotationInput,
};

#[tauri::command]
pub fn quotation_create(
    state: State<'_, AppState>,
    payload: CreateQuotationInput,
) -> AppResult<QuotationWithLines> {
    service::create(&state.db, payload)
}

#[tauri::command]
pub fn quotation_update(
    state: State<'_, AppState>,
    payload: UpdateQuotationInput,
) -> AppResult<QuotationWithLines> {
    service::update(&state.db, payload)
}

#[tauri::command]
pub fn quotation_find_by_id(
    state: State<'_, AppState>,
    id: String,
) -> AppResult<QuotationWithLines> {
    service::find_by_id(&state.db, &id)
}

#[tauri::command]
pub fn quotation_list(state: State<'_, AppState>) -> AppResult<Vec<Quotation>> {
    service::list(&state.db)
}

#[tauri::command]
pub fn quotation_list_by_customer(
    state: State<'_, AppState>,
    customer_id: String,
) -> AppResult<Vec<Quotation>> {
    service::list_by_customer(&state.db, &customer_id)
}

#[tauri::command]
pub fn quotation_mark_sent(state: State<'_, AppState>, id: String) -> AppResult<Quotation> {
    service::mark_sent(&state.db, &id)
}

#[tauri::command]
pub fn quotation_mark_accepted(state: State<'_, AppState>, id: String) -> AppResult<Quotation> {
    service::mark_accepted(&state.db, &id)
}

#[tauri::command]
pub fn quotation_mark_rejected(state: State<'_, AppState>, id: String) -> AppResult<Quotation> {
    service::mark_rejected(&state.db, &id)
}

#[tauri::command]
pub fn quotation_mark_expired(state: State<'_, AppState>, id: String) -> AppResult<Quotation> {
    service::mark_expired(&state.db, &id)
}
