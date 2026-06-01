import { invoke } from "@tauri-apps/api/core";
import type { MonthlyRevenueRow } from "@/types/bindings/MonthlyRevenueRow";
import type { OutstandingReport } from "@/types/bindings/OutstandingReport";
import type { YearlyReport } from "@/types/bindings/YearlyReport";

export const reportApi = {
  monthly: (year: number, month: number) =>
    invoke<MonthlyRevenueRow>("report_monthly_revenue", { year, month }),
  yearly: (year: number) => invoke<YearlyReport>("report_yearly_revenue", { year }),
  outstanding: () => invoke<OutstandingReport>("report_outstanding_invoices"),
};
