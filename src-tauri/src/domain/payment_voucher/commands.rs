use tauri::State;

use crate::error::AppResult;
use crate::AppState;

use super::service;
use super::types::{CreatePaymentVoucherInput, PaymentVoucher, UpdatePaymentVoucherInput};

#[tauri::command]
pub fn payment_voucher_create(
    state: State<'_, AppState>,
    payload: CreatePaymentVoucherInput,
) -> AppResult<PaymentVoucher> {
    service::create(&state.db, payload)
}

#[tauri::command]
pub fn payment_voucher_update(
    state: State<'_, AppState>,
    payload: UpdatePaymentVoucherInput,
) -> AppResult<PaymentVoucher> {
    service::update(&state.db, payload)
}

#[tauri::command]
pub fn payment_voucher_delete(state: State<'_, AppState>, id: String) -> AppResult<()> {
    service::delete(&state.db, &id)
}

#[tauri::command]
pub fn payment_voucher_find_by_id(
    state: State<'_, AppState>,
    id: String,
) -> AppResult<PaymentVoucher> {
    service::find_by_id(&state.db, &id)
}

#[tauri::command]
pub fn payment_voucher_list(state: State<'_, AppState>) -> AppResult<Vec<PaymentVoucher>> {
    service::list(&state.db)
}

#[tauri::command]
pub fn payment_voucher_list_by_invoice(
    state: State<'_, AppState>,
    invoice_id: String,
) -> AppResult<Vec<PaymentVoucher>> {
    service::list_by_invoice(&state.db, &invoice_id)
}

#[tauri::command]
pub fn payment_voucher_list_by_customer(
    state: State<'_, AppState>,
    customer_id: String,
) -> AppResult<Vec<PaymentVoucher>> {
    service::list_by_customer(&state.db, &customer_id)
}

#[tauri::command]
pub fn payment_voucher_sum_by_invoice(
    state: State<'_, AppState>,
    invoice_id: String,
) -> AppResult<f64> {
    service::sum_by_invoice(&state.db, &invoice_id)
}
