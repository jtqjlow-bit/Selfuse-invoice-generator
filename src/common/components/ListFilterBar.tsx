export type StatusOption = { value: string; label: string };

interface Props {
  search: string;
  onSearch: (v: string) => void;
  searchPlaceholder?: string;
  dateLabel: string;
  dateFrom: string;
  onDateFrom: (v: string) => void;
  dateTo: string;
  onDateTo: (v: string) => void;
  statusOptions?: StatusOption[];
  status?: string;
  onStatus?: (v: string) => void;
  onReset: () => void;
  resultCount: number;
  totalCount: number;
}

export function ListFilterBar({
  search,
  onSearch,
  searchPlaceholder,
  dateLabel,
  dateFrom,
  onDateFrom,
  dateTo,
  onDateTo,
  statusOptions,
  status,
  onStatus,
  onReset,
  resultCount,
  totalCount,
}: Props) {
  const inputCls =
    "rounded-md border border-input bg-background px-2 py-1 text-sm";
  return (
    <div className="mb-4 flex flex-wrap items-end gap-3 rounded-md border border-border bg-card p-3">
      <div className="flex flex-col gap-1">
        <label className="text-xs text-muted-foreground">搜索</label>
        <input
          value={search}
          onChange={(e) => onSearch(e.target.value)}
          placeholder={searchPlaceholder ?? "客户名 / 金额"}
          className={`w-52 ${inputCls}`}
        />
      </div>

      {statusOptions && onStatus && (
        <div className="flex flex-col gap-1">
          <label className="text-xs text-muted-foreground">状态</label>
          <select
            value={status}
            onChange={(e) => onStatus(e.target.value)}
            className={inputCls}
          >
            <option value="">全部</option>
            {statusOptions.map((o) => (
              <option key={o.value} value={o.value}>
                {o.label}
              </option>
            ))}
          </select>
        </div>
      )}

      <div className="flex flex-col gap-1">
        <label className="text-xs text-muted-foreground">{dateLabel}从</label>
        <input
          type="date"
          value={dateFrom}
          onChange={(e) => onDateFrom(e.target.value)}
          className={inputCls}
        />
      </div>
      <div className="flex flex-col gap-1">
        <label className="text-xs text-muted-foreground">{dateLabel}至</label>
        <input
          type="date"
          value={dateTo}
          onChange={(e) => onDateTo(e.target.value)}
          className={inputCls}
        />
      </div>

      <button
        type="button"
        onClick={onReset}
        className="rounded-md border border-input bg-background px-3 py-1.5 text-sm hover:bg-accent"
      >
        清除
      </button>

      <span className="ml-auto text-xs text-muted-foreground">
        显示 {resultCount} / {totalCount}
      </span>
    </div>
  );
}
