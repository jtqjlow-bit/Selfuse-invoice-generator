import { useEffect, useMemo, useState, type FormEvent } from "react";
import { useNavigate, useParams } from "react-router-dom";
import { quotationApi } from "@/api/quotation";
import { invoiceApi } from "@/api/invoice";
import { customerApi } from "@/api/customer";
import { businessProfileApi } from "@/api/business_profile";
import { pdfRenderApi } from "@/api/pdf";
import { CurrencySelect } from "@/common/components/CurrencySelect";
import { PdfPreviewPanel } from "@/common/components/PdfPreviewPanel";
import { QuotationPreview } from "@/common/components/preview/QuotationPreview";
import {
  customerSnapshotName,
  formatErr,
  formatMoney,
  sanitizeFilenamePart,
} from "@/common/utils/format";
import type { BusinessProfile } from "@/types/bindings/BusinessProfile";
import type { Customer } from "@/types/bindings/Customer";
import type { LineItemInput } from "@/types/bindings/LineItemInput";
import type { ProfileAssetDataUrls } from "@/types/bindings/ProfileAssetDataUrls";
import type { Quotation } from "@/types/bindings/Quotation";
import type { QuotationStatus } from "@/types/bindings/QuotationStatus";
import type { QuotationWithLines } from "@/types/bindings/QuotationWithLines";
import {
  quotationStatusBadgeClass,
  quotationStatusLabel,
} from "./statusLabel";

type Mode = "create" | "edit";

interface FormState {
  customer_id: string;
  business_profile_id: string;
  issue_date: string;
  valid_until: string;
  currency: string;
  tax_enabled: boolean;
  tax_rate: number | null;
  lines: LineItemInput[];
  notes: string;
  terms: string;
}

function todayIso(): string {
  const d = new Date();
  const yyyy = d.getFullYear();
  const mm = String(d.getMonth() + 1).padStart(2, "0");
  const dd = String(d.getDate()).padStart(2, "0");
  return `${yyyy}-${mm}-${dd}`;
}

function addDaysIso(iso: string, days: number): string {
  const [y, m, d] = iso.split("-").map(Number);
  const date = new Date(y, (m ?? 1) - 1, d);
  date.setDate(date.getDate() + days);
  const yyyy = date.getFullYear();
  const mm = String(date.getMonth() + 1).padStart(2, "0");
  const dd = String(date.getDate()).padStart(2, "0");
  return `${yyyy}-${mm}-${dd}`;
}

function emptyLine(): LineItemInput {
  return { description: "", quantity: 1, unit_price: 0 };
}

function emptyForm(): FormState {
  return {
    customer_id: "",
    business_profile_id: "",
    issue_date: todayIso(),
    valid_until: addDaysIso(todayIso(), 30),
    currency: "MYR",
    tax_enabled: false,
    tax_rate: null,
    lines: [emptyLine()],
    notes: "",
    terms: "",
  };
}

function fromQuotation(qwl: QuotationWithLines): FormState {
  return {
    customer_id: qwl.quotation.customer_id,
    business_profile_id: qwl.quotation.business_profile_id ?? "",
    issue_date: qwl.quotation.issue_date,
    valid_until: qwl.quotation.valid_until,
    currency: qwl.quotation.currency,
    tax_enabled: qwl.quotation.tax_enabled,
    tax_rate: qwl.quotation.tax_rate,
    lines: qwl.lines.map((l) => ({
      description: l.description,
      quantity: l.quantity,
      unit_price: l.unit_price,
    })),
    notes: qwl.quotation.notes ?? "",
    terms: qwl.quotation.terms ?? "",
  };
}

