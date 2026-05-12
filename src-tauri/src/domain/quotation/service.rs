use rusqlite::Connection;
use uuid::Uuid;

use crate::domain::customer;
use crate::error::{AppError, AppResult};
use crate::infra::Db;
use crate::service::numbering::{self, DocType};
use crate::service::tax_calc::{self, LineForTotals};

use super::repository;
use super::state_machine::{transition, QuotationStatus};
use super::types::{
    CreateQuotationInput, LineItemInput, Quotation, QuotationLineItem, QuotationWithLines,
    UpdateQuotationInput,
};

fn now() -> String {
    chrono::Utc::now().to_rfc3339()
}

fn compute_totals(
    lines: &[LineItemInput],
    tax_enabled: bool,
    tax_rate: Option<f64>,
) -> tax_calc::Totals {
    let mapped: Vec<LineForTotals> = lines
        .iter()
        .map(|l| LineForTotals {
            quantity: l.quantity,
            unit_price: l.unit_price,
        })
        .collect();
    tax_calc::document_totals(&mapped, tax_enabled, tax_rate)
}

fn validate_input(
    issue_date: &str,
    valid_until: &str,
    currency: &str,
    tax_enabled: bool,
    tax_rate: Option<f64>,
    lines: &[LineItemInput],
) -> AppResult<()> {
    if !is_iso_date(issue_date) {
        return Err(AppError::Validation(
            "issue_date 必须是 YYYY-MM-DD 格式".into(),
        ));
    }
    if !is_iso_date(valid_until) {
        return Err(AppError::Validation(
            "valid_until 必须是 YYYY-MM-DD 格式".into(),
        ));
    }
    if currency.trim().is_empty() {
        return Err(AppError::Validation("currency 不能为空".into()));
    }
    if lines.is_empty() {
        return Err(AppError::Validation("至少要有一行项目".into()));
    }
    if tax_enabled {
        let r = tax_rate.unwrap_or(0.0);
        if !(0.0..=1.0).contains(&r) {
            return Err(AppError::Validation(
                "tax_rate 必须在 0.0 ~ 1.0 之间".into(),
            ));
        }
    }
    for (i, l) in lines.iter().enumerate() {
        if l.description.trim().is_empty() {
            return Err(AppError::Validation(format!("第 {} 行描述不能为空", i + 1)));
        }
        if l.quantity <= 0.0 {
            return Err(AppError::Validation(format!(
                "第 {} 行数量必须大于 0",
                i + 1
            )));
        }
        if l.unit_price < 0.0 {
            return Err(AppError::Validation(format!(
                "第 {} 行单价不能为负数",
                i + 1
            )));
        }
    }
    Ok(())
}

fn is_iso_date(s: &str) -> bool {
    chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").is_ok()
}

pub fn create(db: &Db, input: CreateQuotationInput) -> AppResult<QuotationWithLines> {
    validate_input(
        &input.issue_date,
        &input.valid_until,
        &input.currency,
        input.tax_enabled,
        input.tax_rate,
        &input.lines,
    )?;

    let cust = customer::find_by_id(db, &input.customer_id)?;
    let snapshot = serde_json::to_value(&cust)
        .map_err(|e| AppError::Internal(format!("snapshot customer: {e}")))?;
    let totals = compute_totals(&input.lines, input.tax_enabled, input.tax_rate);
    let number = numbering::next(db, DocType::Quotation)?;

    let now_s = now();
    let q_id = Uuid::new_v4().to_string();
    let q = Quotation {
        id: q_id.clone(),
        number,
        customer_id: input.customer_id,
        customer_snapshot: snapshot,
        business_profile_id: input.business_profile_id,
        issue_date: input.issue_date,
        valid_until: input.valid_until,
        currency: input.currency.clone(),
        tax_enabled: input.tax_enabled,
        tax_rate: input.tax_rate,
        subtotal: totals.subtotal,
        tax_amount: totals.tax_amount,
        total: totals.total,
        status: QuotationStatus::Draft,
        converted_invoice_id: None,
        notes: trim_opt(input.notes),
        terms: trim_opt(input.terms),
        created_at: now_s.clone(),
        updated_at: now_s,
    };
    let lines = input
        .lines
        .into_iter()
        .enumerate()
        .map(|(i, l)| QuotationLineItem {
            id: Uuid::new_v4().to_string(),
            quotation_id: q_id.clone(),
            position: (i + 1) as i32,
            description: l.description.trim().to_string(),
            quantity: l.quantity,
            unit_price: l.unit_price,
            line_total: l.quantity * l.unit_price,
            line_currency: input.currency.clone(),
            exchange_rate_to_doc_currency: None,
            tax_rate: None,
            discount_rate: None,
        })
        .collect::<Vec<_>>();

    db.transaction(|tx| {
        repository::insert_quotation(tx, &q)?;
        for l in &lines {
            repository::insert_line(tx, l)?;
        }
        Ok(())
    })?;

    Ok(QuotationWithLines { quotation: q, lines })
}

