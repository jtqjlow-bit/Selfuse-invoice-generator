use std::path::{Path, PathBuf};

use base64::{engine::general_purpose::STANDARD as B64, Engine};
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::infra::{file_system, Db};

use super::repository;
use super::types::{
    BankAccount, BusinessProfile, CreateBusinessProfileInput, EntityType,
    ProfileAssetDataUrls, Qr, QrDataUrl, QrKind, UpdateBusinessProfileInput,
};

/// Max bytes for an uploaded logo. ~2 MB. Larger files almost certainly mean
/// the user dropped in a camera-roll photo by mistake; rejecting at the boundary
/// keeps the SQLite-coupled assets folder lean and avoids slow base64 transfers.
const MAX_LOGO_BYTES: usize = 2 * 1024 * 1024;
/// Max bytes per QR image. ~1 MB.
const MAX_QR_BYTES: usize = 1 * 1024 * 1024;

fn now() -> String {
    chrono::Utc::now().to_rfc3339()
}

pub fn list(db: &Db) -> AppResult<Vec<BusinessProfile>> {
    db.with_conn(|c| repository::list(c))
}

pub fn find_by_id(db: &Db, id: &str) -> AppResult<BusinessProfile> {
    db.with_conn(|c| {
        repository::find_by_id(c, id)?.ok_or_else(|| AppError::NotFound {
            entity: "business_profile".into(),
            id: id.into(),
        })
    })
}

pub fn create(db: &Db, input: CreateBusinessProfileInput) -> AppResult<BusinessProfile> {
    validate_basic(
        input.entity_type,
        &input.name,
        &input.ssm_no,
        &input.nric,
        input.default_tax_rate,
        input.default_quotation_valid_days,
        input.default_invoice_due_days,
    )?;
    let now_s = now();
    let profile = BusinessProfile {
        id: Uuid::new_v4().to_string(),
        entity_type: input.entity_type,
        name: input.name.trim().to_string(),
        address: trim_opt(input.address),
        email: trim_opt(input.email),
        phone: trim_opt(input.phone),
        ssm_no: trim_opt(input.ssm_no),
        nric: trim_opt(input.nric),
        sst_no: trim_opt(input.sst_no),
        logo_path: None,
        qr_path: None,
        bank_accounts: ensure_bank_account_ids(input.bank_accounts),
        qrs: Vec::new(),
        enabled_payment_methods: input.enabled_payment_methods,
        default_tax_rate: input.default_tax_rate,
        default_quotation_valid_days: input.default_quotation_valid_days,
        default_invoice_due_days: input.default_invoice_due_days,
        data_dir: input.data_dir,
        created_at: now_s.clone(),
        updated_at: now_s,
    };
    db.transaction(|tx| repository::insert(tx, &profile))?;
    Ok(profile)
}

pub fn update(db: &Db, input: UpdateBusinessProfileInput) -> AppResult<BusinessProfile> {
    validate_basic(
        input.entity_type,
        &input.name,
        &input.ssm_no,
        &input.nric,
        input.default_tax_rate,
        input.default_quotation_valid_days,
        input.default_invoice_due_days,
    )?;
    db.transaction(|tx| {
        let mut existing = repository::find_by_id(tx, &input.id)?.ok_or_else(|| {
            AppError::NotFound {
                entity: "business_profile".into(),
                id: input.id.clone(),
            }
        })?;
        existing.entity_type = input.entity_type;
        existing.name = input.name.trim().to_string();
        existing.address = trim_opt(input.address);
        existing.email = trim_opt(input.email);
        existing.phone = trim_opt(input.phone);
        existing.ssm_no = trim_opt(input.ssm_no);
        existing.nric = trim_opt(input.nric);
        existing.sst_no = trim_opt(input.sst_no);
        existing.bank_accounts = ensure_bank_account_ids(input.bank_accounts);
        existing.enabled_payment_methods = input.enabled_payment_methods;
        existing.default_tax_rate = input.default_tax_rate;
        existing.default_quotation_valid_days = input.default_quotation_valid_days;
        existing.default_invoice_due_days = input.default_invoice_due_days;
        existing.data_dir = input.data_dir;
        // Individuals don't get a logo — force-clear if type was switched.
        if existing.entity_type == EntityType::Individual && existing.logo_path.is_some() {
            existing.logo_path = None;
        }
        existing.updated_at = now();
        repository::update(tx, &existing)?;
        Ok(existing)
    })
}

