use tauri::State;

use crate::error::AppResult;
use crate::AppState;

use super::service;
use super::types::{PdfDocType, PdfTemplate, UploadCustomTemplateInput};

#[tauri::command]
pub fn pdf_template_list(state: State<'_, AppState>) -> AppResult<Vec<PdfTemplate>> {
    service::list(&state.db)
}

#[tauri::command]
pub fn pdf_template_list_by_doc_type(
    state: State<'_, AppState>,
    doc_type: PdfDocType,
) -> AppResult<Vec<PdfTemplate>> {
    service::list_by_doc_type(&state.db, doc_type)
}

#[tauri::command]
pub fn pdf_template_find_by_id(
    state: State<'_, AppState>,
    id: String,
) -> AppResult<PdfTemplate> {
    service::find_by_id(&state.db, &id)
}

#[tauri::command]
pub fn pdf_template_upload_custom(
    state: State<'_, AppState>,
    payload: UploadCustomTemplateInput,
) -> AppResult<PdfTemplate> {
    service::upload_custom(&state.db, &state.data_dir, payload)
}

#[tauri::command]
pub fn pdf_template_delete_custom(state: State<'_, AppState>, id: String) -> AppResult<()> {
    service::delete_custom(&state.db, &id)
}

#[tauri::command]
pub fn pdf_template_get_renderable(
    state: State<'_, AppState>,
    id: String,
) -> AppResult<String> {
    service::get_renderable(&state.db, &id)
}
