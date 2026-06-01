use tauri::State;

use crate::error::AppResult;
use crate::AppState;

use super::service;
use super::types::{MonthlyRevenueRow, OutstandingReport, YearlyReport};

#[tauri::command]
pub fn report_monthly_revenue(
    state: State<'_, AppState>,
    year: i32,
    month: u32,
) -> AppResult<MonthlyRevenueRow> {
    service::monthly_revenue(&state.db, year, month)
}

#[tauri::command]
pub fn report_yearly_revenue(
    state: State<'_, AppState>,
    year: i32,
) -> AppResult<YearlyReport> {
    service::yearly_revenue(&state.db, year)
}

#[tauri::command]
pub fn report_outstanding_invoices(state: State<'_, AppState>) -> AppResult<OutstandingReport> {
    service::outstanding_invoices(&state.db)
}
