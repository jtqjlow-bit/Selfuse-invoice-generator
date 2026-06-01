import { useEffect, useState } from "react";
import { Link } from "react-router-dom";
import { reportApi } from "@/api/report";
import { formatErr, formatMoney } from "@/common/utils/format";
import {
  invoiceStatusBadgeClass,
  invoiceStatusLabel,
} from "@/features/invoice/statusLabel";
import type { CurrencyAmount } from "@/types/bindings/CurrencyAmount";
import type { OutstandingReport } from "@/types/bindings/OutstandingReport";
import type { YearlyReport } from "@/types/bindings/YearlyReport";

const MONTH_NAMES = [
  "1月", "2月", "3月", "4月", "5月", "6月",
  "7月", "8月", "9月", "10月", "11月", "12月",
];

type Tab = "revenue" | "outstanding";

export function ReportPage() {
  const [tab, setTab] = useState<Tab>("revenue");
  return (
    <div className="mx-auto max-w-5xl p-8">
      <h1 className="mb-1 text-2xl font-semibold">报表</h1>
      <p className="mb-6 text-sm text-muted-foreground">营收回顾 / 未结清发票</p>
      <div className="mb-4 flex gap-1 border-b border-border">
        <TabButton active={tab === "revenue"} onClick={() => setTab("revenue")}>
          年度营收
        </TabButton>
        <TabButton
          active={tab === "outstanding"}
          onClick={() => setTab("outstanding")}
        >
          未结清发票
        </TabButton>
      </div>
      {tab === "revenue" ? <RevenueView /> : <OutstandingView />}
    </div>
  );
}

function TabButton({
  active,
  onClick,
  children,
}: {
  active: boolean;
  onClick: () => void;
  children: React.ReactNode;
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      className={`-mb-px border-b-2 px-3 py-2 text-sm ${
        active
          ? "border-primary text-foreground"
          : "border-transparent text-muted-foreground hover:text-foreground"
      }`}
    >
      {children}
    </button>
  );
}

