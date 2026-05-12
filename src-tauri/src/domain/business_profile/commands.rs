use tauri::State;

use crate::error::AppResult;
use crate::AppState;

use super::service;
use super::types::{
    BusinessProfile, CreateBusinessProfileInput, ProfileAssetDataUrls, QrKind,
    UpdateBusinessProfileInput,
};

#[tauri::command]
pub fn business_profile_list(
    state: State<'_, AppState>,
) -> AppResult<Vec<BusinessProfile>> {
    service::list(&state.db)
}

#[tauri::command]
pub fn business_profile_find_by_id(
    state: State<'_, AppState>,
    id: String,
) -> AppResult<BusinessProfile> {
    service::find_by_id(&state.db, &id)
}

#[tauri::command]
pub fn business_profile_create(
    state: State<'_, AppState>,
    payload: CreateBusinessProfileInput,
) -> AppResult<BusinessProfile> {
    service::create(&state.db, payload)
}

#[tauri::command]
pub fn business_profile_update(
    state: State<'_, AppState>,
    payload: UpdateBusinessProfileInput,
) -> AppResult<BusinessProfile> {
    service::update(&state.db, payload)
}

#[tauri::command]
pub fn business_profile_delete(
    state: State<'_, AppState>,
    id: String,
) -> AppResult<()> {
    service::delete(&state.db, &id)
}

#[tauri::command]
pub fn business_profile_set_logo(
    state: State<'_, AppState>,
    id: String,
    bytes_b64: String,
    ext: String,
) -> AppResult<BusinessProfile> {
    service::set_logo(&state.db, &state.data_dir, &id, &bytes_b64, &ext)
}

#[tauri::command]
pub fn business_profile_clear_logo(
    state: State<'_, AppState>,
    id: String,
) -> AppResult<BusinessProfile> {
    service::clear_logo(&state.db, &id)
}

#[tauri::command]
pub fn business_profile_set_qr(
    state: State<'_, AppState>,
    id: String,
    bytes_b64: String,
    ext: String,
) -> AppResult<BusinessProfile> {
    service::set_qr(&state.db, &state.data_dir, &id, &bytes_b64, &ext)
}

#[tauri::command]
pub fn business_profile_clear_qr(
    state: State<'_, AppState>,
    id: String,
) -> AppResult<BusinessProfile> {
    service::clear_qr(&state.db, &id)
}

#[tauri::command]
pub fn business_profile_add_qr(
    state: State<'_, AppState>,
    id: String,
    kind: QrKind,
    label: String,
    bytes_b64: String,
    ext: String,
) -> AppResult<BusinessProfile> {
    service::add_qr(&state.db, &state.data_dir, &id, kind, &label, &bytes_b64, &ext)
}

#[tauri::command]
pub fn business_profile_remove_qr(
    state: State<'_, AppState>,
    id: String,
    qr_id: String,
) -> AppResult<BusinessProfile> {
    service::remove_qr(&state.db, &id, &qr_id)
}

#[tauri::command]
pub fn business_profile_update_qr_label(
    state: State<'_, AppState>,
    id: String,
    qr_id: String,
    label: String,
) -> AppResult<BusinessProfile> {
    service::update_qr_label(&state.db, &id, &qr_id, &label)
}

#[tauri::command]
pub fn business_profile_get_asset_data_urls(
    state: State<'_, AppState>,
    id: String,
) -> AppResult<ProfileAssetDataUrls> {
    service::get_asset_data_urls(&state.db, &id)
}
