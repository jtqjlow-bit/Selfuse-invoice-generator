use tempfile::tempdir;

use crate::error::AppError;
use crate::infra::Db;

use super::service;
use super::types::{PdfDocType, PdfTemplateType, UploadCustomTemplateInput};

fn fresh_db() -> (tempfile::TempDir, Db) {
    let dir = tempdir().unwrap();
    let db = Db::open(dir.path().join("test.db")).unwrap();
    db.run_migrations().unwrap();
    (dir, db)
}

#[test]
fn migration_seeds_three_presets() {
    let (_d, db) = fresh_db();
    let all = service::list(&db).unwrap();
    assert_eq!(all.len(), 3);
    assert!(all.iter().all(|t| t.type_ == PdfTemplateType::Preset));

    let doc_types: Vec<_> = all.iter().map(|t| t.doc_type).collect();
    assert!(doc_types.contains(&PdfDocType::Quotation));
    assert!(doc_types.contains(&PdfDocType::Invoice));
    assert!(doc_types.contains(&PdfDocType::PaymentVoucher));
}

#[test]
fn list_by_doc_type_returns_only_matching() {
    let (_d, db) = fresh_db();
    let q = service::list_by_doc_type(&db, PdfDocType::Quotation).unwrap();
    assert_eq!(q.len(), 1);
    assert_eq!(q[0].doc_type, PdfDocType::Quotation);
}

#[test]
fn get_renderable_returns_preset_html() {
    let (_d, db) = fresh_db();
    let html = service::get_renderable(&db, "preset-invoice-default").unwrap();
    assert!(html.contains("<html"));
    assert!(html.contains("Invoice"));
}

#[test]
fn upload_custom_writes_file_and_inserts_row() {
    let (_d, db) = fresh_db();
    let dir = tempdir().unwrap();
    let t = service::upload_custom(
        &db,
        dir.path(),
        UploadCustomTemplateInput {
            doc_type: PdfDocType::Invoice,
            name: "我的发票模板".into(),
            html_content: "<!doctype html><html><body>hi</body></html>".into(),
        },
    )
    .unwrap();
    assert_eq!(t.type_, PdfTemplateType::Custom);
    assert_eq!(t.doc_type, PdfDocType::Invoice);
    assert!(std::path::Path::new(&t.file_path).exists());

    let reread = service::get_renderable(&db, &t.id).unwrap();
    assert!(reread.contains("hi"));
}

#[test]
fn upload_custom_rejects_empty_name() {
    let (_d, db) = fresh_db();
    let dir = tempdir().unwrap();
    let err = service::upload_custom(
        &db,
        dir.path(),
        UploadCustomTemplateInput {
            doc_type: PdfDocType::Invoice,
            name: "   ".into(),
            html_content: "<html></html>".into(),
        },
    )
    .unwrap_err();
    assert!(matches!(err, AppError::Validation(_)));
}

#[test]
fn upload_custom_rejects_non_html_content() {
    let (_d, db) = fresh_db();
    let dir = tempdir().unwrap();
    let err = service::upload_custom(
        &db,
        dir.path(),
        UploadCustomTemplateInput {
            doc_type: PdfDocType::Invoice,
            name: "Bad".into(),
            html_content: "this is just text, not html".into(),
        },
    )
    .unwrap_err();
    assert!(matches!(err, AppError::Validation(_)));
}

#[test]
fn upload_custom_rejects_empty_html() {
    let (_d, db) = fresh_db();
    let dir = tempdir().unwrap();
    let err = service::upload_custom(
        &db,
        dir.path(),
        UploadCustomTemplateInput {
            doc_type: PdfDocType::Invoice,
            name: "Empty".into(),
            html_content: "   ".into(),
        },
    )
    .unwrap_err();
    assert!(matches!(err, AppError::Validation(_)));
}

#[test]
fn delete_custom_removes_row_and_file() {
    let (_d, db) = fresh_db();
    let dir = tempdir().unwrap();
    let t = service::upload_custom(
        &db,
        dir.path(),
        UploadCustomTemplateInput {
            doc_type: PdfDocType::Quotation,
            name: "tmp".into(),
            html_content: "<html></html>".into(),
        },
    )
    .unwrap();
    let path = t.file_path.clone();
    assert!(std::path::Path::new(&path).exists());

    service::delete_custom(&db, &t.id).unwrap();
    assert!(!std::path::Path::new(&path).exists());

    let err = service::find_by_id(&db, &t.id).unwrap_err();
    assert!(matches!(err, AppError::NotFound { .. }));
}

#[test]
fn delete_custom_rejects_preset() {
    let (_d, db) = fresh_db();
    let err = service::delete_custom(&db, "preset-invoice-default").unwrap_err();
    assert!(matches!(err, AppError::Validation(_)));
    // Preset row should still exist.
    let still_there = service::find_by_id(&db, "preset-invoice-default").unwrap();
    assert_eq!(still_there.type_, PdfTemplateType::Preset);
}
