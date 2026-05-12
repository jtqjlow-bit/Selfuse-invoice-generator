use rusqlite::{params, Connection, OptionalExtension, Row};

use crate::error::{AppError, AppResult};

use super::state_machine::QuotationStatus;
use super::types::{Quotation, QuotationLineItem};

fn map_quotation(row: &Row<'_>) -> rusqlite::Result<Quotation> {
    let status_str: String = row.get("status")?;
    let status = QuotationStatus::from_str(&status_str).ok_or_else(|| {
        rusqlite::Error::FromSqlConversionFailure(
            0,
            rusqlite::types::Type::Text,
            format!("unknown status {status_str}").into(),
        )
    })?;
    let snapshot_str: String = row.get("customer_snapshot")?;
    let snapshot: serde_json::Value = serde_json::from_str(&snapshot_str).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(
            0,
            rusqlite::types::Type::Text,
            Box::new(e),
        )
    })?;
    let tax_enabled_int: i64 = row.get("tax_enabled")?;

    Ok(Quotation {
        id: row.get("id")?,
        number: row.get("number")?,
        customer_id: row.get("customer_id")?,
        customer_snapshot: snapshot,
        business_profile_id: row.get::<_, Option<String>>("business_profile_id")?,
        issue_date: row.get("issue_date")?,
        valid_until: row.get("valid_until")?,
        currency: row.get("currency")?,
        tax_enabled: tax_enabled_int != 0,
        tax_rate: row.get("tax_rate")?,
        subtotal: row.get("subtotal")?,
        tax_amount: row.get("tax_amount")?,
        total: row.get("total")?,
        status,
        converted_invoice_id: row.get("converted_invoice_id")?,
        notes: row.get("notes")?,
        terms: row.get("terms")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
    })
}

fn map_line(row: &Row<'_>) -> rusqlite::Result<QuotationLineItem> {
    Ok(QuotationLineItem {
        id: row.get("id")?,
        quotation_id: row.get("quotation_id")?,
        position: row.get("position")?,
        description: row.get("description")?,
        quantity: row.get("quantity")?,
        unit_price: row.get("unit_price")?,
        line_total: row.get("line_total")?,
        line_currency: row.get("line_currency")?,
        exchange_rate_to_doc_currency: row.get("exchange_rate_to_doc_currency")?,
        tax_rate: row.get("tax_rate")?,
        discount_rate: row.get("discount_rate")?,
    })
}

pub fn insert_quotation(conn: &Connection, q: &Quotation) -> AppResult<()> {
    let snapshot_str = serde_json::to_string(&q.customer_snapshot)
        .map_err(|e| AppError::Internal(format!("serialize snapshot: {e}")))?;
    conn.execute(
        "INSERT INTO quotation (
            id, number, customer_id, customer_snapshot, business_profile_id, issue_date,
            valid_until, currency, tax_enabled, tax_rate, subtotal, tax_amount, total, status,
            converted_invoice_id, notes, terms, created_at, updated_at
        ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19)",
        params![
            q.id,
            q.number,
            q.customer_id,
            snapshot_str,
            q.business_profile_id,
            q.issue_date,
            q.valid_until,
            q.currency,
            q.tax_enabled as i64,
            q.tax_rate,
            q.subtotal,
            q.tax_amount,
            q.total,
            q.status.as_str(),
            q.converted_invoice_id,
            q.notes,
            q.terms,
            q.created_at,
            q.updated_at,
        ],
    )?;
    Ok(())
}

pub fn insert_line(conn: &Connection, line: &QuotationLineItem) -> AppResult<()> {
    conn.execute(
        "INSERT INTO quotation_line_item (
            id, quotation_id, position, description, quantity, unit_price, line_total,
            line_currency, exchange_rate_to_doc_currency, tax_rate, discount_rate
        ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)",
        params![
            line.id,
            line.quotation_id,
            line.position,
            line.description,
            line.quantity,
            line.unit_price,
            line.line_total,
            line.line_currency,
            line.exchange_rate_to_doc_currency,
            line.tax_rate,
            line.discount_rate,
        ],
    )?;
    Ok(())
}

