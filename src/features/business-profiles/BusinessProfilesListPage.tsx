import { useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import { businessProfileApi } from "@/api/business_profile";
import { formatErr } from "@/common/utils/format";
import type { BusinessProfile } from "@/types/bindings/BusinessProfile";

export function BusinessProfilesListPage() {
  const [items, setItems] = useState<BusinessProfile[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const navigate = useNavigate();

  function reload() {
    setLoading(true);
    businessProfileApi
      .list()
      .then(setItems)
      .catch((e) => setError(formatErr(e)))
      .finally(() => setLoading(false));
  }

  useEffect(reload, []);

  async function onDelete(p: BusinessProfile) {
    if (!window.confirm(`确认删除 "${p.name}"？`)) return;
    try {
      await businessProfileApi.delete(p.id);
      reload();
    } catch (e) {
      setError(formatErr(e));
    }
  }

  return (
    <div className="mx-auto max-w-4xl p-8">
      <div className="mb-6 flex items-center justify-between">
        <h1 className="text-2xl font-semibold">公司资料</h1>
        <button
          type="button"
          onClick={() => navigate("/business-profiles/new")}
          className="rounded-md bg-primary px-3 py-1.5 text-sm text-primary-foreground hover:opacity-90"
        >
          + 新增
        </button>
      </div>

      <p className="mb-4 text-sm text-muted-foreground">
        每张 Quotation / Invoice / Payment Voucher 都要选一份"出单方"。可以为不同业务（公司、个人接单）建多份。
      </p>

      {error && <p className="mb-3 text-sm text-red-600">{error}</p>}

      {loading ? (
        <p className="text-muted-foreground">加载中…</p>
      ) : items.length === 0 ? (
        <p className="text-muted-foreground">
          还没有任何资料。点右上"+ 新增"开始。
        </p>
      ) : (
        <div className="overflow-hidden rounded-md border border-border">
          <table className="w-full text-sm">
            <thead className="bg-muted text-left text-muted-foreground">
              <tr>
                <th className="px-3 py-2 font-medium">名称</th>
                <th className="w-24 px-3 py-2 font-medium">类型</th>
                <th className="w-48 px-3 py-2 font-medium">ID</th>
                <th className="w-40 px-3 py-2 text-right font-medium">操作</th>
              </tr>
            </thead>
            <tbody>
              {items.map((p) => (
                <tr key={p.id} className="border-t border-border">
                  <td className="px-3 py-2">{p.name}</td>
                  <td className="px-3 py-2">
                    {p.entity_type === "Company" ? (
                      <span className="rounded-md bg-blue-100 px-2 py-0.5 text-xs text-blue-700">
                        公司
                      </span>
                    ) : (
                      <span className="rounded-md bg-purple-100 px-2 py-0.5 text-xs text-purple-700">
                        个人
                      </span>
                    )}
                  </td>
                  <td className="px-3 py-2 font-mono text-xs text-muted-foreground">
                    {p.entity_type === "Company"
                      ? `SSM: ${p.ssm_no ?? "—"}`
                      : `NRIC: ${p.nric ?? "—"}`}
                  </td>
                  <td className="px-3 py-2 text-right">
                    <button
                      onClick={() => navigate(`/business-profiles/${p.id}`)}
                      className="text-xs text-primary hover:underline"
                    >
                      编辑
                    </button>
                    <button
                      onClick={() => onDelete(p)}
                      className="ml-3 text-xs text-red-600 hover:underline"
                    >
                      删除
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
