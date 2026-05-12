use std::path::Path;

use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::infra::{file_system, Db};

use super::repository;
use super::types::{PdfDocType, PdfTemplate, PdfTemplateType, UploadCustomTemplateInput};

const PRESET_QUOTATION: &str = include_str!("../../../templates/preset_quotation.html");
const PRESET_INVOICE: &str = include_str!("../../../templates/preset_invoice.html");
const PRESET_PAYMENT_VOUCHER: &str =
    include_str!("../../../templates/preset_payment_voucher.html");

fn now() -> String {
    chrono::Utc::now().to_rfc3339()
}

pub fn list(db: &Db) -> AppResult<Vec<PdfTemplate>> {
    db.with_conn(|c| repository::list(c))
}

pub fn list_by_doc_type(db: &Db, doc_type: PdfDocType) -> AppResult<Vec<PdfTemplate>> {
    db.with_conn(|c| repository::list_by_doc_type(c, doc_type))
}

pub fn find_by_id(db: &Db, id: &str) -> AppResult<PdfTemplate> {
    db.with_conn(|c| {
        repository::find_by_id(c, id)?.ok_or_else(|| AppError::NotFound {
            entity: "pdf_template".into(),
            id: id.into(),
        })
    })
}

/// Load the HTML body (with Tera placeholders) of a template, whether preset
/// or custom-on-disk. The renderer (Slice 7b) calls this to get the text it
/// passes to Tera.
pub fn get_renderable(db: &Db, id: &str) -> AppResult<String> {
    let t = find_by_id(db, id)?;
    if let Some(key) = t.file_path.strip_prefix("preset:") {
        return match key {
            "quotation_default" => Ok(PRESET_QUOTATION.to_string()),
            "invoice_default" => Ok(PRESET_INVOICE.to_string()),
            "pv_default" => Ok(PRESET_PAYMENT_VOUCHER.to_string()),
            other => Err(AppError::Internal(format!(
                "unknown preset template key {other}"
            ))),
        };
    }
    file_system::read_file(&t.file_path)
}

pub fn upload_custom(
    db: &Db,
    data_dir: &Path,
    input: UploadCustomTemplateInput,
) -> AppResult<PdfTemplate> {
    if input.name.trim().is_empty() {
        return Err(AppError::Validation("模板名称不能为空".into()));
    }
    if input.html_content.trim().is_empty() {
        return Err(AppError::Validation("HTML 内容不能为空".into()));
    }
    // Light sanity check: should look like HTML.
    let trimmed = input.html_content.trim_start();
    if !trimmed.starts_with('<') {
        return Err(AppError::Validation(
            "HTML 内容必须以 '<' 开头（看起来不像 HTML 文件）".into(),
        ));
    }

    let templates_dir = data_dir.join("templates");
    file_system::ensure_dir(&templates_dir)?;

    let id = Uuid::new_v4().to_string();
    let file_path = templates_dir.join(format!("{id}.html"));
    file_system::write_file(&file_path, &input.html_content)?;

    let now_s = now();
    let template = PdfTemplate {
        id: id.clone(),
        doc_type: input.doc_type,
        name: input.name.trim().to_string(),
        type_: PdfTemplateType::Custom,
        file_path: file_path.to_string_lossy().to_string(),
        config_json: serde_json::json!({}),
        created_at: now_s.clone(),
        updated_at: now_s,
    };

    db.transaction(|tx| repository::insert(tx, &template))?;
    Ok(template)
}

pub fn delete_custom(db: &Db, id: &str) -> AppResult<()> {
    let existing = find_by_id(db, id)?;
    if existing.type_ != PdfTemplateType::Custom {
        return Err(AppError::Validation(
            "不能删除预设 (Preset) 模板".into(),
        ));
    }
    db.transaction(|tx| repository::delete(tx, id))?;
    // Best-effort file cleanup (don't fail the whole op if the file is already gone).
    let _ = file_system::delete_file(&existing.file_path);
    Ok(())
}
