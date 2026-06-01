use std::collections::BTreeMap;

use chrono::{Datelike, NaiveDate, Utc};

use crate::domain::invoice::{self, Invoice, InvoiceStatus};
use crate::domain::payment_voucher::{self, PaymentVoucher};
use crate::error::AppResult;
use crate::infra::Db;

use super::types::{
    CurrencyAmount, MonthlyRevenueRow, OutstandingInvoice, OutstandingReport, YearlyReport,
};

pub fn monthly_revenue(db: &Db, year: i32, month: u32) -> AppResult<MonthlyRevenueRow> {
    let pvs = payment_voucher::list(db)?;
    Ok(build_row(&pvs, year, month))
}

pub fn yearly_revenue(db: &Db, year: i32) -> AppResult<YearlyReport> {
    let pvs = payment_voucher::list(db)?;
    let months: Vec<MonthlyRevenueRow> = (1..=12).map(|m| build_row(&pvs, year, m)).collect();
    let total = sum_currency_amounts(months.iter().flat_map(|m| m.revenue.iter().cloned()));
    Ok(YearlyReport {
        year,
        months,
        total_revenue: total,
    })
}

pub fn outstanding_invoices(db: &Db) -> AppResult<OutstandingReport> {
    let invoices = invoice::list(db)?;
    let today = Utc::now().date_naive();
    let mut rows: Vec<OutstandingInvoice> = invoices
        .into_iter()
        .filter_map(|inv| invoice_outstanding_row(inv, today))
        .collect();
    // Most overdue first; tie-break by due_date asc so the UI is deterministic.
    rows.sort_by(|a, b| {
        b.days_overdue
            .cmp(&a.days_overdue)
            .then_with(|| a.invoice.due_date.cmp(&b.invoice.due_date))
    });

    let totals = sum_currency_amounts(
        rows.iter()
            .map(|r| CurrencyAmount {
                currency: r.invoice.currency.clone(),
                amount: r.balance,
            }),
    );
    Ok(OutstandingReport {
        invoices: rows,
        total_outstanding: totals,
    })
}

fn build_row(pvs: &[PaymentVoucher], year: i32, month: u32) -> MonthlyRevenueRow {
    let mut sums: BTreeMap<String, f64> = BTreeMap::new();
    let mut count: u32 = 0;
    for pv in pvs {
        if let Some(d) = NaiveDate::parse_from_str(&pv.date, "%Y-%m-%d").ok() {
            if d.year() == year && d.month() == month {
                *sums.entry(pv.currency.clone()).or_insert(0.0) += pv.amount;
                count += 1;
            }
        }
    }
    MonthlyRevenueRow {
        year,
        month,
        revenue: sums
            .into_iter()
            .map(|(currency, amount)| CurrencyAmount { currency, amount })
            .collect(),
        pv_count: count,
    }
}

fn invoice_outstanding_row(inv: Invoice, today: NaiveDate) -> Option<OutstandingInvoice> {
    if !matches!(
        inv.status,
        InvoiceStatus::Sent | InvoiceStatus::PartialPaid | InvoiceStatus::Overdue
    ) {
        return None;
    }
    let balance = inv.total - inv.paid_amount;
    if balance <= 0.0 {
        return None;
    }
    let due = NaiveDate::parse_from_str(&inv.due_date, "%Y-%m-%d").ok();
    let days_overdue = match due {
        Some(d) if today > d => (today - d).num_days() as i32,
        _ => 0,
    };
    let customer_name = inv
        .customer_snapshot
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("(未知客户)")
        .to_string();
    Some(OutstandingInvoice {
        invoice: inv,
        customer_name,
        balance,
        days_overdue,
    })
}

fn sum_currency_amounts(iter: impl IntoIterator<Item = CurrencyAmount>) -> Vec<CurrencyAmount> {
    let mut sums: BTreeMap<String, f64> = BTreeMap::new();
    for ca in iter {
        *sums.entry(ca.currency).or_insert(0.0) += ca.amount;
    }
    sums.into_iter()
        .map(|(currency, amount)| CurrencyAmount { currency, amount })
        .collect()
}
