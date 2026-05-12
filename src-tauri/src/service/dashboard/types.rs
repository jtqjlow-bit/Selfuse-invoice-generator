use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::domain::invoice::Invoice;

/// One row of `currency → amount`. Aggregates are returned as a list because
/// docs in this app can be in mixed currencies (MYR + USD etc.); summing them
/// would be wrong.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/bindings/")]
pub struct CurrencyAmount {
    pub currency: String,
    pub amount: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/bindings/")]
pub struct DashboardData {
    /// Sum of PV.amount for PVs dated in the current calendar month, grouped
    /// by currency. Represents cash collected this month.
    pub this_month_revenue: Vec<CurrencyAmount>,
    /// Sum of (invoice.total - invoice.paid_amount) for invoices in Sent /
    /// PartialPaid / Overdue. Excludes Draft (not issued) and Void (cancelled).
    pub outstanding_total: Vec<CurrencyAmount>,
    /// Count of invoices currently in Overdue status.
    pub overdue_count: u32,
    /// 5 most recent invoices (by issue_date desc).
    pub recent_invoices: Vec<Invoice>,
}