pub fn delete(db: &Db, id: &str) -> AppResult<()> {
    // Clean up on-disk assets best-effort. Don't fail the delete if any are missing.
    if let Ok(profile) = find_by_id(db, id) {
        if let Some(p) = &profile.logo_path {
            let _ = file_system::delete_file(Path::new(p));
        }
        if let Some(p) = &profile.qr_path {
            let _ = file_system::delete_file(Path::new(p));
        }
    }
    db.transaction(|tx| repository::delete(tx, id))
}

fn is_empty_opt(o: &Option<String>) -> bool {
    o.as_ref().map(|s| s.trim().is_empty()).unwrap_or(true)
}

fn validate_basic(
    entity_type: EntityType,
    name: &str,
    ssm_no: &Option<String>,
    nric: &Option<String>,
    default_tax_rate: Option<f64>,
    default_quotation_valid_days: i32,
    default_invoice_due_days: i32,
) -> AppResult<()> {
    if name.trim().is_empty() {
        let label = match entity_type {
            EntityType::Company => "公司名",
            EntityType::Individual => "姓名",
        };
        return Err(AppError::Validation(format!("{label} 不能为空")));
    }
    match entity_type {
        EntityType::Company => {
            if is_empty_opt(ssm_no) {
                return Err(AppError::Validation(
                    "Company 类型必须填写 SSM 号".into(),
                ));
            }
        }
        EntityType::Individual => {
            if is_empty_opt(nric) {
                return Err(AppError::Validation(
                    "Individual 类型必须填写 NRIC".into(),
                ));
            }
        }
    }
    if default_quotation_valid_days < 0 {
        return Err(AppError::Validation(
            "default_quotation_valid_days 不能为负数".into(),
        ));
    }
    if default_invoice_due_days < 0 {
        return Err(AppError::Validation(
            "default_invoice_due_days 不能为负数".into(),
        ));
    }
    if let Some(rate) = default_tax_rate {
        if !(0.0..=1.0).contains(&rate) {
            return Err(AppError::Validation(
                "default_tax_rate 必须在 0.0 ~ 1.0 之间".into(),
            ));
        }
    }
    Ok(())
}

fn save_asset(
    data_dir: &Path,
    profile_id: &str,
    basename: &str,
    bytes_b64: &str,
    ext: &str,
    max_bytes: usize,
) -> AppResult<String> {
    let ext = ext.trim().trim_start_matches('.').to_lowercase();
    if ext.is_empty() || ext.len() > 8 || !ext.chars().all(|c| c.is_ascii_alphanumeric()) {
        return Err(AppError::Validation(format!("扩展名无效：{ext}")));
    }
    let bytes = B64
        .decode(bytes_b64.as_bytes())
        .map_err(|e| AppError::Validation(format!("base64 解码失败：{e}")))?;
    if bytes.is_empty() {
        return Err(AppError::Validation("文件内容为空".into()));
    }
    if bytes.len() > max_bytes {
        return Err(AppError::Validation(format!(
            "文件过大：{:.1} MB（上限 {:.1} MB）",
            bytes.len() as f64 / 1_048_576.0,
            max_bytes as f64 / 1_048_576.0,
        )));
    }
    let assets_dir = data_dir.join("assets").join(profile_id);
    file_system::ensure_dir(&assets_dir)?;
    // Clean up older versions of the same asset (different ext).
    if let Ok(entries) = std::fs::read_dir(&assets_dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.file_stem().and_then(|s| s.to_str()) == Some(basename) {
                let _ = file_system::delete_file(&p);
            }
        }
    }
    let target: PathBuf = assets_dir.join(format!("{basename}.{ext}"));
    file_system::write_bytes(&target, &bytes)?;
    Ok(target.to_string_lossy().to_string())
}

