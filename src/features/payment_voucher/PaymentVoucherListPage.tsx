import { useEffect, useMemo, useState } from "react";
import { useNavigate } from "react-router-dom";
import { paymentVoucherApi } from "@/api/payment_voucher";
import { ListFilterBar } from "@/common/components/ListFilterBar";
import { customerSnapshotName, formatErr, formatMoney } from "@/common/utils/format";
import { inDateRange, matchesSearch } from "@/common/utils/listFilter";
import type { PaymentVoucher } from "@/types/bindings/PaymentVoucher";

export function PaymentVoucherListPage() {
  const [items, setItems] = useState<PaymentVoucher[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const navigate = useNavigate();

  const [search, setSearch] = useState("");
  const [dateFrom, setDateFrom] = useState("");
  const [dateTo, setDateTo] = useState("");

  useEffect(() => {
    paymentVoucherApi
      .list()
      .then(setItems)
      .catch((e) => setError(formatErr(e)))
      .finally(() => setLoading(false));
  }, []);

  const filtered = useMemo(
    () =>
      items.filter((pv) => {
        const name = customerSnapshotName(pv.customer_snapshot);
        if (!matchesSearch(name, pv.amount, search)) return false;
        if (!inDateRange(pv.date, dateFrom, dateTo)) return false;
        return true;
      }),
    [items, search, dateFrom, dateTo],
  );

  const resetFilters = () => {
    setSearch("");
    setDateFrom("");
    setDateTo("");
  };

  return (
    <div className="p-8">
      <div className="mb-6 flex items-center justify-between">
        <h1 className="text-2xl font-semibold">Payment Voucher</h1>
        <button
          type="button"
          onClick={() => navigate("/payment-vouchers/new")}
          className="rounded-md bg-primary px-3 py-1.5 text-sm text-primary-foreground hover:opacity-90"
        >
          + 新增 PV
        </button>
      </div>

      {error && <p className="mb-3 text-sm text-red-600">{error}</p>}

      {loading ? (
        <p className="text-muted-foreground">加载中…</p>
      ) : items.length === 0 ? (
        <p className="text-muted-foreground">
          还没有 Payment Voucher。点右上角"+ 新增 PV"开始。
        </p>
      ) : (
        <>
          <ListFilterBar
            search={search}
            onSearch={setSearch}
            dateLabel="日期"
            dateFrom={dateFrom}
            onDateFrom={setDateFrom}
            dateTo={dateTo}
            onDateTo={setDateTo}
            onReset={resetFilters}
            resultCount={filtered.length}
            totalCount={items.length}
          />
          {filtered.length === 0 ? (
            <p className="text-muted-foreground">没有匹配的 Payment Voucher。</p>
          ) : (
            <div className="overflow-hidden rounded-md border border-border">
              <table className="w-full text-sm">
                <thead className="bg-muted text-left text-muted-foreground">
                  <tr>
                    <th className="px-3 py-2 font-medium">编号</th>
                    <th className="px-3 py-2 font-medium">日期</th>
                    <th className="px-3 py-2 font-medium">客户</th>
                    <th className="px-3 py-2 text-right font-medium">金额</th>
                    <th className="px-3 py-2 font-medium">付款方式</th>
                    <th className="px-3 py-2 font-medium">关联 Invoice</th>
                    <th className="px-3 py-2 font-medium">操作</th>
                  </tr>
                </thead>
                <tbody>
                  {filtered.map((pv) => (
                <tr key={pv.id} className="border-t border-border">
                  <td className="px-3 py-2 font-mono">{pv.number}</td>
                  <td className="px-3 py-2 text-muted-foreground">{pv.date}</td>
                  <td className="px-3 py-2">
                    {customerSnapshotName(pv.customer_snapshot)}
                  </td>
                  <td className="px-3 py-2 text-right font-mono">
                    {formatMoney(pv.amount, pv.currency)}
                  </td>
                  <td className="px-3 py-2 text-muted-foreground">
                    {pv.payment_method}
                  </td>
                  <td className="px-3 py-2">
                    {pv.invoice_id ? (
                      <button
                        onClick={() => navigate(`/invoices/${pv.invoice_id}`)}
                        className="text-xs text-primary hover:underline"
                      >
                        查看 Invoice
                      </button>
                    ) : (
                      <span className="text-xs text-muted-foreground">独立</span>
                    )}
                  </td>
                  <td className="px-3 py-2">
                    <button
                      onClick={() => navigate(`/payment-vouchers/${pv.id}`)}
                      className="text-xs text-primary hover:underline"
                    >
                      编辑
                    </button>
                  </td>
                </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}
        </>
      )}
    </div>
  );
}