pub fn update_header(conn: &Connection, q: &Quotation) -> AppResult<()> {
    let snapshot_str = serde_json::to_string(&q.customer_snapshot)
        .map_err(|e| AppError::Internal(format!("serialize snapshot: {e}")))?;
    let affected = conn.execute(
        "UPDATE quotation SET
            customer_id = ?2, customer_snapshot = ?3, business_profile_id = ?4,
            issue_date = ?5, valid_until = ?6, currency = ?7, tax_enabled = ?8, tax_rate = ?9,
            subtotal = ?10, tax_amount = ?11, total = ?12, notes = ?13, terms = ?14,
            updated_at = ?15
        WHERE id = ?1",
        params![
            q.id,
            q.customer_id,
            snapshot_str,
            q.business_profile_id,
            q.issue_date,
            q.valid_until,
            q.currency,
            q.tax_enabled as i64,
            q.tax_rate,
            q.subtotal,
            q.tax_amount,
            q.total,
            q.notes,
            q.terms,
            q.updated_at,
        ],
    )?;
    if affected == 0 {
        return Err(AppError::NotFound {
            entity: "quotation".into(),
            id: q.id.clone(),
        });
    }
    Ok(())
}

pub fn update_status(
    conn: &Connection,
    id: &str,
    status: QuotationStatus,
    now: &str,
) -> AppResult<()> {
    let affected = conn.execute(
        "UPDATE quotation SET status = ?1, updated_at = ?2 WHERE id = ?3",
        params![status.as_str(), now, id],
    )?;
    if affected == 0 {
        return Err(AppError::NotFound {
            entity: "quotation".into(),
            id: id.into(),
        });
    }
    Ok(())
}

pub fn set_converted_invoice_id(
    conn: &Connection,
    quotation_id: &str,
    invoice_id: &str,
    now: &str,
) -> AppResult<()> {
    let affected = conn.execute(
        "UPDATE quotation SET converted_invoice_id = ?1, updated_at = ?2 WHERE id = ?3",
        params![invoice_id, now, quotation_id],
    )?;
    if affected == 0 {
        return Err(AppError::NotFound {
            entity: "quotation".into(),
            id: quotation_id.into(),
        });
    }
    Ok(())
}

pub fn delete_lines_for(conn: &Connection, quotation_id: &str) -> AppResult<()> {
    conn.execute(
        "DELETE FROM quotation_line_item WHERE quotation_id = ?1",
        params![quotation_id],
    )?;
    Ok(())
}

pub fn find_quotation(conn: &Connection, id: &str) -> AppResult<Option<Quotation>> {
    Ok(conn
        .query_row("SELECT * FROM quotation WHERE id = ?1", params![id], map_quotation)
        .optional()?)
}

pub fn list_lines(conn: &Connection, quotation_id: &str) -> AppResult<Vec<QuotationLineItem>> {
    let mut s = conn.prepare(
        "SELECT * FROM quotation_line_item WHERE quotation_id = ?1 ORDER BY position ASC",
    )?;
    let rows = s.query_map(params![quotation_id], map_line)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

pub fn list(conn: &Connection) -> AppResult<Vec<Quotation>> {
    let mut s = conn.prepare("SELECT * FROM quotation ORDER BY issue_date DESC, number DESC")?;
    let rows = s.query_map([], map_quotation)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

pub fn list_by_customer(conn: &Connection, customer_id: &str) -> AppResult<Vec<Quotation>> {
    let mut s = conn.prepare(
        "SELECT * FROM quotation WHERE customer_id = ?1 ORDER BY issue_date DESC, number DESC",
    )?;
    let rows = s.query_map(params![customer_id], map_quotation)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}