function RevenueView() {
  const currentYear = new Date().getFullYear();
  const [year, setYear] = useState(currentYear);
  const [report, setReport] = useState<YearlyReport | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    setLoading(true);
    setError(null);
    reportApi
      .yearly(year)
      .then(setReport)
      .catch((e) => setError(formatErr(e)))
      .finally(() => setLoading(false));
  }, [year]);

  return (
    <div>
      <div className="mb-4 flex items-center gap-2">
        <label className="text-sm text-muted-foreground">年份</label>
        <select
          className="rounded-md border border-input bg-background px-2 py-1 text-sm"
          value={year}
          onChange={(e) => setYear(Number(e.target.value))}
        >
          {Array.from({ length: 6 }, (_, i) => currentYear - i).map((y) => (
            <option key={y} value={y}>
              {y}
            </option>
          ))}
        </select>
      </div>

      {loading && <p className="text-sm text-muted-foreground">加载中…</p>}
      {error && <p className="text-sm text-red-600">{error}</p>}

      {report && !loading && (
        <>
          <div className="mb-4 rounded-md border border-green-200 bg-green-50 p-4">
            <div className="text-xs font-medium text-green-800">
              {year} 全年营收
            </div>
            {report.total_revenue.length === 0 ? (
              <div className="mt-1 text-xl font-semibold text-green-800">—</div>
            ) : (
              <div className="mt-1 space-y-0.5">
                {report.total_revenue.map((a) => (
                  <div
                    key={a.currency}
                    className="text-xl font-semibold text-green-800 font-mono"
                  >
                    {formatMoney(a.amount, a.currency)}
                  </div>
                ))}
              </div>
            )}
          </div>

          <div className="overflow-hidden rounded-md border border-border">
            <table className="w-full text-sm">
              <thead className="bg-muted text-left text-muted-foreground">
                <tr>
                  <th className="w-24 px-3 py-2 font-medium">月份</th>
                  <th className="w-24 px-3 py-2 text-right font-medium">PV 数</th>
                  <th className="px-3 py-2 text-right font-medium">营收</th>
                </tr>
              </thead>
              <tbody>
                {report.months.map((m) => (
                  <tr key={m.month} className="border-t border-border">
                    <td className="px-3 py-2">{MONTH_NAMES[m.month - 1]}</td>
                    <td className="px-3 py-2 text-right text-muted-foreground">
                      {m.pv_count}
                    </td>
                    <td className="px-3 py-2 text-right">
                      {m.revenue.length === 0 ? (
                        <span className="text-muted-foreground">—</span>
                      ) : (
                        <RevenueCell amounts={m.revenue} />
                      )}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </>
      )}
    </div>
  );
}

function RevenueCell({ amounts }: { amounts: CurrencyAmount[] }) {
  return (
    <div className="space-y-0.5">
      {amounts.map((a) => (
        <div key={a.currency} className="font-mono">
          {formatMoney(a.amount, a.currency)}
        </div>
      ))}
    </div>
  );
}

function OutstandingView() {
  const [report, setReport] = useState<OutstandingReport | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    reportApi
      .outstanding()
      .then(setReport)
      .catch((e) => setError(formatErr(e)))
      .finally(() => setLoading(false));
  }, []);

  if (loading) return <p className="text-sm text-muted-foreground">加载中…</p>;
  if (error) return <p className="text-sm text-red-600">{error}</p>;
  if (!report) return null;

  return (
    <div>
      <div className="mb-4 rounded-md border border-amber-200 bg-amber-50 p-4">
        <div className="text-xs font-medium text-amber-800">未结清总额</div>
        {report.total_outstanding.length === 0 ? (
          <div className="mt-1 text-xl font-semibold text-amber-800">—</div>
        ) : (
          <div className="mt-1 space-y-0.5">
            {report.total_outstanding.map((a) => (
              <div
                key={a.currency}
                className="text-xl font-semibold text-amber-800 font-mono"
              >
                {formatMoney(a.amount, a.currency)}
              </div>
            ))}
          </div>
        )}
        <div className="mt-2 text-xs text-amber-800/80">
          共 {report.invoices.length} 张未结清发票
        </div>
      </div>

      {report.invoices.length === 0 ? (
        <p className="rounded-md border border-border bg-card p-4 text-sm text-muted-foreground">
          全部发票都结清啦 🎉
        </p>
      ) : (
        <div className="overflow-hidden rounded-md border border-border">
          <table className="w-full text-sm">
            <thead className="bg-muted text-left text-muted-foreground">
              <tr>
                <th className="px-3 py-2 font-medium">编号</th>
                <th className="px-3 py-2 font-medium">客户</th>
                <th className="w-28 px-3 py-2 font-medium">到期日</th>
                <th className="w-24 px-3 py-2 text-right font-medium">逾期</th>
                <th className="w-32 px-3 py-2 text-right font-medium">余额</th>
                <th className="w-24 px-3 py-2 font-medium">状态</th>
              </tr>
            </thead>
            <tbody>
              {report.invoices.map((r) => (
                <tr key={r.invoice.id} className="border-t border-border">
                  <td className="px-3 py-2">
                    <Link
                      to={`/invoices/${r.invoice.id}`}
                      className="font-mono text-primary hover:underline"
                    >
                      {r.invoice.number}
                    </Link>
                  </td>
                  <td className="px-3 py-2 truncate max-w-[200px]">
                    {r.customer_name}
                  </td>
                  <td className="px-3 py-2 text-muted-foreground">
                    {r.invoice.due_date}
                  </td>
                  <td className="px-3 py-2 text-right">
                    {r.days_overdue > 0 ? (
                      <span className="text-red-600">{r.days_overdue} 天</span>
                    ) : (
                      <span className="text-muted-foreground">—</span>
                    )}
                  </td>
                  <td className="px-3 py-2 text-right font-mono">
                    {formatMoney(r.balance, r.invoice.currency)}
                  </td>
                  <td className="px-3 py-2">
                    <span
                      className={`inline-block rounded-md px-2 py-0.5 text-xs ${invoiceStatusBadgeClass(
                        r.invoice.status,
                      )}`}
                    >
                      {invoiceStatusLabel(r.invoice.status)}
                    </span>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}
