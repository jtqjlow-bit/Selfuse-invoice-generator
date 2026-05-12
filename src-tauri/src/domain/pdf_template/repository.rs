use rusqlite::{params, Connection, OptionalExtension, Row};

use crate::error::{AppError, AppResult};

use super::types::{PdfDocType, PdfTemplate, PdfTemplateType};

fn map_row(row: &Row<'_>) -> rusqlite::Result<PdfTemplate> {
    let doc_type_str: String = row.get("doc_type")?;
    let doc_type = PdfDocType::from_str(&doc_type_str).ok_or_else(|| {
        rusqlite::Error::FromSqlConversionFailure(
            0,
            rusqlite::types::Type::Text,
            format!("unknown doc_type {doc_type_str}").into(),
        )
    })?;
    let type_str: String = row.get("type_")?;
    let type_ = PdfTemplateType::from_str(&type_str).ok_or_else(|| {
        rusqlite::Error::FromSqlConversionFailure(
            0,
            rusqlite::types::Type::Text,
            format!("unknown template type {type_str}").into(),
        )
    })?;
    let config_str: String = row.get("config_json")?;
    let config_json: serde_json::Value = serde_json::from_str(&config_str).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
    })?;
    Ok(PdfTemplate {
        id: row.get("id")?,
        doc_type,
        name: row.get("name")?,
        type_,
        file_path: row.get("file_path")?,
        config_json,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
    })
}

pub fn insert(conn: &Connection, t: &PdfTemplate) -> AppResult<()> {
    let config_str = serde_json::to_string(&t.config_json)
        .map_err(|e| AppError::Internal(format!("serialize config_json: {e}")))?;
    conn.execute(
        "INSERT INTO pdf_template (
            id, doc_type, name, type_, file_path, config_json, created_at, updated_at
        ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
        params![
            t.id,
            t.doc_type.as_str(),
            t.name,
            t.type_.as_str(),
            t.file_path,
            config_str,
            t.created_at,
            t.updated_at,
        ],
    )?;
    Ok(())
}

pub fn delete(conn: &Connection, id: &str) -> AppResult<()> {
    let affected = conn.execute("DELETE FROM pdf_template WHERE id = ?1", params![id])?;
    if affected == 0 {
        return Err(AppError::NotFound {
            entity: "pdf_template".into(),
            id: id.into(),
        });
    }
    Ok(())
}

pub fn find_by_id(conn: &Connection, id: &str) -> AppResult<Option<PdfTemplate>> {
    Ok(conn
        .query_row(
            "SELECT * FROM pdf_template WHERE id = ?1",
            params![id],
            map_row,
        )
        .optional()?)
}

pub fn list(conn: &Connection) -> AppResult<Vec<PdfTemplate>> {
    let mut s = conn.prepare(
        "SELECT * FROM pdf_template
         ORDER BY doc_type ASC, type_ ASC, name COLLATE NOCASE ASC",
    )?;
    let rows = s.query_map([], map_row)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

pub fn list_by_doc_type(conn: &Connection, doc_type: PdfDocType) -> AppResult<Vec<PdfTemplate>> {
    let mut s = conn.prepare(
        "SELECT * FROM pdf_template
         WHERE doc_type = ?1
         ORDER BY type_ ASC, name COLLATE NOCASE ASC",
    )?;
    let rows = s.query_map(params![doc_type.as_str()], map_row)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}