pub fn update(db: &Db, input: UpdateQuotationInput) -> AppResult<QuotationWithLines> {
    validate_input(
        &input.issue_date,
        &input.valid_until,
        &input.currency,
        input.tax_enabled,
        input.tax_rate,
        &input.lines,
    )?;

    // Re-snapshot customer at edit time (single-user single-process, OK to read outside the tx).
    let cust = customer::find_by_id(db, &input.customer_id)?;
    let snapshot = serde_json::to_value(&cust)
        .map_err(|e| AppError::Internal(format!("snapshot customer: {e}")))?;
    let totals = compute_totals(&input.lines, input.tax_enabled, input.tax_rate);
    let now_s = now();

    let UpdateQuotationInput {
        id,
        customer_id,
        business_profile_id,
        issue_date,
        valid_until,
        currency,
        tax_enabled,
        tax_rate,
        lines: line_inputs,
        notes,
        terms,
    } = input;

    let new_lines: Vec<QuotationLineItem> = line_inputs
        .into_iter()
        .enumerate()
        .map(|(i, l)| QuotationLineItem {
            id: Uuid::new_v4().to_string(),
            quotation_id: id.clone(),
            position: (i + 1) as i32,
            description: l.description.trim().to_string(),
            quantity: l.quantity,
            unit_price: l.unit_price,
            line_total: l.quantity * l.unit_price,
            line_currency: currency.clone(),
            exchange_rate_to_doc_currency: None,
            tax_rate: None,
            discount_rate: None,
        })
        .collect();

    db.transaction(move |tx| {
        let mut q = repository::find_quotation(tx, &id)?.ok_or_else(|| AppError::NotFound {
            entity: "quotation".into(),
            id: id.clone(),
        })?;
        if q.status != QuotationStatus::Draft {
            return Err(AppError::Validation(format!(
                "只有 Draft 状态可以修改，当前状态为 {}",
                q.status.as_str()
            )));
        }

        q.customer_id = customer_id;
        q.customer_snapshot = snapshot;
        q.business_profile_id = business_profile_id;
        q.issue_date = issue_date;
        q.valid_until = valid_until;
        q.currency = currency;
        q.tax_enabled = tax_enabled;
        q.tax_rate = tax_rate;
        q.subtotal = totals.subtotal;
        q.tax_amount = totals.tax_amount;
        q.total = totals.total;
        q.notes = trim_opt(notes);
        q.terms = trim_opt(terms);
        q.updated_at = now_s;

        repository::update_header(tx, &q)?;
        repository::delete_lines_for(tx, &q.id)?;
        for l in &new_lines {
            repository::insert_line(tx, l)?;
        }
        Ok(QuotationWithLines {
            quotation: q,
            lines: new_lines,
        })
    })
}

pub fn find_by_id(db: &Db, id: &str) -> AppResult<QuotationWithLines> {
    db.with_conn(|c| {
        let q = repository::find_quotation(c, id)?.ok_or_else(|| AppError::NotFound {
            entity: "quotation".into(),
            id: id.into(),
        })?;
        let lines = repository::list_lines(c, &q.id)?;
        Ok(QuotationWithLines { quotation: q, lines })
    })
}

pub fn list(db: &Db) -> AppResult<Vec<Quotation>> {
    db.with_conn(|c| repository::list(c))
}

pub fn list_by_customer(db: &Db, customer_id: &str) -> AppResult<Vec<Quotation>> {
    db.with_conn(|c| repository::list_by_customer(c, customer_id))
}

fn transition_to(db: &Db, id: &str, to: QuotationStatus) -> AppResult<Quotation> {
    db.transaction(|tx| {
        let q = repository::find_quotation(tx, id)?.ok_or_else(|| AppError::NotFound {
            entity: "quotation".into(),
            id: id.into(),
        })?;
        let new_status = transition(q.status, to)?;
        repository::update_status(tx, id, new_status, &now())?;
        let updated = repository::find_quotation(tx, id)?.ok_or_else(|| AppError::NotFound {
            entity: "quotation".into(),
            id: id.into(),
        })?;
        Ok(updated)
    })
}

pub fn mark_sent(db: &Db, id: &str) -> AppResult<Quotation> {
    transition_to(db, id, QuotationStatus::Sent)
}

pub fn mark_accepted(db: &Db, id: &str) -> AppResult<Quotation> {
    transition_to(db, id, QuotationStatus::Accepted)
}

pub fn mark_rejected(db: &Db, id: &str) -> AppResult<Quotation> {
    transition_to(db, id, QuotationStatus::Rejected)
}

pub fn mark_expired(db: &Db, id: &str) -> AppResult<Quotation> {
    transition_to(db, id, QuotationStatus::Expired)
}

/// Transaction-aware: called by `domain::invoice` while it creates an invoice
/// from this quotation, inside the invoice creation transaction.
pub fn set_converted_invoice_id_in_tx(
    conn: &Connection,
    quotation_id: &str,
    invoice_id: &str,
) -> AppResult<()> {
    repository::set_converted_invoice_id(conn, quotation_id, invoice_id, &now())
}

fn trim_opt(v: Option<String>) -> Option<String> {
    v.and_then(|s| {
        let t = s.trim();
        if t.is_empty() { None } else { Some(t.to_string()) }
    })
}
