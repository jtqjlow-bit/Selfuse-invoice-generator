use std::path::PathBuf;

use tauri::State;

use crate::error::AppResult;
use crate::AppState;

use super::renderer;
use super::types::{
    InvoicePreviewInput, PaymentVoucherPreviewInput, QuotationPreviewInput, RenderResult,
};

#[tauri::command]
pub fn pdf_render_quotation(
    state: State<'_, AppState>,
    quotation_id: String,
    template_id: String,
    target_path: String,
) -> AppResult<RenderResult> {
    renderer::render_quotation(
        &state.db,
        &state.data_dir,
        &quotation_id,
        &template_id,
        &PathBuf::from(target_path),
    )
}

#[tauri::command]
pub fn pdf_render_invoice(
    state: State<'_, AppState>,
    invoice_id: String,
    template_id: String,
    target_path: String,
) -> AppResult<RenderResult> {
    renderer::render_invoice(
        &state.db,
        &state.data_dir,
        &invoice_id,
        &template_id,
        &PathBuf::from(target_path),
    )
}

#[tauri::command]
pub fn pdf_render_payment_voucher(
    state: State<'_, AppState>,
    pv_id: String,
    template_id: String,
    target_path: String,
) -> AppResult<RenderResult> {
    renderer::render_payment_voucher(
        &state.db,
        &state.data_dir,
        &pv_id,
        &template_id,
        &PathBuf::from(target_path),
    )
}

#[tauri::command]
pub fn pdf_preview_quotation_html(
    state: State<'_, AppState>,
    payload: QuotationPreviewInput,
) -> AppResult<String> {
    renderer::render_quotation_html_preview(&state.db, &payload)
}

#[tauri::command]
pub fn pdf_preview_invoice_html(
    state: State<'_, AppState>,
    payload: InvoicePreviewInput,
) -> AppResult<String> {
    renderer::render_invoice_html_preview(&state.db, &payload)
}

#[tauri::command]
pub fn pdf_preview_payment_voucher_html(
    state: State<'_, AppState>,
    payload: PaymentVoucherPreviewInput,
) -> AppResult<String> {
    renderer::render_payment_voucher_html_preview(&state.db, &payload)
}

#[tauri::command]
pub fn pdf_render_template_sample(
    state: State<'_, AppState>,
    template_id: String,
) -> AppResult<String> {
    renderer::render_template_sample(&state.db, &template_id)
}
