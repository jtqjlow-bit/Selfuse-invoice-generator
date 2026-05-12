use rusqlite::{params, Connection, OptionalExtension, Row};

use crate::error::{AppError, AppResult};

use super::state_machine::InvoiceStatus;
use super::types::{Invoice, InvoiceLineItem};

fn map_invoice(row: &Row<'_>) -> rusqlite::Result<Invoice> {
    let status_str: String = row.get("status")?;
    let status = InvoiceStatus::from_str(&status_str).ok_or_else(|| {
        rusqlite::Error::FromSqlConversionFailure(
            0,
            rusqlite::types::Type::Text,
            format!("unknown status {status_str}").into(),
        )
    })?;
    let snapshot_str: String = row.get("customer_snapshot")?;
    let customer_snapshot: serde_json::Value =
        serde_json::from_str(&snapshot_str).map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
        })?;
    let pm_str: String = row.get("payment_methods_snapshot")?;
    let payment_methods_snapshot: serde_json::Value =
        serde_json::from_str(&pm_str).map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
        })?;
    let sel_ba_str: String = row.get("selected_bank_account_ids")?;
    let selected_bank_account_ids: Vec<String> = serde_json::from_str(&sel_ba_str)
        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e)))?;
    let sel_qr_str: String = row.get("selected_qr_ids")?;
    let selected_qr_ids: Vec<String> = serde_json::from_str(&sel_qr_str)
        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e)))?;
    let sel_sm_str: String = row.get("selected_static_methods")?;
    let selected_static_methods: Vec<String> = serde_json::from_str(&sel_sm_str)
        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e)))?;
    let tax_enabled_int: i64 = row.get("tax_enabled")?;

    Ok(Invoice {
        id: row.get("id")?,
        number: row.get("number")?,
        customer_id: row.get("customer_id")?,
        customer_snapshot,
        business_profile_id: row.get::<_, Option<String>>("business_profile_id")?,
        source_quotation_id: row.get("source_quotation_id")?,
        issue_date: row.get("issue_date")?,
        due_date: row.get("due_date")?,
        currency: row.get("currency")?,
        tax_enabled: tax_enabled_int != 0,
        tax_rate: row.get("tax_rate")?,
        subtotal: row.get("subtotal")?,
        tax_amount: row.get("tax_amount")?,
        total: row.get("total")?,
        paid_amount: row.get("paid_amount")?,
        payment_methods_snapshot,
        selected_bank_account_ids,
        selected_qr_ids,
        selected_static_methods,
        status,
        notes: row.get("notes")?,
        terms: row.get("terms")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
    })
}

