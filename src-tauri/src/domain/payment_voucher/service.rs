use uuid::Uuid;

use crate::domain::{customer, invoice};
use crate::error::{AppError, AppResult};
use crate::infra::Db;
use crate::service::numbering::{self, DocType};

use super::repository;
use super::types::{CreatePaymentVoucherInput, PaymentVoucher, UpdatePaymentVoucherInput};

fn now() -> String {
    chrono::Utc::now().to_rfc3339()
}

fn is_iso_date(s: &str) -> bool {
    chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").is_ok()
}

fn validate(date: &str, amount: f64, payment_method: &str) -> AppResult<()> {
    if !is_iso_date(date) {
        return Err(AppError::Validation("date 必须是 YYYY-MM-DD 格式".into()));
    }
    if amount <= 0.0 {
        return Err(AppError::Validation("amount 必须大于 0".into()));
    }
    if payment_method.trim().is_empty() {
        return Err(AppError::Validation("payment_method 不能为空".into()));
    }
    Ok(())
}

pub fn create(db: &Db, input: CreatePaymentVoucherInput) -> AppResult<PaymentVoucher> {
    validate(&input.date, input.amount, &input.payment_method)?;

    // Two flavors: linked-to-invoice vs standalone.
    let (invoice_id, customer_id, customer_snapshot, currency, business_profile_id) =
        match input.invoice_id.clone() {
            Some(inv_id) => {
                let iwl = invoice::find_by_id(db, &inv_id)?;
                if !invoice::allows_payment_voucher(iwl.invoice.status) {
                    return Err(AppError::Validation(format!(
                        "Invoice 状态为 {} 时不能记录付款",
                        iwl.invoice.status.as_str()
                    )));
                }
                (
                    Some(inv_id),
                    iwl.invoice.customer_id.clone(),
                    iwl.invoice.customer_snapshot.clone(),
                    iwl.invoice.currency.clone(),
                    iwl.invoice.business_profile_id.clone(),
                )
            }
            None => {
                let cust_id = input.customer_id.clone().ok_or_else(|| {
                    AppError::Validation("独立 PV 必须提供 customer_id".into())
                })?;
                let curr = input
                    .currency
                    .clone()
                    .filter(|s| !s.trim().is_empty())
                    .ok_or_else(|| AppError::Validation("独立 PV 必须提供 currency".into()))?;
                let cust = customer::find_by_id(db, &cust_id)?;
                let snap = serde_json::to_value(&cust)
                    .map_err(|e| AppError::Internal(format!("snapshot customer: {e}")))?;
                (None, cust_id, snap, curr, input.business_profile_id.clone())
            }
        };

    let number = numbering::next(db, DocType::PaymentVoucher)?;
    let pv = PaymentVoucher {
        id: Uuid::new_v4().to_string(),
        number,
        invoice_id: invoice_id.clone(),
        customer_id,
        customer_snapshot,
        business_profile_id,
        date: input.date,
        amount: input.amount,
        currency,
        payment_method: input.payment_method.trim().to_string(),
        notes: trim_opt(input.notes),
        created_at: now(),
    };

    db.transaction(|tx| {
        repository::insert(tx, &pv)?;
        if let Some(inv_id) = &invoice_id {
            let sum = repository::sum_by_invoice(tx, inv_id)?;
            invoice::apply_paid_amount_in_tx(tx, inv_id, sum)?;
        }
        Ok(())
    })?;

    Ok(pv)
}

pub fn update(db: &Db, input: UpdatePaymentVoucherInput) -> AppResult<PaymentVoucher> {
    validate(&input.date, input.amount, &input.payment_method)?;

    db.transaction(|tx| {
        let mut existing = repository::find_by_id(tx, &input.id)?.ok_or_else(|| {
            AppError::NotFound {
                entity: "payment_voucher".into(),
                id: input.id.clone(),
            }
        })?;
        if let Some(inv_id) = existing.invoice_id.clone() {
            invoice::assert_allows_payment_voucher_in_tx(tx, &inv_id)?;
        }
        existing.date = input.date;
        existing.amount = input.amount;
        existing.payment_method = input.payment_method.trim().to_string();
        existing.notes = trim_opt(input.notes);
        repository::update(tx, &existing)?;
        if let Some(inv_id) = existing.invoice_id.clone() {
            let sum = repository::sum_by_invoice(tx, &inv_id)?;
            invoice::apply_paid_amount_in_tx(tx, &inv_id, sum)?;
        }
        Ok(existing)
    })
}

pub fn delete(db: &Db, id: &str) -> AppResult<()> {
    db.transaction(|tx| {
        let existing = repository::find_by_id(tx, id)?.ok_or_else(|| AppError::NotFound {
            entity: "payment_voucher".into(),
            id: id.into(),
        })?;
        if let Some(inv_id) = existing.invoice_id.clone() {
            invoice::assert_allows_payment_voucher_in_tx(tx, &inv_id)?;
        }
        repository::delete(tx, id)?;
        if let Some(inv_id) = existing.invoice_id.clone() {
            let sum = repository::sum_by_invoice(tx, &inv_id)?;
            invoice::apply_paid_amount_in_tx(tx, &inv_id, sum)?;
        }
        Ok(())
    })
}

pub fn find_by_id(db: &Db, id: &str) -> AppResult<PaymentVoucher> {
    db.with_conn(|c| {
        repository::find_by_id(c, id)?.ok_or_else(|| AppError::NotFound {
            entity: "payment_voucher".into(),
            id: id.into(),
        })
    })
}

pub fn list(db: &Db) -> AppResult<Vec<PaymentVoucher>> {
    db.with_conn(|c| repository::list(c))
}

pub fn list_by_invoice(db: &Db, invoice_id: &str) -> AppResult<Vec<PaymentVoucher>> {
    db.with_conn(|c| repository::list_by_invoice(c, invoice_id))
}

pub fn list_by_customer(db: &Db, customer_id: &str) -> AppResult<Vec<PaymentVoucher>> {
    db.with_conn(|c| repository::list_by_customer(c, customer_id))
}

pub fn sum_by_invoice(db: &Db, invoice_id: &str) -> AppResult<f64> {
    db.with_conn(|c| repository::sum_by_invoice(c, invoice_id))
}

fn trim_opt(v: Option<String>) -> Option<String> {
    v.and_then(|s| {
        let t = s.trim();
        if t.is_empty() { None } else { Some(t.to_string()) }
    })
}
