import { useEffect, useState } from "react";
import { currencyApi } from "@/api/currency";
import { formatErr } from "@/common/utils/format";
import type { ExchangeRate } from "@/types/bindings/ExchangeRate";

export function SettingsPage() {
  return (
    <div className="mx-auto max-w-3xl p-8">
      <h1 className="mb-1 text-2xl font-semibold">设置</h1>
      <p className="mb-6 text-sm text-muted-foreground">汇率管理</p>
      <CurrencySection />
    </div>
  );
}

function CurrencySection() {
  const [rates, setRates] = useState<ExchangeRate[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const reload = () => {
    setLoading(true);
    currencyApi
      .listCached()
      .then(setRates)
      .catch((e) => setError(formatErr(e)))
      .finally(() => setLoading(false));
  };

  useEffect(reload, []);

  const refreshAll = async () => {
    setBusy(true);
    setError(null);
    try {
      await currencyApi.refresh();
      reload();
    } catch (e) {
      setError(formatErr(e));
    } finally {
      setBusy(false);
    }
  };

  return (
    <section className="space-y-6">
      <FetchRateForm onDone={reload} onError={setError} />
      <ConvertTool onError={setError} />

      {error && <p className="text-sm text-red-600">{error}</p>}

      <div>
        <div className="mb-2 flex items-center justify-between">
          <h2 className="text-sm font-semibold">已缓存汇率</h2>
          <button
            type="button"
            onClick={refreshAll}
            disabled={busy || rates.length === 0}
            className="rounded-md border border-input bg-background px-3 py-1.5 text-sm hover:bg-accent disabled:opacity-50"
          >
            {busy ? "刷新中…" : "刷新全部"}
          </button>
        </div>
        {loading ? (
          <p className="text-sm text-muted-foreground">加载中…</p>
        ) : rates.length === 0 ? (
          <p className="rounded-md border border-border bg-card p-4 text-sm text-muted-foreground">
            还没有缓存任何汇率，用上面的「查询并缓存」添加一对货币。
          </p>
        ) : (
          <div className="overflow-hidden rounded-md border border-border">
            <table className="w-full text-sm">
              <thead className="bg-muted text-left text-muted-foreground">
                <tr>
                  <th className="px-3 py-2 font-medium">货币对</th>
                  <th className="px-3 py-2 text-right font-medium">汇率</th>
                  <th className="px-3 py-2 font-medium">更新时间</th>
                </tr>
              </thead>
              <tbody>
                {rates.map((r) => (
                  <tr key={`${r.base}-${r.target}`} className="border-t border-border">
                    <td className="px-3 py-2 font-mono">
                      {r.base} → {r.target}
                    </td>
                    <td className="px-3 py-2 text-right font-mono">{r.rate}</td>
                    <td className="px-3 py-2 text-muted-foreground">
                      {new Date(r.fetched_at).toLocaleString()}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>
    </section>
  );
}

function FetchRateForm({
  onDone,
  onError,
}: {
  onDone: () => void;
  onError: (msg: string) => void;
}) {
  const [from, setFrom] = useState("USD");
  const [to, setTo] = useState("MYR");
  const [busy, setBusy] = useState(false);

  const submit = async (e: React.FormEvent) => {
    e.preventDefault();
    setBusy(true);
    onError("");
    try {
      await currencyApi.getRate(from, to);
      onDone();
    } catch (err) {
      onError(formatErr(err));
    } finally {
      setBusy(false);
    }
  };

  return (
    <form onSubmit={submit} className="rounded-md border border-border bg-card p-4">
      <h2 className="mb-3 text-sm font-semibold">查询并缓存</h2>
      <div className="flex flex-wrap items-end gap-2">
        <CodeInput label="从" value={from} onChange={setFrom} />
        <CodeInput label="到" value={to} onChange={setTo} />
        <button
          type="submit"
          disabled={busy}
          className="rounded-md bg-primary px-3 py-1.5 text-sm text-primary-foreground hover:opacity-90 disabled:opacity-50"
        >
          {busy ? "查询中…" : "查询"}
        </button>
      </div>
    </form>
  );
}

function ConvertTool({ onError }: { onError: (msg: string) => void }) {
  const [amount, setAmount] = useState("100");
  const [from, setFrom] = useState("USD");
  const [to, setTo] = useState("MYR");
  const [result, setResult] = useState<number | null>(null);
  const [busy, setBusy] = useState(false);

  const submit = async (e: React.FormEvent) => {
    e.preventDefault();
    setBusy(true);
    onError("");
    setResult(null);
    try {
      const r = await currencyApi.convert(Number(amount), from, to);
      setResult(r);
    } catch (err) {
      onError(formatErr(err));
    } finally {
      setBusy(false);
    }
  };

  return (
    <form onSubmit={submit} className="rounded-md border border-border bg-card p-4">
      <h2 className="mb-3 text-sm font-semibold">换算</h2>
      <div className="flex flex-wrap items-end gap-2">
        <div className="flex flex-col gap-1">
          <label className="text-xs text-muted-foreground">金额</label>
          <input
            type="number"
            step="any"
            value={amount}
            onChange={(e) => setAmount(e.target.value)}
            className="w-28 rounded-md border border-input bg-background px-2 py-1 text-sm"
          />
        </div>
        <CodeInput label="从" value={from} onChange={setFrom} />
        <CodeInput label="到" value={to} onChange={setTo} />
        <button
          type="submit"
          disabled={busy}
          className="rounded-md bg-primary px-3 py-1.5 text-sm text-primary-foreground hover:opacity-90 disabled:opacity-50"
        >
          {busy ? "换算中…" : "换算"}
        </button>
        {result !== null && (
          <span className="ml-2 text-sm font-mono">
            = {to} {result.toLocaleString(undefined, { maximumFractionDigits: 4 })}
          </span>
        )}
      </div>
    </form>
  );
}

function CodeInput({
  label,
  value,
  onChange,
}: {
  label: string;
  value: string;
  onChange: (v: string) => void;
}) {
  return (
    <div className="flex flex-col gap-1">
      <label className="text-xs text-muted-foreground">{label}</label>
      <input
        value={value}
        onChange={(e) => onChange(e.target.value.toUpperCase().slice(0, 3))}
        maxLength={3}
        className="w-20 rounded-md border border-input bg-background px-2 py-1 text-sm font-mono uppercase"
      />
    </div>
  );
}
