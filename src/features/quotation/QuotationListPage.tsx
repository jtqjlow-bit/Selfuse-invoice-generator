import { useEffect, useMemo, useState } from "react";
import { Link, useNavigate } from "react-router-dom";
import { quotationApi } from "@/api/quotation";
import { ListFilterBar } from "@/common/components/ListFilterBar";
import { customerSnapshotName, formatErr, formatMoney } from "@/common/utils/format";
import { inDateRange, matchesSearch } from "@/common/utils/listFilter";
import type { Quotation } from "@/types/bindings/Quotation";
import type { QuotationStatus } from "@/types/bindings/QuotationStatus";
import {
  quotationStatusBadgeClass,
  quotationStatusLabel,
} from "./statusLabel";

const STATUS_VALUES: QuotationStatus[] = [
  "Draft",
  "Sent",
  "Accepted",
  "Rejected",
  "Expired",
];

export function QuotationListPage() {
  const [items, setItems] = useState<Quotation[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const navigate = useNavigate();

  const [search, setSearch] = useState("");
  const [status, setStatus] = useState("");
  const [dateFrom, setDateFrom] = useState("");
  const [dateTo, setDateTo] = useState("");

  useEffect(() => {
    quotationApi
      .list()
      .then(setItems)
      .catch((e) => setError(formatErr(e)))
      .finally(() => setLoading(false));
  }, []);

  const filtered = useMemo(
    () =>
      items.filter((q) => {
        const name = customerSnapshotName(q.customer_snapshot);
        if (!matchesSearch(name, q.total, search)) return false;
        if (status && q.status !== status) return false;
        if (!inDateRange(q.issue_date, dateFrom, dateTo)) return false;
        return true;
      }),
    [items, search, status, dateFrom, dateTo],
  );

  const resetFilters = () => {
    setSearch("");
    setStatus("");
    setDateFrom("");
    setDateTo("");
  };

  return (
    <div className="p-8">
      <div className="mb-6 flex items-center justify-between">
        <h1 className="text-2xl font-semibold">Quotation</h1>
        <Link
          to="/quotations/new"
          className="rounded-md bg-primary px-4 py-2 text-sm text-primary-foreground hover:opacity-90"
        >
          + New Quotation
        </Link>
      </div>

      {error && <p className="mb-3 text-sm text-red-600">{error}</p>}

      {loading ? (
        <p className="text-muted-foreground">加载中…</p>
      ) : items.length === 0 ? (
        <p className="text-muted-foreground">
          还没有 Quotation。点右上角"+ New Quotation"开始。
        </p>
      ) : (
        <>
          <ListFilterBar
            search={search}
            onSearch={setSearch}
            dateLabel="出票日期"
            dateFrom={dateFrom}
            onDateFrom={setDateFrom}
            dateTo={dateTo}
            onDateTo={setDateTo}
            statusOptions={STATUS_VALUES.map((s) => ({
              value: s,
              label: quotationStatusLabel(s),
            }))}
            status={status}
            onStatus={setStatus}
            onReset={resetFilters}
            resultCount={filtered.length}
            totalCount={items.length}
          />
          {filtered.length === 0 ? (
            <p className="text-muted-foreground">没有匹配的 Quotation。</p>
          ) : (
            <div className="overflow-hidden rounded-md border border-border">
              <table className="w-full text-sm">
                <thead className="bg-muted text-left text-muted-foreground">
                  <tr>
                    <th className="px-3 py-2 font-medium">编号</th>
                    <th className="px-3 py-2 font-medium">客户</th>
                    <th className="px-3 py-2 font-medium">出票日期</th>
                    <th className="px-3 py-2 font-medium">有效期至</th>
                    <th className="px-3 py-2 text-right font-medium">合计</th>
                    <th className="px-3 py-2 font-medium">状态</th>
                    <th className="px-3 py-2 font-medium">操作</th>
                  </tr>
                </thead>
                <tbody>
                  {filtered.map((q) => (
                <tr key={q.id} className="border-t border-border">
                  <td className="px-3 py-2 font-mono">{q.number}</td>
                  <td className="px-3 py-2">
                    {customerSnapshotName(q.customer_snapshot)}
                  </td>
                  <td className="px-3 py-2 text-muted-foreground">
                    {q.issue_date}
                  </td>
                  <td className="px-3 py-2 text-muted-foreground">
                    {q.valid_until}
                  </td>
                  <td className="px-3 py-2 text-right font-mono">
                    {formatMoney(q.total, q.currency)}
                  </td>
                  <td className="px-3 py-2">
                    <span
                      className={`inline-block rounded-md px-2 py-0.5 text-xs ${quotationStatusBadgeClass(
                        q.status,
                      )}`}
                    >
                      {quotationStatusLabel(q.status)}
                    </span>
                  </td>
                  <td className="px-3 py-2">
                    <button
                      onClick={() => navigate(`/quotations/${q.id}`)}
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
        </>
      )}
    </div>
  );
}
