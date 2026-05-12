use std::collections::BTreeMap;

use chrono::{Datelike, NaiveDate, Utc};

use crate::domain::invoice::{self, InvoiceStatus};
use crate::domain::payment_voucher;
use crate::error::AppResult;
use crate::infra::Db;

use super::types::{CurrencyAmount, DashboardData};

pub fn get_dashboard_data(db: &Db) -> AppResult<DashboardData> {
    let today = Utc::now().date_naive();
    let (start, next_month_start) = month_bounds(today);

    let pvs = payment_voucher::list(db)?;
    let invoices = invoice::list(db)?;

    let mut revenue: BTreeMap<String, f64> = BTreeMap::new();
    for pv in &pvs {
        let d = NaiveDate::parse_from_str(&pv.date, "%Y-%m-%d").ok();
        if let Some(d) = d {
            if d >= start && d < next_month_start {
                *revenue.entry(pv.currency.clone()).or_insert(0.0) += pv.amount;
            }
        }
    }

    let mut outstanding: BTreeMap<String, f64> = BTreeMap::new();
    let mut overdue_count: u32 = 0;
    for inv in &invoices {
        match inv.status {
            InvoiceStatus::Sent | InvoiceStatus::PartialPaid | InvoiceStatus::Overdue => {
                let bal = inv.total - inv.paid_amount;
                if bal > 0.0 {
                    *outstanding.entry(inv.currency.clone()).or_insert(0.0) += bal;
                }
            }
            _ => {}
        }
        if inv.status == InvoiceStatus::Overdue {
            overdue_count += 1;
        }
    }

    // Recent 5 (assumes invoice::list returns sorted by issue_date desc; the
    // repo's ORDER BY confirms this). Take first 5.
    let recent_invoices: Vec<_> = invoices.into_iter().take(5).collect();

    Ok(DashboardData {
        this_month_revenue: revenue
            .into_iter()
            .map(|(currency, amount)| CurrencyAmount { currency, amount })
            .collect(),
        outstanding_total: outstanding
            .into_iter()
            .map(|(currency, amount)| CurrencyAmount { currency, amount })
            .collect(),
        overdue_count,
        recent_invoices,
    })
}

/// Inclusive start of current month, exclusive start of next month.
fn month_bounds(today: NaiveDate) -> (NaiveDate, NaiveDate) {
    let start = NaiveDate::from_ymd_opt(today.year(), today.month(), 1).expect("valid month 1st");
    let (ny, nm) = if today.month() == 12 {
        (today.year() + 1, 1)
    } else {
        (today.year(), today.month() + 1)
    };
    let next = NaiveDate::from_ymd_opt(ny, nm, 1).expect("valid next-month 1st");
    (start, next)
}
