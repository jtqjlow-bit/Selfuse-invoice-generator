import { useEffect, useState } from "react";
import { Link } from "react-router-dom";
import { dashboardApi } from "@/api/dashboard";
import { formatErr, formatMoney } from "@/common/utils/format";
import { invoiceStatusBadgeClass, invoiceStatusLabel } from "@/features/invoice/statusLabel";
import type { CurrencyAmount } from "@/types/bindings/CurrencyAmount";
import type { DashboardData } from "@/types/bindings/DashboardData";

export function DashboardPage() {
  const [data, setData] = useState<DashboardData | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    dashboardApi
      .getData()
      .then(setData)
      .catch((e) => setError(formatErr(e)))
      .finally(() => setLoading(false));
  }, []);

  if (loading) return <p className="p-8 text-muted-foreground">加载中…</p>;
  if (error) return <p className="p-8 text-sm text-red-600">{error}</p>;
  if (!data) return null;

  return (
    <div className="mx-auto max-w-5xl p-8">
      <h1 className="mb-1 text-2xl font-semibold">Dashboard</h1>
      <p className="mb-6 text-sm text-muted-foreground">本月概览</p>

      <div className="mb-8 grid grid-cols-1 gap-4 sm:grid-cols-3">
        <StatCard
          label="本月营收"
          tooltip="本日历月内所有 Payment Voucher 金额合计（按币种分组）"
          amounts={data.this_month_revenue}
          tone="green"
        />
        <StatCard
          label="未付款总额"
          tooltip="已发送 / 部分付款 / 逾期 状态发票的未结金额合计"
          amounts={data.outstanding_total}
          tone="amber"
        />
        <CountCard
          label="逾期发票"
          count={data.overdue_count}
          tone={data.overdue_count > 0 ? "red" : "gray"}
        />
      </div>

      <section>
        <div className="mb-3 flex items-center justify-between">
          <h2 className="text-base font-medium">最近发票</h2>
          <Link to="/invoices" className="text-xs text-primary hover:underline">
            查看全部 →
          </Link>
        </div>
        {data.recent_invoices.length === 0 ? (
          <p className="rounded-md border border-border bg-card p-4 text-sm text-muted-foreground">
            还没有发票。先去{" "}
            <Link to="/invoices/new" className="text-primary hover:underline">
              开第一张
            </Link>
            ？
          </p>
        ) : (
          <div className="overflow-hidden rounded-md border border-border">
            <table className="w-full text-sm">
              <thead className="bg-muted text-left text-muted-foreground">
                <tr>
                  <th className="px-3 py-2 font-medium">编号</th>
                  <th className="px-3 py-2 font-medium">日期</th>
                  <th className="px-3 py-2 font-medium">客户</th>
                  <th className="px-3 py-2 text-right font-medium">合计</th>
                  <th className="px-3 py-2 text-right font-medium">已付</th>
                  <th className="w-28 px-3 py-2 font-medium">状态</th>
                </tr>
              </thead>
              <tbody>
                {data.recent_invoices.map((inv) => {
                  const customerName =
                    (inv.customer_snapshot as { name?: string } | null)?.name ??
                    "(未知客户)";
                  return (
                    <tr key={inv.id} className="border-t border-border">
                      <td className="px-3 py-2">
                        <Link
                          to={`/invoices/${inv.id}`}
                          className="font-mono text-primary hover:underline"
                        >
                          {inv.number}
                        </Link>
                      </td>
                      <td className="px-3 py-2 text-muted-foreground">
                        {inv.issue_date}
                      </td>
                      <td className="px-3 py-2 truncate max-w-[200px]">
                        {customerName}
                      </td>
                      <td className="px-3 py-2 text-right font-mono">
                        {formatMoney(inv.total, inv.currency)}
                      </td>
                      <td className="px-3 py-2 text-right font-mono">
                        {formatMoney(inv.paid_amount, inv.currency)}
                      </td>
                      <td className="px-3 py-2">
                        <span
                          className={`inline-block rounded-md px-2 py-0.5 text-xs ${invoiceStatusBadgeClass(
                            inv.status,
                          )}`}
                        >
                          {invoiceStatusLabel(inv.status)}
                        </span>
                      </td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          </div>
        )}
      </section>
    </div>
  );
}

const toneClasses: Record<string, string> = {
  green: "bg-green-50 border-green-200 text-green-800",
  amber: "bg-amber-50 border-amber-200 text-amber-800",
  red: "bg-red-50 border-red-200 text-red-800",
  gray: "bg-card border-border text-foreground",
};

function StatCard({
  label,
  tooltip,
  amounts,
  tone,
}: {
  label: string;
  tooltip?: string;
  amounts: CurrencyAmount[];
  tone: "green" | "amber" | "red" | "gray";
}) {
  return (
    <div className={`rounded-md border p-4 ${toneClasses[tone]}`} title={tooltip}>
      <div className="text-xs font-medium opacity-80">{label}</div>
      {amounts.length === 0 ? (
        <div className="mt-2 text-2xl font-semibold">—</div>
      ) : (
        <div className="mt-2 space-y-1">
          {amounts.map((a) => (
            <div key={a.currency} className="text-xl font-semibold font-mono">
              {formatMoney(a.amount, a.currency)}
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

function CountCard({
  label,
  count,
  tone,
}: {
  label: string;
  count: number;
  tone: "green" | "amber" | "red" | "gray";
}) {
  return (
    <div className={`rounded-md border p-4 ${toneClasses[tone]}`}>
      <div className="text-xs font-medium opacity-80">{label}</div>
      <div className="mt-2 text-3xl font-semibold">{count}</div>
    </div>
  );
}
