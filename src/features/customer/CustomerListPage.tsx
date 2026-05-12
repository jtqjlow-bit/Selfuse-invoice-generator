import { useEffect, useState } from "react";
import { Link, useNavigate } from "react-router-dom";
import { customerApi } from "@/api/customer";
import type { Customer } from "@/types/bindings/Customer";

export function CustomerListPage() {
  const [items, setItems] = useState<Customer[]>([]);
  const [query, setQuery] = useState("");
  const [includeArchived, setIncludeArchived] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const navigate = useNavigate();

  async function reload() {
    setLoading(true);
    setError(null);
    try {
      const list = query.trim()
        ? await customerApi.search(query, includeArchived)
        : await customerApi.list(includeArchived);
      setItems(list);
    } catch (e) {
      setError(formatErr(e));
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => {
    reload();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [includeArchived]);

  async function onArchiveToggle(c: Customer) {
    try {
      if (c.archived) {
        await customerApi.unarchive(c.id);
      } else {
        await customerApi.archive(c.id);
      }
      await reload();
    } catch (e) {
      setError(formatErr(e));
    }
  }

  return (
    <div className="p-8">
      <div className="mb-6 flex items-center justify-between">
        <h1 className="text-2xl font-semibold">客户</h1>
        <Link
          to="/customers/new"
          className="rounded-md bg-primary px-4 py-2 text-sm text-primary-foreground hover:opacity-90"
        >
          + 新增客户
        </Link>
      </div>

      <form
        className="mb-4 flex items-center gap-3"
        onSubmit={(e) => {
          e.preventDefault();
          reload();
        }}
      >
        <input
          type="search"
          placeholder="搜索名字、联系人、邮箱、电话、SSM、NRIC…"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          className="flex-1 rounded-md border border-input bg-background px-3 py-2 text-sm"
        />
        <button
          type="submit"
          className="rounded-md border border-input px-3 py-2 text-sm hover:bg-accent"
        >
          搜索
        </button>
        <label className="flex items-center gap-2 text-sm text-muted-foreground">
          <input
            type="checkbox"
            checked={includeArchived}
            onChange={(e) => setIncludeArchived(e.target.checked)}
          />
          包含已归档
        </label>
      </form>

      {error && <p className="mb-3 text-sm text-red-600">{error}</p>}

      {loading ? (
        <p className="text-muted-foreground">加载中…</p>
      ) : items.length === 0 ? (
        <p className="text-muted-foreground">还没有客户。点右上角"新增客户"开始。</p>
      ) : (
        <div className="overflow-hidden rounded-md border border-border">
          <table className="w-full text-sm">
            <thead className="bg-muted text-left text-muted-foreground">
              <tr>
                <th className="px-3 py-2 font-medium">名字</th>
                <th className="px-3 py-2 font-medium">类型</th>
                <th className="px-3 py-2 font-medium">联系人</th>
                <th className="px-3 py-2 font-medium">Email</th>
                <th className="px-3 py-2 font-medium">电话</th>
                <th className="px-3 py-2 font-medium">状态</th>
                <th className="px-3 py-2 font-medium">操作</th>
              </tr>
            </thead>
            <tbody>
              {items.map((c) => (
                <tr key={c.id} className="border-t border-border">
                  <td className="px-3 py-2">{c.name}</td>
                  <td className="px-3 py-2 text-muted-foreground">
                    {c.type === "Company" ? "公司" : "个人"}
                  </td>
                  <td className="px-3 py-2 text-muted-foreground">
                    {c.contact_person ?? "—"}
                  </td>
                  <td className="px-3 py-2 text-muted-foreground">
                    {c.email ?? "—"}
                  </td>
                  <td className="px-3 py-2 text-muted-foreground">
                    {c.phone ?? "—"}
                  </td>
                  <td className="px-3 py-2">
                    {c.archived ? (
                      <span className="text-xs text-muted-foreground">已归档</span>
                    ) : (
                      <span className="text-xs text-green-600">活跃</span>
                    )}
                  </td>
                  <td className="px-3 py-2">
                    <div className="flex gap-2">
                      <button
                        onClick={() => navigate(`/customers/${c.id}`)}
                        className="text-xs text-primary hover:underline"
                      >
                        编辑
                      </button>
                      <button
                        onClick={() => onArchiveToggle(c)}
                        className="text-xs text-muted-foreground hover:underline"
                      >
                        {c.archived ? "取消归档" : "归档"}
                      </button>
                    </div>
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

function formatErr(e: unknown): string {
  if (e && typeof e === "object" && "message" in e) {
    return String((e as { message: string }).message);
  }
  return String(e);
}
