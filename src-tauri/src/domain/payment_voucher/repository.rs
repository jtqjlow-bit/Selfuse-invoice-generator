use rusqlite::{params, Connection, OptionalExtension, Row};

use crate::error::{AppError, AppResult};

use super::types::PaymentVoucher;

fn map_row(row: &Row<'_>) -> rusqlite::Result<PaymentVoucher> {
    let snapshot_str: String = row.get("customer_snapshot")?;
    let snapshot: serde_json::Value = serde_json::from_str(&snapshot_str).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
    })?;
    Ok(PaymentVoucher {
        id: row.get("id")?,
        number: row.get("number")?,
        invoice_id: row.get::<_, Option<String>>("invoice_id")?,
        customer_id: row.get("customer_id")?,
        customer_snapshot: snapshot,
        business_profile_id: row.get::<_, Option<String>>("business_profile_id")?,
        date: row.get("date")?,
        amount: row.get("amount")?,
        currency: row.get("currency")?,
        payment_method: row.get("payment_method")?,
        notes: row.get("notes")?,
        created_at: row.get("created_at")?,
    })
}

pub fn insert(conn: &Connection, pv: &PaymentVoucher) -> AppResult<()> {
    let snapshot_str = serde_json::to_string(&pv.customer_snapshot)
        .map_err(|e| AppError::Internal(format!("serialize snapshot: {e}")))?;
    conn.execute(
        "INSERT INTO payment_voucher (
            id, number, invoice_id, customer_id, customer_snapshot, business_profile_id,
            date, amount, currency, payment_method, notes, created_at
        ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)",
        params![
            pv.id,
            pv.number,
            pv.invoice_id,
            pv.customer_id,
            snapshot_str,
            pv.business_profile_id,
            pv.date,
            pv.amount,
            pv.currency,
            pv.payment_method,
            pv.notes,
            pv.created_at,
        ],
    )?;
    Ok(())
}

pub fn update(conn: &Connection, pv: &PaymentVoucher) -> AppResult<()> {
    let affected = conn.execute(
        "UPDATE payment_voucher SET
            date = ?2, amount = ?3, payment_method = ?4, notes = ?5
        WHERE id = ?1",
        params![pv.id, pv.date, pv.amount, pv.payment_method, pv.notes],
    )?;
    if affected == 0 {
        return Err(AppError::NotFound {
            entity: "payment_voucher".into(),
            id: pv.id.clone(),
        });
    }
    Ok(())
}

pub fn delete(conn: &Connection, id: &str) -> AppResult<()> {
    let affected = conn.execute("DELETE FROM payment_voucher WHERE id = ?1", params![id])?;
    if affected == 0 {
        return Err(AppError::NotFound {
            entity: "payment_voucher".into(),
            id: id.into(),
        });
    }
    Ok(())
}

pub fn find_by_id(conn: &Connection, id: &str) -> AppResult<Option<PaymentVoucher>> {
    Ok(conn
        .query_row(
            "SELECT * FROM payment_voucher WHERE id = ?1",
            params![id],
            map_row,
        )
        .optional()?)
}

pub fn list(conn: &Connection) -> AppResult<Vec<PaymentVoucher>> {
    let mut s = conn.prepare("SELECT * FROM payment_voucher ORDER BY date DESC, number DESC")?;
    let rows = s.query_map([], map_row)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

pub fn list_by_invoice(conn: &Connection, invoice_id: &str) -> AppResult<Vec<PaymentVoucher>> {
    let mut s = conn.prepare(
        "SELECT * FROM payment_voucher WHERE invoice_id = ?1 ORDER BY date ASC, number ASC",
    )?;
    let rows = s.query_map(params![invoice_id], map_row)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

pub fn list_by_customer(conn: &Connection, customer_id: &str) -> AppResult<Vec<PaymentVoucher>> {
    let mut s = conn.prepare(
        "SELECT * FROM payment_voucher WHERE customer_id = ?1 ORDER BY date DESC, number DESC",
    )?;
    let rows = s.query_map(params![customer_id], map_row)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

pub fn sum_by_invoice(conn: &Connection, invoice_id: &str) -> AppResult<f64> {
    let v: Option<f64> = conn
        .query_row(
            "SELECT COALESCE(SUM(amount), 0) FROM payment_voucher WHERE invoice_id = ?1",
            params![invoice_id],
            |r| r.get(0),
        )
        .optional()?;
    Ok(v.unwrap_or(0.0))
}