pub fn set_logo(
    db: &Db,
    data_dir: &Path,
    profile_id: &str,
    bytes_b64: &str,
    ext: &str,
) -> AppResult<BusinessProfile> {
    let path = save_asset(data_dir, profile_id, "logo", bytes_b64, ext, MAX_LOGO_BYTES)?;
    db.transaction(|tx| repository::set_logo_path(tx, profile_id, Some(&path)))?;
    find_by_id(db, profile_id)
}

pub fn clear_logo(db: &Db, profile_id: &str) -> AppResult<BusinessProfile> {
    let current = find_by_id(db, profile_id)?;
    if let Some(p) = &current.logo_path {
        let _ = file_system::delete_file(Path::new(p));
    }
    db.transaction(|tx| repository::set_logo_path(tx, profile_id, None))?;
    find_by_id(db, profile_id)
}

pub fn set_qr(
    db: &Db,
    data_dir: &Path,
    profile_id: &str,
    bytes_b64: &str,
    ext: &str,
) -> AppResult<BusinessProfile> {
    let path = save_asset(data_dir, profile_id, "qr", bytes_b64, ext, MAX_QR_BYTES)?;
    db.transaction(|tx| repository::set_qr_path(tx, profile_id, Some(&path)))?;
    find_by_id(db, profile_id)
}

pub fn clear_qr(db: &Db, profile_id: &str) -> AppResult<BusinessProfile> {
    let current = find_by_id(db, profile_id)?;
    if let Some(p) = &current.qr_path {
        let _ = file_system::delete_file(Path::new(p));
    }
    db.transaction(|tx| repository::set_qr_path(tx, profile_id, None))?;
    find_by_id(db, profile_id)
}

/// Add a typed QR (image written under `<data_dir>/assets/<profile_id>/qrs/<qr_id>.<ext>`).
pub fn add_qr(
    db: &Db,
    data_dir: &Path,
    profile_id: &str,
    kind: QrKind,
    label: &str,
    bytes_b64: &str,
    ext: &str,
) -> AppResult<BusinessProfile> {
    let qr_id = Uuid::new_v4().to_string();
    let path = save_qr_image(data_dir, profile_id, &qr_id, bytes_b64, ext)?;
    let qr = Qr {
        id: qr_id,
        kind,
        label: label.trim().to_string(),
        file_path: path,
    };
    db.transaction(|tx| {
        let mut profile = repository::find_by_id(tx, profile_id)?.ok_or_else(|| {
            AppError::NotFound {
                entity: "business_profile".into(),
                id: profile_id.into(),
            }
        })?;
        profile.qrs.push(qr);
        profile.updated_at = now();
        repository::update(tx, &profile)?;
        Ok(profile)
    })
}

pub fn remove_qr(db: &Db, profile_id: &str, qr_id: &str) -> AppResult<BusinessProfile> {
    db.transaction(|tx| {
        let mut profile = repository::find_by_id(tx, profile_id)?.ok_or_else(|| {
            AppError::NotFound {
                entity: "business_profile".into(),
                id: profile_id.into(),
            }
        })?;
        let removed = profile.qrs.iter().find(|q| q.id == qr_id).cloned();
        profile.qrs.retain(|q| q.id != qr_id);
        profile.updated_at = now();
        repository::update(tx, &profile)?;
        if let Some(q) = removed {
            let _ = file_system::delete_file(Path::new(&q.file_path));
        }
        Ok(profile)
    })
}

