use rusqlite::Connection;
use uuid::Uuid;

use crate::domain::business_profile;
use crate::domain::customer;
use crate::domain::quotation;
use crate::error::{AppError, AppResult};
use crate::infra::Db;
use crate::service::numbering::{self, DocType};
use crate::service::tax_calc::{self, LineForTotals};

use super::repository;
use super::state_machine::{transition, InvoiceStatus};
use super::types::{
    CreateFromQuotationInput, CreateInvoiceInput, Invoice, InvoiceLineItem, InvoiceWithLines,
    UpdateInvoiceInput,
};
use crate::domain::quotation::LineItemInput;

fn now() -> String {
    chrono::Utc::now().to_rfc3339()
}

fn today_iso() -> String {
    use chrono::{Datelike, Local};
    let n = Local::now();
    format!("{:04}-{:02}-{:02}", n.year(), n.month(), n.day())
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

fn is_iso_date(s: &str) -> bool {
    chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").is_ok()
}

fn validate_input(
    issue_date: &str,
    due_date: &str,
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
    if !is_iso_date(due_date) {
        return Err(AppError::Validation(
            "due_date 必须是 YYYY-MM-DD 格式".into(),
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

fn build_lines(
    line_inputs: Vec<LineItemInput>,
    invoice_id: &str,
    currency: &str,
) -> Vec<InvoiceLineItem> {
    line_inputs
        .into_iter()
        .enumerate()
        .map(|(i, l)| InvoiceLineItem {
            id: Uuid::new_v4().to_string(),
            invoice_id: invoice_id.to_string(),
            position: (i + 1) as i32,
            description: l.description.trim().to_string(),
            quantity: l.quantity,
            unit_price: l.unit_price,
            line_total: l.quantity * l.unit_price,
            line_currency: currency.to_string(),
            exchange_rate_to_doc_currency: None,
            tax_rate: None,
            discount_rate: None,
        })
        .collect()
}

pub fn create(db: &Db, input: CreateInvoiceInput) -> AppResult<InvoiceWithLines> {
    validate_input(
        &input.issue_date,
        &input.due_date,
        &input.currency,
        input.tax_enabled,
        input.tax_rate,
        &input.lines,
    )?;

    let cust = customer::find_by_id(db, &input.customer_id)?;
    let snapshot = serde_json::to_value(&cust)
        .map_err(|e| AppError::Internal(format!("snapshot customer: {e}")))?;
    let payment_methods_snapshot = build_payment_methods_snapshot(db, input.business_profile_id.as_deref())?;
    let totals = compute_totals(&input.lines, input.tax_enabled, input.tax_rate);
    let number = numbering::next(db, DocType::Invoice)?;
    let now_s = now();
    let inv_id = Uuid::new_v4().to_string();

    let invoice = Invoice {
        id: inv_id.clone(),
        number,
        customer_id: input.customer_id,
        customer_snapshot: snapshot,
        business_profile_id: input.business_profile_id,
        source_quotation_id: None,
        issue_date: input.issue_date,
        due_date: input.due_date,
        currency: input.currency.clone(),
        tax_enabled: input.tax_enabled,
        tax_rate: input.tax_rate,
        subtotal: totals.subtotal,
        tax_amount: totals.tax_amount,
        total: totals.total,
        paid_amount: 0.0,
        payment_methods_snapshot,
        selected_bank_account_ids: input.selected_bank_account_ids,
        selected_qr_ids: input.selected_qr_ids,
        selected_static_methods: input.selected_static_methods,
        status: InvoiceStatus::Draft,
        notes: trim_opt(input.notes),
        terms: trim_opt(input.terms),
        created_at: now_s.clone(),
        updated_at: now_s,
    };
    let lines = build_lines(input.lines, &inv_id, &input.currency);

    db.transaction(|tx| {
        repository::insert_invoice(tx, &invoice)?;
        for l in &lines {
            repository::insert_line(tx, l)?;
        }
        Ok(())
    })?;

    Ok(InvoiceWithLines { invoice, lines })
}

pub fn create_from_quotation(
    db: &Db,
    input: CreateFromQuotationInput,
) -> AppResult<InvoiceWithLines> {
    if !is_iso_date(&input.issue_date) {
        return Err(AppError::Validation(
            "issue_date 必须是 YYYY-MM-DD 格式".into(),
        ));
    }
    if !is_iso_date(&input.due_date) {
        return Err(AppError::Validation("due_date 必须是 YYYY-MM-DD 格式".into()));
    }

    let qwl = quotation::find_by_id(db, &input.quotation_id)?;
    if qwl.quotation.status != quotation::QuotationStatus::Accepted {
        return Err(AppError::Validation(
            "只能从 已接受 状态的报价转发票".into(),
        ));
    }
    if qwl.quotation.converted_invoice_id.is_some() {
        return Err(AppError::Validation(
            "这份报价已经转过发票了".into(),
        ));
    }

    // For Quotation→Invoice conversion, prefer the user-supplied profile id
    // on the input, falling back to the source Quotation's profile id.
    let business_profile_id = input
        .business_profile_id
        .clone()
        .or_else(|| qwl.quotation.business_profile_id.clone());
    let payment_methods_snapshot = build_payment_methods_snapshot(db, business_profile_id.as_deref())?;
    let number = numbering::next(db, DocType::Invoice)?;
    let now_s = now();
    let inv_id = Uuid::new_v4().to_string();

    let line_inputs: Vec<LineItemInput> = qwl
        .lines
        .iter()
        .map(|l| LineItemInput {
            description: l.description.clone(),
            quantity: l.quantity,
            unit_price: l.unit_price,
        })
        .collect();

    let invoice = Invoice {
        id: inv_id.clone(),
        number,
        customer_id: qwl.quotation.customer_id.clone(),
        customer_snapshot: qwl.quotation.customer_snapshot.clone(),
        business_profile_id,
        source_quotation_id: Some(qwl.quotation.id.clone()),
        issue_date: input.issue_date,
        due_date: input.due_date,
        currency: qwl.quotation.currency.clone(),
        tax_enabled: qwl.quotation.tax_enabled,
        tax_rate: qwl.quotation.tax_rate,
        subtotal: qwl.quotation.subtotal,
        tax_amount: qwl.quotation.tax_amount,
        total: qwl.quotation.total,
        paid_amount: 0.0,
        payment_methods_snapshot,
        selected_bank_account_ids: Vec::new(),
        selected_qr_ids: Vec::new(),
        selected_static_methods: Vec::new(),
        status: InvoiceStatus::Draft,
        notes: qwl.quotation.notes.clone(),
        terms: qwl.quotation.terms.clone(),
        created_at: now_s.clone(),
        updated_at: now_s,
    };
    let lines = build_lines(line_inputs, &inv_id, &qwl.quotation.currency);

    let quotation_id = qwl.quotation.id.clone();
    let invoice_id_for_link = inv_id.clone();

    db.transaction(move |tx| {
        repository::insert_invoice(tx, &invoice)?;
        for l in &lines {
            repository::insert_line(tx, l)?;
        }
        quotation::set_converted_invoice_id_in_tx(tx, &quotation_id, &invoice_id_for_link)?;
        Ok(InvoiceWithLines { invoice, lines })
    })
}

pub fn update(db: &Db, input: UpdateInvoiceInput) -> AppResult<InvoiceWithLines> {
    validate_input(
        &input.issue_date,
        &input.due_date,
        &input.currency,
        input.tax_enabled,
        input.tax_rate,
        &input.lines,
    )?;

    let cust = customer::find_by_id(db, &input.customer_id)?;
    let snapshot = serde_json::to_value(&cust)
        .map_err(|e| AppError::Internal(format!("snapshot customer: {e}")))?;
    let payment_methods_snapshot = build_payment_methods_snapshot(db, input.business_profile_id.as_deref())?;
    let totals = compute_totals(&input.lines, input.tax_enabled, input.tax_rate);
    let now_s = now();

    let UpdateInvoiceInput {
        id,
        customer_id,
        business_profile_id,
        issue_date,
        due_date,
        currency,
        tax_enabled,
        tax_rate,
        lines: line_inputs,
        notes,
        terms,
        selected_bank_account_ids,
        selected_qr_ids,
        selected_static_methods,
    } = input;

    let new_lines = build_lines(line_inputs, &id, &currency);

    db.transaction(move |tx| {
        let mut inv = repository::find_invoice(tx, &id)?.ok_or_else(|| AppError::NotFound {
            entity: "invoice".into(),
            id: id.clone(),
        })?;
        if inv.status != InvoiceStatus::Draft {
            return Err(AppError::Validation(format!(
                "只有 Draft 状态可以修改，当前状态为 {}",
                inv.status.as_str()
            )));
        }

        inv.customer_id = customer_id;
        inv.customer_snapshot = snapshot;
        inv.business_profile_id = business_profile_id;
        inv.issue_date = issue_date;
        inv.due_date = due_date;
        inv.currency = currency;
        inv.tax_enabled = tax_enabled;
        inv.tax_rate = tax_rate;
        inv.subtotal = totals.subtotal;
        inv.tax_amount = totals.tax_amount;
        inv.total = totals.total;
        inv.payment_methods_snapshot = payment_methods_snapshot;
        inv.selected_bank_account_ids = selected_bank_account_ids;
        inv.selected_qr_ids = selected_qr_ids;
        inv.selected_static_methods = selected_static_methods;
        inv.notes = trim_opt(notes);
        inv.terms = trim_opt(terms);
        inv.updated_at = now_s;

        repository::update_header(tx, &inv)?;
        repository::delete_lines_for(tx, &inv.id)?;
        for l in &new_lines {
            repository::insert_line(tx, l)?;
        }
        Ok(InvoiceWithLines {
            invoice: inv,
            lines: new_lines,
        })
    })
}

pub fn find_by_id(db: &Db, id: &str) -> AppResult<InvoiceWithLines> {
    db.with_conn(|c| {
        let inv = repository::find_invoice(c, id)?.ok_or_else(|| AppError::NotFound {
            entity: "invoice".into(),
            id: id.into(),
        })?;
        let lines = repository::list_lines(c, &inv.id)?;
        Ok(InvoiceWithLines { invoice: inv, lines })
    })
}

pub fn list(db: &Db) -> AppResult<Vec<Invoice>> {
    db.with_conn(|c| repository::list(c))
}

pub fn list_by_customer(db: &Db, customer_id: &str) -> AppResult<Vec<Invoice>> {
    db.with_conn(|c| repository::list_by_customer(c, customer_id))
}

fn transition_to(db: &Db, id: &str, to: InvoiceStatus) -> AppResult<Invoice> {
    db.transaction(|tx| {
        let inv = repository::find_invoice(tx, id)?.ok_or_else(|| AppError::NotFound {
            entity: "invoice".into(),
            id: id.into(),
        })?;
        let new_status = transition(inv.status, to)?;
        repository::update_status(tx, id, new_status, &now())?;
        let updated = repository::find_invoice(tx, id)?.ok_or_else(|| AppError::NotFound {
            entity: "invoice".into(),
            id: id.into(),
        })?;
        Ok(updated)
    })
}

pub fn mark_sent(db: &Db, id: &str) -> AppResult<Invoice> {
    let sent = transition_to(db, id, InvoiceStatus::Sent)?;
    // If the invoice is already past its due_date at the moment of sending,
    // promote straight to Overdue so the user doesn't see a Sent invoice that's
    // logically already late (auto_mark_overdue_all only runs at startup).
    if sent.due_date.as_str() < today_iso().as_str() {
        return transition_to(db, id, InvoiceStatus::Overdue);
    }
    Ok(sent)
}

pub fn mark_partial_paid(db: &Db, id: &str) -> AppResult<Invoice> {
    transition_to(db, id, InvoiceStatus::PartialPaid)
}

pub fn mark_paid(db: &Db, id: &str) -> AppResult<Invoice> {
    transition_to(db, id, InvoiceStatus::Paid)
}

pub fn mark_overdue(db: &Db, id: &str) -> AppResult<Invoice> {
    transition_to(db, id, InvoiceStatus::Overdue)
}

pub fn mark_void(db: &Db, id: &str) -> AppResult<Invoice> {
    transition_to(db, id, InvoiceStatus::Void)
}

/// Move an Overdue invoice back to Sent (or PartialPaid if some payment exists).
pub fn cancel_overdue(db: &Db, id: &str) -> AppResult<Invoice> {
    db.transaction(|tx| {
        let inv = repository::find_invoice(tx, id)?.ok_or_else(|| AppError::NotFound {
            entity: "invoice".into(),
            id: id.into(),
        })?;
        if inv.status != InvoiceStatus::Overdue {
            return Err(AppError::Validation(
                "只有 Overdue 状态可以取消逾期".into(),
            ));
        }
        let target = if inv.paid_amount > 0.0 {
            InvoiceStatus::PartialPaid
        } else {
            InvoiceStatus::Sent
        };
        let new_status = transition(inv.status, target)?;
        repository::update_status(tx, id, new_status, &now())?;
        let updated = repository::find_invoice(tx, id)?.ok_or_else(|| AppError::NotFound {
            entity: "invoice".into(),
            id: id.into(),
        })?;
        Ok(updated)
    })
}

/// Find every Sent / PartialPaid invoice past due_date and flip it to Overdue.
/// Intended to be called at app startup; quiet if nothing to do.
pub fn auto_mark_overdue_all(db: &Db) -> AppResult<u32> {
    let today = today_iso();
    let ids = db.with_conn(|c| repository::list_overdue_candidates(c, &today))?;
    let mut flipped = 0u32;
    for id in ids {
        // each transition is its own transaction; if one fails we keep going for others
        if mark_overdue(db, &id).is_ok() {
            flipped += 1;
        }
    }
    Ok(flipped)
}

/// Recalculate paid_amount from payment_voucher rows. Forces a refresh — usable
/// from any code path that has access to the `Db`.
pub fn recalc_paid_amount(db: &Db, invoice_id: &str) -> AppResult<f64> {
    let sum = crate::domain::payment_voucher::sum_by_invoice(db, invoice_id)?;
    db.transaction(|tx| apply_paid_amount_in_tx(tx, invoice_id, sum))?;
    Ok(sum)
}

/// Transaction-aware: written `paid_amount` on the invoice + optionally
/// auto-transition the status when paid_amount crosses thresholds.
///
/// Auto-transition policy (only forward; never downgrade on PV deletion):
///   - Sent / Overdue: paid >= total → Paid; paid > 0 → PartialPaid
///   - PartialPaid:    paid >= total → Paid; otherwise stay
///   - Draft / Paid / Void: leave status alone (PV shouldn't exist on Draft/Void anyway)
pub fn apply_paid_amount_in_tx(
    conn: &Connection,
    invoice_id: &str,
    new_paid: f64,
) -> AppResult<()> {
    let now_s = now();
    repository::update_paid_amount(conn, invoice_id, new_paid, &now_s)?;

    let inv = repository::find_invoice(conn, invoice_id)?.ok_or_else(|| AppError::NotFound {
        entity: "invoice".into(),
        id: invoice_id.into(),
    })?;

    let new_status = match inv.status {
        InvoiceStatus::Sent | InvoiceStatus::Overdue => {
            if new_paid >= inv.total {
                Some(InvoiceStatus::Paid)
            } else if new_paid > 0.0 {
                Some(InvoiceStatus::PartialPaid)
            } else {
                None
            }
        }
        InvoiceStatus::PartialPaid => {
            if new_paid >= inv.total {
                Some(InvoiceStatus::Paid)
            } else {
                None
            }
        }
        _ => None,
    };

    if let Some(target) = new_status {
        if super::state_machine::can_transition(inv.status, target) {
            repository::update_status(conn, invoice_id, target, &now_s)?;
        }
    }
    Ok(())
}

/// Read-only: is this invoice in a state that allows recording / editing / deleting
/// payment vouchers? Draft and Void invoices reject all PV mutations.
pub fn allows_payment_voucher(status: InvoiceStatus) -> bool {
    !matches!(status, InvoiceStatus::Draft | InvoiceStatus::Void)
}

/// Transaction-aware: error out early if the invoice doesn't allow PV mutations.
/// Used by `domain::payment_voucher::update` / `delete` so they don't race
/// against a concurrent Void/Draft transition.
pub fn assert_allows_payment_voucher_in_tx(
    conn: &Connection,
    invoice_id: &str,
) -> AppResult<()> {
    let inv = repository::find_invoice(conn, invoice_id)?.ok_or_else(|| AppError::NotFound {
        entity: "invoice".into(),
        id: invoice_id.into(),
    })?;
    if !allows_payment_voucher(inv.status) {
        return Err(AppError::Validation(format!(
            "Invoice 状态为 {} 时不能修改付款记录",
            inv.status.as_str()
        )));
    }
    Ok(())
}

/// Escape hatch from the Void terminal state. Recomputes the "natural" status
/// based on `paid_amount` and `due_date` at restore time — this is NOT a state-
/// machine transition; the state machine still treats Void as terminal so all
/// other code paths behave correctly.
///
/// Use case: an invoice was voided in error and the customer still wants to pay
/// against it. Restore preserves paid_amount + line items + numbering.
pub fn restore_void(db: &Db, id: &str) -> AppResult<Invoice> {
    db.transaction(|tx| {
        let inv = repository::find_invoice(tx, id)?.ok_or_else(|| AppError::NotFound {
            entity: "invoice".into(),
            id: id.into(),
        })?;
        if inv.status != InvoiceStatus::Void {
            return Err(AppError::Validation(
                "只有作废状态的 Invoice 可以恢复".into(),
            ));
        }
        let natural = compute_natural_status(&inv, &today_iso());
        // bypass `transition()` deliberately; Void is terminal in the state machine.
        repository::update_status(tx, id, natural, &now())?;
        let updated = repository::find_invoice(tx, id)?.ok_or_else(|| AppError::NotFound {
            entity: "invoice".into(),
            id: id.into(),
        })?;
        Ok(updated)
    })
}

fn compute_natural_status(inv: &Invoice, today: &str) -> InvoiceStatus {
    if inv.paid_amount >= inv.total && inv.total > 0.0 {
        InvoiceStatus::Paid
    } else if inv.due_date.as_str() < today {
        InvoiceStatus::Overdue
    } else if inv.paid_amount > 0.0 {
        InvoiceStatus::PartialPaid
    } else {
        InvoiceStatus::Sent
    }
}

fn build_payment_methods_snapshot(
    db: &Db,
    business_profile_id: Option<&str>,
) -> AppResult<serde_json::Value> {
    // Kept for historical record (the column was never removed). Live rendering
    // pulls from the current profile (see service::pdf_renderer::renderer), so
    // this snapshot is mostly informational.
    let Some(id) = business_profile_id else {
        return Ok(serde_json::json!({}));
    };
    let Ok(p) = business_profile::find_by_id(db, id) else {
        return Ok(serde_json::json!({}));
    };
    Ok(serde_json::json!({
        "enabled_payment_methods": p.enabled_payment_methods,
        "bank_accounts": p.bank_accounts,
    }))
}

fn trim_opt(v: Option<String>) -> Option<String> {
    v.and_then(|s| {
        let t = s.trim();
        if t.is_empty() { None } else { Some(t.to_string()) }
    })
}
