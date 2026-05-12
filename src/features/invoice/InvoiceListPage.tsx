import { useEffect, useState } from "react";
import { Link, useNavigate } from "react-router-dom";
import { invoiceApi } from "@/api/invoice";
import { customerSnapshotName, formatErr, formatMoney } from "@/common/utils/format";
import type { Invoice } from "@/types/bindings/Invoice";
import { invoiceStatusBadgeClass, invoiceStatusLabel } from "./statusLabel";

export function InvoiceListPage() {
  const [items, setItems] = useState<Invoice[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const navigate = useNavigate();

  useEffect(() => {
    invoiceApi
      .list()
      .then(setItems)
      .catch((e) => setError(formatErr(e)))
      .finally(() => setLoading(false));
  }, []);

  return (
    <div className="p-8">
      <div className="mb-6 flex items-center justify-between">
        <h1 className="text-2xl font-semibold">Invoice</h1>
        <Link
          to="/invoices/new"
          className="rounded-md bg-primary px-4 py-2 text-sm text-primary-foreground hover:opacity-90"
        >
          + New Invoice
        </Link>
      </div>

      {error && <p className="mb-3 text-sm text-red-600">{error}</p>}

      {loading ? (
        <p className="text-muted-foreground">加载中…</p>
      ) : items.length === 0 ? (
        <p className="text-muted-foreground">
          还没有 Invoice。可以从已接受的 Quotation 转换过来，或点右上角"+ New Invoice"从零开始。
        </p>
      ) : (
        <div className="overflow-hidden rounded-md border border-border">
          <table className="w-full text-sm">
            <thead className="bg-muted text-left text-muted-foreground">
              <tr>
                <th className="px-3 py-2 font-medium">编号</th>
                <th className="px-3 py-2 font-medium">客户</th>
                <th className="px-3 py-2 font-medium">出票日期</th>
                <th className="px-3 py-2 font-medium">到期日</th>
                <th className="px-3 py-2 text-right font-medium">合计</th>
                <th className="px-3 py-2 text-right font-medium">已付</th>
                <th className="px-3 py-2 font-medium">状态</th>
                <th className="px-3 py-2 font-medium">操作</th>
              </tr>
            </thead>
            <tbody>
              {items.map((inv) => (
                <tr key={inv.id} className="border-t border-border">
                  <td className="px-3 py-2 font-mono">{inv.number}</td>
                  <td className="px-3 py-2">
                    {customerSnapshotName(inv.customer_snapshot)}
                  </td>
                  <td className="px-3 py-2 text-muted-foreground">
                    {inv.issue_date}
                  </td>
                  <td className="px-3 py-2 text-muted-foreground">
                    {inv.due_date}
                  </td>
                  <td className="px-3 py-2 text-right font-mono">
                    {formatMoney(inv.total, inv.currency)}
                  </td>
                  <td className="px-3 py-2 text-right font-mono text-muted-foreground">
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
                  <td className="px-3 py-2">
                    <button
                      onClick={() => navigate(`/invoices/${inv.id}`)}
                      className="text-xs text-primary hover:underline"
                    >
                      查看
                    </button>
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
