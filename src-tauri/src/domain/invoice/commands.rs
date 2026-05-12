use tauri::State;

use crate::error::AppResult;
use crate::AppState;

use super::service;
use super::types::{
    CreateFromQuotationInput, CreateInvoiceInput, Invoice, InvoiceWithLines, UpdateInvoiceInput,
};

#[tauri::command]
pub fn invoice_create(
    state: State<'_, AppState>,
    payload: CreateInvoiceInput,
) -> AppResult<InvoiceWithLines> {
    service::create(&state.db, payload)
}

#[tauri::command]
pub fn invoice_create_from_quotation(
    state: State<'_, AppState>,
    payload: CreateFromQuotationInput,
) -> AppResult<InvoiceWithLines> {
    service::create_from_quotation(&state.db, payload)
}

#[tauri::command]
pub fn invoice_update(
    state: State<'_, AppState>,
    payload: UpdateInvoiceInput,
) -> AppResult<InvoiceWithLines> {
    service::update(&state.db, payload)
}

#[tauri::command]
pub fn invoice_find_by_id(
    state: State<'_, AppState>,
    id: String,
) -> AppResult<InvoiceWithLines> {
    service::find_by_id(&state.db, &id)
}

#[tauri::command]
pub fn invoice_list(state: State<'_, AppState>) -> AppResult<Vec<Invoice>> {
    service::list(&state.db)
}

#[tauri::command]
pub fn invoice_list_by_customer(
    state: State<'_, AppState>,
    customer_id: String,
) -> AppResult<Vec<Invoice>> {
    service::list_by_customer(&state.db, &customer_id)
}

#[tauri::command]
pub fn invoice_mark_sent(state: State<'_, AppState>, id: String) -> AppResult<Invoice> {
    service::mark_sent(&state.db, &id)
}

#[tauri::command]
pub fn invoice_mark_partial_paid(state: State<'_, AppState>, id: String) -> AppResult<Invoice> {
    service::mark_partial_paid(&state.db, &id)
}

#[tauri::command]
pub fn invoice_mark_paid(state: State<'_, AppState>, id: String) -> AppResult<Invoice> {
    service::mark_paid(&state.db, &id)
}

#[tauri::command]
pub fn invoice_mark_overdue(state: State<'_, AppState>, id: String) -> AppResult<Invoice> {
    service::mark_overdue(&state.db, &id)
}

#[tauri::command]
pub fn invoice_mark_void(state: State<'_, AppState>, id: String) -> AppResult<Invoice> {
    service::mark_void(&state.db, &id)
}

#[tauri::command]
pub fn invoice_cancel_overdue(state: State<'_, AppState>, id: String) -> AppResult<Invoice> {
    service::cancel_overdue(&state.db, &id)
}

#[tauri::command]
pub fn invoice_restore_void(state: State<'_, AppState>, id: String) -> AppResult<Invoice> {
    service::restore_void(&state.db, &id)
}