export function QuotationFormPage() {
  const { id } = useParams<{ id?: string }>();
  const mode: Mode = id ? "edit" : "create";
  const navigate = useNavigate();

  const [form, setForm] = useState<FormState>(emptyForm());
  const [customers, setCustomers] = useState<Customer[]>([]);
  const [profiles, setProfiles] = useState<BusinessProfile[]>([]);
  const [existing, setExisting] = useState<Quotation | null>(null);
  const [assets, setAssets] = useState<ProfileAssetDataUrls | null>(null);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Reload profile assets whenever the selected business profile changes.
  // This is the one IPC + file-read cost in the new preview pipeline; per
  // keystroke afterwards the preview is pure in-process React.
  useEffect(() => {
    if (!form.business_profile_id) {
      setAssets(null);
      return;
    }
    let cancelled = false;
    businessProfileApi
      .getAssetDataUrls(form.business_profile_id)
      .then((a) => {
        if (!cancelled) setAssets(a);
      })
      .catch(() => {
        if (!cancelled) setAssets(null);
      });
    return () => {
      cancelled = true;
    };
  }, [form.business_profile_id]);

  // Load customers + business profiles + (edit-mode) existing quotation
  useEffect(() => {
    async function load() {
      try {
        const [activeCustomers, profileList] = await Promise.all([
          customerApi.list(false),
          businessProfileApi.list(),
        ]);
        setCustomers(activeCustomers);
        setProfiles(profileList);

        if (mode === "edit" && id) {
          const qwl = await quotationApi.findById(id);
          setForm(fromQuotation(qwl));
          setExisting(qwl.quotation);
        } else if (profileList.length > 0) {
          // Create mode — pick the first profile and seed defaults from it.
          const p = profileList[0];
          const today = todayIso();
          setForm((prev) => ({
            ...prev,
            business_profile_id: p.id,
            issue_date: today,
            valid_until: addDaysIso(today, p.default_quotation_valid_days),
            tax_enabled: p.default_tax_rate != null,
            tax_rate: p.default_tax_rate,
          }));
        }
      } catch (e) {
        setError(formatErr(e));
      } finally {
        setLoading(false);
      }
    }
    load();
  }, [id, mode]);

  const status: QuotationStatus = existing?.status ?? "Draft";
  const isDraft = status === "Draft";
  const isSent = status === "Sent";
  const isAccepted = status === "Accepted";
  const canConvert =
    isAccepted && existing != null && existing.converted_invoice_id == null;
  // Form is editable only in create mode or Draft edit mode
  const editable = mode === "create" || isDraft;

  const totals = useMemo(() => {
    const subtotal = form.lines.reduce(
      (acc, l) => acc + (Number(l.quantity) || 0) * (Number(l.unit_price) || 0),
      0,
    );
    const taxAmount = form.tax_enabled
      ? subtotal * (form.tax_rate ?? 0)
      : 0;
    return { subtotal, taxAmount, total: subtotal + taxAmount };
  }, [form.lines, form.tax_enabled, form.tax_rate]);

  function updateField<K extends keyof FormState>(k: K, v: FormState[K]) {
    setForm((p) => ({ ...p, [k]: v }));
  }

  function updateLine(i: number, patch: Partial<LineItemInput>) {
    setForm((p) => ({
      ...p,
      lines: p.lines.map((l, idx) => (idx === i ? { ...l, ...patch } : l)),
    }));
  }

  function addLine() {
    setForm((p) => ({ ...p, lines: [...p.lines, emptyLine()] }));
  }

  function removeLine(i: number) {
    setForm((p) => ({
      ...p,
      lines: p.lines.length > 1 ? p.lines.filter((_, idx) => idx !== i) : p.lines,
    }));
  }

  async function onSave(e: FormEvent, andMarkSent: boolean) {
    e.preventDefault();
    setSaving(true);
    setError(null);
    try {
      const payload = {
        customer_id: form.customer_id,
        business_profile_id: form.business_profile_id || null,
        issue_date: form.issue_date,
        valid_until: form.valid_until,
        currency: form.currency,
        tax_enabled: form.tax_enabled,
        tax_rate: form.tax_enabled ? form.tax_rate : null,
        lines: form.lines.map((l) => ({
          description: l.description,
          quantity: Number(l.quantity) || 0,
          unit_price: Number(l.unit_price) || 0,
        })),
        notes: form.notes.trim() || null,
        terms: form.terms.trim() || null,
      };
      let savedId: string;
      if (mode === "create") {
        const saved = await quotationApi.create(payload);
        savedId = saved.quotation.id;
      } else if (id) {
        const saved = await quotationApi.update({ id, ...payload });
        savedId = saved.quotation.id;
      } else {
        throw new Error("missing id in edit mode");
      }
      if (andMarkSent) {
        await quotationApi.markSent(savedId);
      }
      navigate("/quotations");
    } catch (e) {
      setError(formatErr(e));
      setSaving(false);
    }
  }

  async function onConvertToInvoice() {
    if (!existing) return;
    if (
      !window.confirm(
        `确认从这份 Quotation 创建 Invoice？到期日会默认取所属公司资料的 default_invoice_due_days。`,
      )
    ) {
      return;
    }
    setSaving(true);
    setError(null);
    try {
      // Pull due-days default from the quotation's own profile (fallback 30).
      const dueDays =
        profiles.find((p) => p.id === existing.business_profile_id)
          ?.default_invoice_due_days ?? 30;
      const today = todayIso();
      const r = await invoiceApi.createFromQuotation({
        quotation_id: existing.id,
        business_profile_id: existing.business_profile_id,
        issue_date: today,
        due_date: addDaysIso(today, dueDays),
      });
      navigate(`/invoices/${r.invoice.id}`);
    } catch (e) {
      setError(formatErr(e));
      setSaving(false);
    }
  }

  async function onTransition(action: "Accepted" | "Rejected" | "Expired") {
    if (!id || !existing) return;
    const label =
      action === "Accepted" ? "已接受" : action === "Rejected" ? "已拒绝" : "已过期";
    if (!window.confirm(`确认标记为${label}？这一步不可逆（不能再变回已发送/草稿）。`)) {
      return;
    }
    setSaving(true);
    setError(null);
    try {
      if (action === "Accepted") await quotationApi.markAccepted(id);
      else if (action === "Rejected") await quotationApi.markRejected(id);
      else await quotationApi.markExpired(id);
      navigate("/quotations");
    } catch (e) {
      setError(formatErr(e));
      setSaving(false);
    }
  }

  const selectedProfile = useMemo(
    () => profiles.find((p) => p.id === form.business_profile_id) ?? null,
    [profiles, form.business_profile_id],
  );
  const selectedCustomer = useMemo(
    () => customers.find((c) => c.id === form.customer_id) ?? null,
    [customers, form.customer_id],
  );

  if (loading) {
    return <p className="p-8 text-muted-foreground">加载中…</p>;
  }

  const canSave =
    form.customer_id !== "" &&
    form.business_profile_id !== "" &&
    form.lines.length > 0 &&
    form.lines.every((l) => l.description.trim() !== "" && Number(l.quantity) > 0);
  const canPreview = form.customer_id !== "" && form.lines.length > 0;

  return (
    <div className="flex h-full">
      <div className="flex-1 overflow-y-auto p-6">
        <div className="mb-6 flex items-center justify-between">
        <h1 className="text-2xl font-semibold">
          {mode === "create"
            ? "New Quotation"
            : `Quotation ${existing?.number ?? ""}`}
        </h1>
        {existing && (
          <span
            className={`inline-block rounded-md px-3 py-1 text-sm ${quotationStatusBadgeClass(
              existing.status,
            )}`}
          >
            {quotationStatusLabel(existing.status)}
          </span>
        )}
      </div>

      <form onSubmit={(e) => onSave(e, false)} className="space-y-5">
        <fieldset disabled={!editable} className="space-y-5 disabled:opacity-70">
          <Field label="出单方 (公司资料) *">
            <select
              className={inputCls}
              value={form.business_profile_id}
              onChange={(e) => updateField("business_profile_id", e.target.value)}
              required
            >
              <option value="">— 请选择 —</option>
              {profiles.map((p) => (
                <option key={p.id} value={p.id}>
                  {p.name} ({p.entity_type === "Company" ? "公司" : "个人"})
                </option>
              ))}
            </select>
            {profiles.length === 0 && (
              <p className="mt-1 text-xs text-orange-600">
                还没有公司资料 — 请先到「公司资料」新增一份。
              </p>
            )}
          </Field>

          <Field label="客户 *">
            <select
              className={inputCls}
              value={form.customer_id}
              onChange={(e) => updateField("customer_id", e.target.value)}
              required
            >
              <option value="">— 请选择 —</option>
              {customers.map((c) => (
                <option key={c.id} value={c.id}>
                  {c.name} ({c.type === "Company" ? "公司" : "个人"})
                </option>
              ))}
            </select>
          </Field>

          <div className="grid grid-cols-3 gap-4">
            <Field label="出票日期 *">
              <input
                type="date"
                className={inputCls}
                value={form.issue_date}
                onChange={(e) => updateField("issue_date", e.target.value)}
                required
              />
            </Field>
            <Field label="有效期至 *">
              <input
                type="date"
                className={inputCls}
                value={form.valid_until}
                onChange={(e) => updateField("valid_until", e.target.value)}
                required
              />
            </Field>
            <Field label="货币">
              <CurrencySelect
                className={inputCls}
                value={form.currency}
                onChange={(v) => updateField("currency", v)}
                disabled={!editable}
              />
            </Field>
          </div>

          <div>
            <div className="mb-2 flex items-center justify-between">
              <label className="text-sm font-medium">行项目 *</label>
              <button
                type="button"
                onClick={addLine}
                disabled={!editable}
                className="text-xs text-primary hover:underline disabled:opacity-50"
              >
                + 添加一行
              </button>
            </div>
            <div className="overflow-hidden rounded-md border border-border">
              <table className="w-full text-sm">
                <thead className="bg-muted text-left text-muted-foreground">
                  <tr>
                    <th className="px-3 py-2 font-medium">描述</th>
                    <th className="w-24 px-3 py-2 font-medium">数量</th>
                    <th className="w-32 px-3 py-2 font-medium">单价</th>
                    <th className="w-36 px-3 py-2 text-right font-medium">
                      小计
                    </th>
                    <th className="w-10 px-3 py-2"></th>
                  </tr>
                </thead>
                <tbody>
                  {form.lines.map((line, i) => (
                    <tr key={i} className="border-t border-border">
                      <td className="px-3 py-2">
                        <textarea
                          rows={2}
                          className={inputCls + " min-h-10 resize-y"}
                          value={line.description}
                          onChange={(e) =>
                            updateLine(i, { description: e.target.value })
                          }
                          disabled={!editable}
                        />
                      </td>
                      <td className="px-3 py-2">
                        <input
                          type="number"
                          step="0.01"
                          min="0"
                          className={inputCls}
                          value={line.quantity}
                          onChange={(e) =>
                            updateLine(i, { quantity: Number(e.target.value) })
                          }
                          disabled={!editable}
                        />
                      </td>
                      <td className="px-3 py-2">
                        <input
                          type="number"
                          step="0.01"
                          min="0"
                          className={inputCls}
                          value={line.unit_price}
                          onChange={(e) =>
                            updateLine(i, { unit_price: Number(e.target.value) })
                          }
                          disabled={!editable}
                        />
                      </td>
                      <td className="px-3 py-2 text-right font-mono">
                        {formatMoney(
                          (Number(line.quantity) || 0) *
                            (Number(line.unit_price) || 0),
                          form.currency,
                        )}
                      </td>
                      <td className="px-3 py-2 text-center">
                        {form.lines.length > 1 && editable && (
                          <button
                            type="button"
                            onClick={() => removeLine(i)}
                            title="删除此行"
                            className="text-muted-foreground hover:text-red-600"
                          >
                            ×
                          </button>
                        )}
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </div>

          <div className="rounded-md border border-border p-4 text-sm">
            <div className="flex items-center gap-3">
              <label className="flex items-center gap-2">
                <input
                  type="checkbox"
                  checked={form.tax_enabled}
                  onChange={(e) =>
                    updateField("tax_enabled", e.target.checked)
                  }
                  disabled={!editable}
                />
                启用税
              </label>
              {form.tax_enabled && (
                <label className="flex items-center gap-2 text-muted-foreground">
                  税率（0.0 ~ 1.0）：
                  <input
                    type="number"
                    step="0.01"
                    min="0"
                    max="1"
                    className={inputCls + " w-24"}
                    value={form.tax_rate ?? ""}
                    onChange={(e) =>
                      updateField(
                        "tax_rate",
                        e.target.value === "" ? null : Number(e.target.value),
                      )
                    }
                    disabled={!editable}
                  />
                </label>
              )}
            </div>
            <div className="mt-3 grid grid-cols-3 gap-3 border-t border-border pt-3 font-mono text-right">
              <div>
                <div className="text-xs font-sans text-muted-foreground">
                  小计
                </div>
                <div>{formatMoney(totals.subtotal, form.currency)}</div>
              </div>
              <div>
                <div className="text-xs font-sans text-muted-foreground">税额</div>
                <div>{formatMoney(totals.taxAmount, form.currency)}</div>
              </div>
              <div>
                <div className="text-xs font-sans text-muted-foreground">
                  合计
                </div>
                <div className="text-base font-semibold">
                  {formatMoney(totals.total, form.currency)}
                </div>
              </div>
            </div>
          </div>

          <div className="grid grid-cols-2 gap-4">
            <Field label="备注">
              <textarea
                rows={3}
                className={inputCls + " min-h-16"}
                value={form.notes}
                onChange={(e) => updateField("notes", e.target.value)}
              />
            </Field>
            <Field label="条款">
              <textarea
                rows={3}
                className={inputCls + " min-h-16"}
                value={form.terms}
                onChange={(e) => updateField("terms", e.target.value)}
              />
            </Field>
          </div>
        </fieldset>

        {error && <p className="text-sm text-red-600">{error}</p>}

        <div className="flex flex-wrap items-center justify-between gap-3 pt-2">
          <div className="flex gap-2">
            {editable && (
              <>
                <button
                  type="submit"
                  disabled={!canSave || saving}
                  className="rounded-md bg-primary px-4 py-2 text-sm text-primary-foreground hover:opacity-90 disabled:opacity-50"
                >
                  {saving ? "保存中…" : "保存"}
                </button>
                <button
                  type="button"
                  disabled={!canSave || saving}
                  onClick={(e) => onSave(e, true)}
                  className="rounded-md border border-input px-4 py-2 text-sm hover:bg-accent disabled:opacity-50"
                >
                  保存并标记已发送
                </button>
              </>
            )}
            <button
              type="button"
              onClick={() => navigate("/quotations")}
              className="rounded-md border border-input px-4 py-2 text-sm hover:bg-accent"
            >
              {editable ? "取消" : "返回"}
            </button>
          </div>

          {isSent && (
            <div className="flex gap-2">
              <button
                type="button"
                onClick={() => onTransition("Accepted")}
                disabled={saving}
                className="rounded-md bg-green-600 px-4 py-2 text-sm text-white hover:opacity-90 disabled:opacity-50"
              >
                标记已接受
              </button>
              <button
                type="button"
                onClick={() => onTransition("Rejected")}
                disabled={saving}
                className="rounded-md bg-red-600 px-4 py-2 text-sm text-white hover:opacity-90 disabled:opacity-50"
              >
                标记已拒绝
              </button>
              <button
                type="button"
                onClick={() => onTransition("Expired")}
                disabled={saving}
                className="rounded-md bg-orange-600 px-4 py-2 text-sm text-white hover:opacity-90 disabled:opacity-50"
              >
                标记已过期
              </button>
            </div>
          )}

          {canConvert && (
            <button
              type="button"
              onClick={onConvertToInvoice}
              disabled={saving}
              className="rounded-md bg-primary px-4 py-2 text-sm text-primary-foreground hover:opacity-90 disabled:opacity-50"
            >
              {saving ? "转换中…" : "Convert to Invoice"}
            </button>
          )}

          {isAccepted && existing?.converted_invoice_id && (
            <button
              type="button"
              onClick={() =>
                navigate(`/invoices/${existing.converted_invoice_id}`)
              }
              className="text-sm text-primary hover:underline"
            >
              → 查看已生成的 Invoice
            </button>
          )}
        </div>
      </form>
      </div>
      <div className="w-[45%] min-w-[400px] border-l border-border">
        <PdfPreviewPanel
          docType="Quotation"
          preview={
            <QuotationPreview
              profile={selectedProfile}
              assets={assets}
              customer={selectedCustomer}
              number={existing?.number ?? ""}
              status={existing?.status ?? "Draft"}
              issueDate={form.issue_date}
              validUntil={form.valid_until}
              currency={form.currency}
              taxEnabled={form.tax_enabled}
              taxRate={form.tax_rate}
              lines={form.lines}
              notes={form.notes}
              terms={form.terms}
            />
          }
          canPreview={canPreview}
          notReadyReason="请先选择客户并添加至少一行项目。"
          defaultFilename={
            existing
              ? `${existing.number}-${sanitizeFilenamePart(
                  customerSnapshotName(existing.customer_snapshot),
                )}.pdf`
              : "quotation.pdf"
          }
          onGeneratePdf={
            existing
              ? async (templateId, targetPath) => {
                  await pdfRenderApi.renderQuotation(
                    existing.id,
                    templateId,
                    targetPath,
                  );
                }
              : undefined
          }
        />
      </div>
    </div>
  );
}

const inputCls =
  "w-full rounded-md border border-input bg-background px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-ring disabled:cursor-not-allowed";

function Field({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <label className="block">
      <span className="mb-1 block text-sm text-muted-foreground">{label}</span>
      {children}
    </label>
  );
}