fn map_line(row: &Row<'_>) -> rusqlite::Result<InvoiceLineItem> {
    Ok(InvoiceLineItem {
        id: row.get("id")?,
        invoice_id: row.get("invoice_id")?,
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

pub fn insert_invoice(conn: &Connection, inv: &Invoice) -> AppResult<()> {
    let snapshot_str = serde_json::to_string(&inv.customer_snapshot)
        .map_err(|e| AppError::Internal(format!("serialize snapshot: {e}")))?;
    let pm_str = serde_json::to_string(&inv.payment_methods_snapshot)
        .map_err(|e| AppError::Internal(format!("serialize payment methods: {e}")))?;
    let sel_ba_str = serde_json::to_string(&inv.selected_bank_account_ids)
        .map_err(|e| AppError::Internal(format!("serialize selected_bank_account_ids: {e}")))?;
    let sel_qr_str = serde_json::to_string(&inv.selected_qr_ids)
        .map_err(|e| AppError::Internal(format!("serialize selected_qr_ids: {e}")))?;
    let sel_sm_str = serde_json::to_string(&inv.selected_static_methods)
        .map_err(|e| AppError::Internal(format!("serialize selected_static_methods: {e}")))?;
    conn.execute(
        "INSERT INTO invoice (
            id, number, customer_id, customer_snapshot, business_profile_id,
            source_quotation_id, issue_date, due_date, currency, tax_enabled, tax_rate,
            subtotal, tax_amount, total, paid_amount, payment_methods_snapshot,
            selected_bank_account_ids, selected_qr_ids, selected_static_methods,
            status, notes, terms, created_at, updated_at
        ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,?20,?21,?22,?23,?24)",
        params![
            inv.id,
            inv.number,
            inv.customer_id,
            snapshot_str,
            inv.business_profile_id,
            inv.source_quotation_id,
            inv.issue_date,
            inv.due_date,
            inv.currency,
            inv.tax_enabled as i64,
            inv.tax_rate,
            inv.subtotal,
            inv.tax_amount,
            inv.total,
            inv.paid_amount,
            pm_str,
            sel_ba_str,
            sel_qr_str,
            sel_sm_str,
            inv.status.as_str(),
            inv.notes,
            inv.terms,
            inv.created_at,
            inv.updated_at,
        ],
    )?;
    Ok(())
}

pub fn insert_line(conn: &Connection, line: &InvoiceLineItem) -> AppResult<()> {
    conn.execute(
        "INSERT INTO invoice_line_item (
            id, invoice_id, position, description, quantity, unit_price, line_total,
            line_currency, exchange_rate_to_doc_currency, tax_rate, discount_rate
        ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)",
        params![
            line.id,
            line.invoice_id,
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

pub fn update_header(conn: &Connection, inv: &Invoice) -> AppResult<()> {
    let snapshot_str = serde_json::to_string(&inv.customer_snapshot)
        .map_err(|e| AppError::Internal(format!("serialize snapshot: {e}")))?;
    let pm_str = serde_json::to_string(&inv.payment_methods_snapshot)
        .map_err(|e| AppError::Internal(format!("serialize payment methods: {e}")))?;
    let sel_ba_str = serde_json::to_string(&inv.selected_bank_account_ids)
        .map_err(|e| AppError::Internal(format!("serialize selected_bank_account_ids: {e}")))?;
    let sel_qr_str = serde_json::to_string(&inv.selected_qr_ids)
        .map_err(|e| AppError::Internal(format!("serialize selected_qr_ids: {e}")))?;
    let sel_sm_str = serde_json::to_string(&inv.selected_static_methods)
        .map_err(|e| AppError::Internal(format!("serialize selected_static_methods: {e}")))?;
    let affected = conn.execute(
        "UPDATE invoice SET
            customer_id = ?2, customer_snapshot = ?3, business_profile_id = ?4,
            issue_date = ?5, due_date = ?6, currency = ?7, tax_enabled = ?8, tax_rate = ?9,
            subtotal = ?10, tax_amount = ?11, total = ?12, payment_methods_snapshot = ?13,
            selected_bank_account_ids = ?14, selected_qr_ids = ?15, selected_static_methods = ?16,
            notes = ?17, terms = ?18, updated_at = ?19
        WHERE id = ?1",
        params![
            inv.id,
            inv.customer_id,
            snapshot_str,
            inv.business_profile_id,
            inv.issue_date,
            inv.due_date,
            inv.currency,
            inv.tax_enabled as i64,
            inv.tax_rate,
            inv.subtotal,
            inv.tax_amount,
            inv.total,
            pm_str,
            sel_ba_str,
            sel_qr_str,
            sel_sm_str,
            inv.notes,
            inv.terms,
            inv.updated_at,
        ],
    )?;
    if affected == 0 {
        return Err(AppError::NotFound {
            entity: "invoice".into(),
            id: inv.id.clone(),
        });
    }
    Ok(())
}

pub fn update_status(
    conn: &Connection,
    id: &str,
    status: InvoiceStatus,
    now: &str,
) -> AppResult<()> {
    let affected = conn.execute(
        "UPDATE invoice SET status = ?1, updated_at = ?2 WHERE id = ?3",
        params![status.as_str(), now, id],
    )?;
    if affected == 0 {
        return Err(AppError::NotFound {
            entity: "invoice".into(),
            id: id.into(),
        });
    }
    Ok(())
}

pub fn update_paid_amount(
    conn: &Connection,
    id: &str,
    paid: f64,
    now: &str,
) -> AppResult<()> {
    let affected = conn.execute(
        "UPDATE invoice SET paid_amount = ?1, updated_at = ?2 WHERE id = ?3",
        params![paid, now, id],
    )?;
    if affected == 0 {
        return Err(AppError::NotFound {
            entity: "invoice".into(),
            id: id.into(),
        });
    }
    Ok(())
}

pub fn delete_lines_for(conn: &Connection, invoice_id: &str) -> AppResult<()> {
    conn.execute(
        "DELETE FROM invoice_line_item WHERE invoice_id = ?1",
        params![invoice_id],
    )?;
    Ok(())
}

pub fn find_invoice(conn: &Connection, id: &str) -> AppResult<Option<Invoice>> {
    Ok(conn
        .query_row("SELECT * FROM invoice WHERE id = ?1", params![id], map_invoice)
        .optional()?)
}

pub fn list_lines(conn: &Connection, invoice_id: &str) -> AppResult<Vec<InvoiceLineItem>> {
    let mut s = conn.prepare(
        "SELECT * FROM invoice_line_item WHERE invoice_id = ?1 ORDER BY position ASC",
    )?;
    let rows = s.query_map(params![invoice_id], map_line)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

pub fn list(conn: &Connection) -> AppResult<Vec<Invoice>> {
    let mut s = conn.prepare("SELECT * FROM invoice ORDER BY issue_date DESC, number DESC")?;
    let rows = s.query_map([], map_invoice)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

pub fn list_by_customer(conn: &Connection, customer_id: &str) -> AppResult<Vec<Invoice>> {
    let mut s = conn.prepare(
        "SELECT * FROM invoice WHERE customer_id = ?1 ORDER BY issue_date DESC, number DESC",
    )?;
    let rows = s.query_map(params![customer_id], map_invoice)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

/// Returns IDs of invoices that are still in Sent or PartialPaid and whose due_date < today.
pub fn list_overdue_candidates(conn: &Connection, today: &str) -> AppResult<Vec<String>> {
    let mut s = conn.prepare(
        "SELECT id FROM invoice
         WHERE due_date < ?1 AND status IN ('Sent', 'PartialPaid')",
    )?;
    let rows = s.query_map(params![today], |r| r.get::<_, String>(0))?;
    rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}
