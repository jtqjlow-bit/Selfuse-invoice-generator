use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::domain::invoice::Invoice;

/// `currency → amount` row. Used wherever this app exposes a money aggregate
/// that may span multiple currencies (MYR + USD etc.). Lives in the report
/// service because it's the canonical place for money aggregations; dashboard
/// re-imports it.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/bindings/")]
pub struct CurrencyAmount {
    pub currency: String,
    pub amount: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/bindings/")]
pub struct MonthlyRevenueRow {
    pub year: i32,
    pub month: u32,
    /// Sum of PV.amount for PVs dated in this month, grouped by currency.
    pub revenue: Vec<CurrencyAmount>,
    pub pv_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/bindings/")]
pub struct YearlyReport {
    pub year: i32,
    /// 12 rows (January through December). Empty-month rows are still included
    /// so the UI can render a clean 12-row grid.
    pub months: Vec<MonthlyRevenueRow>,
    pub total_revenue: Vec<CurrencyAmount>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/bindings/")]
pub struct OutstandingInvoice {
    pub invoice: Invoice,
    /// Pulled from invoice.customer_snapshot.name for convenient display
    /// without the frontend having to re-parse the JSON snapshot.
    pub customer_name: String,
    pub balance: f64,
    /// Days past due_date. 0 if the due date is today or in the future.
    pub days_overdue: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/bindings/")]
pub struct OutstandingReport {
    /// Sorted by days_overdue desc, then by due_date asc.
    pub invoices: Vec<OutstandingInvoice>,
    pub total_outstanding: Vec<CurrencyAmount>,
}