pub fn update_qr_label(
    db: &Db,
    profile_id: &str,
    qr_id: &str,
    label: &str,
) -> AppResult<BusinessProfile> {
    db.transaction(|tx| {
        let mut profile = repository::find_by_id(tx, profile_id)?.ok_or_else(|| {
            AppError::NotFound {
                entity: "business_profile".into(),
                id: profile_id.into(),
            }
        })?;
        let found = profile.qrs.iter_mut().find(|q| q.id == qr_id);
        let Some(q) = found else {
            return Err(AppError::NotFound {
                entity: "qr".into(),
                id: qr_id.into(),
            });
        };
        q.label = label.trim().to_string();
        profile.updated_at = now();
        repository::update(tx, &profile)?;
        Ok(profile)
    })
}

fn save_qr_image(
    data_dir: &Path,
    profile_id: &str,
    qr_id: &str,
    bytes_b64: &str,
    ext: &str,
) -> AppResult<String> {
    let ext = ext.trim().trim_start_matches('.').to_lowercase();
    if ext.is_empty() || ext.len() > 8 || !ext.chars().all(|c| c.is_ascii_alphanumeric()) {
        return Err(AppError::Validation(format!("扩展名无效：{ext}")));
    }
    let bytes = B64
        .decode(bytes_b64.as_bytes())
        .map_err(|e| AppError::Validation(format!("base64 解码失败：{e}")))?;
    if bytes.is_empty() {
        return Err(AppError::Validation("文件内容为空".into()));
    }
    if bytes.len() > MAX_QR_BYTES {
        return Err(AppError::Validation(format!(
            "文件过大：{:.1} MB（上限 {:.1} MB）",
            bytes.len() as f64 / 1_048_576.0,
            MAX_QR_BYTES as f64 / 1_048_576.0,
        )));
    }
    let qrs_dir = data_dir.join("assets").join(profile_id).join("qrs");
    file_system::ensure_dir(&qrs_dir)?;
    let target: PathBuf = qrs_dir.join(format!("{qr_id}.{ext}"));
    file_system::write_bytes(&target, &bytes)?;
    Ok(target.to_string_lossy().to_string())
}

/// Read logo + each QR image from disk and return them as `data:image/...;base64,...`
/// URLs. Called once when the frontend opens a form so it can render thumbnails
/// + live preview without paying file I/O on every keystroke.
pub fn get_asset_data_urls(db: &Db, profile_id: &str) -> AppResult<ProfileAssetDataUrls> {
    let profile = find_by_id(db, profile_id)?;
    let logo_data_url = profile.logo_path.as_deref().and_then(file_to_data_url);
    let qrs = profile
        .qrs
        .iter()
        .map(|q| QrDataUrl {
            id: q.id.clone(),
            kind: q.kind,
            label: q.label.clone(),
            data_url: file_to_data_url(&q.file_path).unwrap_or_default(),
        })
        .collect();
    Ok(ProfileAssetDataUrls {
        logo_data_url,
        qrs,
    })
}

fn file_to_data_url(path: &str) -> Option<String> {
    let p = Path::new(path);
    let bytes = std::fs::read(p).ok()?;
    if bytes.is_empty() {
        return None;
    }
    let ext = p
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("png")
        .to_lowercase();
    let mime = match ext.as_str() {
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "svg" => "image/svg+xml",
        _ => "application/octet-stream",
    };
    Some(format!("data:{mime};base64,{}", B64.encode(&bytes)))
}

/// Make sure every BankAccount has a stable UUID. New entries from the UI
/// carry an empty id (serde default); we fill in a fresh UUID before save.
fn ensure_bank_account_ids(accounts: Vec<BankAccount>) -> Vec<BankAccount> {
    accounts
        .into_iter()
        .map(|mut a| {
            if a.id.trim().is_empty() {
                a.id = Uuid::new_v4().to_string();
            }
            a
        })
        .collect()
}

fn trim_opt(v: Option<String>) -> Option<String> {
    v.and_then(|s| {
        let t = s.trim();
        if t.is_empty() { None } else { Some(t.to_string()) }
    })
}
