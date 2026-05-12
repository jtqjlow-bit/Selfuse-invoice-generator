import { useEffect, useMemo, useState, type FormEvent } from "react";
import { Link, useNavigate, useParams } from "react-router-dom";
import { invoiceApi } from "@/api/invoice";
import { customerApi } from "@/api/customer";
import { businessProfileApi } from "@/api/business_profile";
import { paymentVoucherApi } from "@/api/payment_voucher";
import { PaymentVoucherSection } from "@/features/payment_voucher";
import { pdfRenderApi } from "@/api/pdf";
import { CurrencySelect } from "@/common/components/CurrencySelect";
import { PdfPreviewPanel } from "@/common/components/PdfPreviewPanel";
import { InvoicePreview } from "@/common/components/preview/InvoicePreview";
import {
  customerSnapshotName,
  formatErr,
  formatMoney,
  sanitizeFilenamePart,
} from "@/common/utils/format";
import type { BusinessProfile } from "@/types/bindings/BusinessProfile";
import type { Customer } from "@/types/bindings/Customer";
import type { Invoice } from "@/types/bindings/Invoice";
import type { InvoiceStatus } from "@/types/bindings/InvoiceStatus";
import type { InvoiceWithLines } from "@/types/bindings/InvoiceWithLines";
import type { LineItemInput } from "@/types/bindings/LineItemInput";
import type { PaymentVoucher } from "@/types/bindings/PaymentVoucher";
import type { ProfileAssetDataUrls } from "@/types/bindings/ProfileAssetDataUrls";
import { invoiceStatusBadgeClass, invoiceStatusLabel } from "./statusLabel";

type Mode = "create" | "edit";

interface FormState {
  customer_id: string;
  business_profile_id: string;
  issue_date: string;
  due_date: string;
  currency: string;
  tax_enabled: boolean;
  tax_rate: number | null;
  lines: LineItemInput[];
  notes: string;
  terms: string;
  selected_bank_account_ids: string[];
  selected_qr_ids: string[];
  selected_static_methods: string[];
}

const STATIC_METHOD_CHOICES = [
  "FPX",
  "DuitNow",
  "Bank Transfer",
  "Cheque",
  "Cash",
  "Credit Card",
];

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
    due_date: addDaysIso(todayIso(), 30),
    currency: "MYR",
    tax_enabled: false,
    tax_rate: null,
    lines: [emptyLine()],
    notes: "",
    terms: "",
    selected_bank_account_ids: [],
    selected_qr_ids: [],
    selected_static_methods: [],
  };
}

function fromInvoice(iwl: InvoiceWithLines): FormState {
  return {
    customer_id: iwl.invoice.customer_id,
    business_profile_id: iwl.invoice.business_profile_id ?? "",
    issue_date: iwl.invoice.issue_date,
    due_date: iwl.invoice.due_date,
    currency: iwl.invoice.currency,
    tax_enabled: iwl.invoice.tax_enabled,
    tax_rate: iwl.invoice.tax_rate,
    lines: iwl.lines.map((l) => ({
      description: l.description,
      quantity: l.quantity,
      unit_price: l.unit_price,
    })),
    notes: iwl.invoice.notes ?? "",
    terms: iwl.invoice.terms ?? "",
    selected_bank_account_ids: iwl.invoice.selected_bank_account_ids,
    selected_qr_ids: iwl.invoice.selected_qr_ids,
    selected_static_methods: iwl.invoice.selected_static_methods,
  };
}

interface TransitionButton {
  label: string;
  action: () => Promise<void>;
  variant: "blue" | "green" | "red" | "orange" | "gray";
  confirm?: string;
}

const variantClass: Record<TransitionButton["variant"], string> = {
  blue: "bg-blue-600 text-white",
  green: "bg-green-600 text-white",
  red: "bg-red-600 text-white",
  orange: "bg-orange-600 text-white",
  gray: "bg-gray-500 text-white",
};

export function InvoiceFormPage() {
  const { id } = useParams<{ id?: string }>();
  const mode: Mode = id ? "edit" : "create";
  const navigate = useNavigate();

  const [form, setForm] = useState<FormState>(emptyForm());
  const [customers, setCustomers] = useState<Customer[]>([]);
  const [profiles, setProfiles] = useState<BusinessProfile[]>([]);
  const [existing, setExisting] = useState<Invoice | null>(null);
  const [assets, setAssets] = useState<ProfileAssetDataUrls | null>(null);
  const [savedPayments, setSavedPayments] = useState<PaymentVoucher[]>([]);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Refresh profile assets only when the selected profile changes — once per
  // change, not per keystroke. The React preview reuses these data URLs.
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

  // Load saved PVs once in edit mode so the preview's payment-history block
  // matches what the PDF will show.
  useEffect(() => {
    if (mode !== "edit" || !id) {
      setSavedPayments([]);
      return;
    }
    paymentVoucherApi
      .listByInvoice(id)
      .then(setSavedPayments)
      .catch(() => setSavedPayments([]));
  }, [id, mode]);

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
          const iwl = await invoiceApi.findById(id);
          setForm(fromInvoice(iwl));
          setExisting(iwl.invoice);
        } else if (profileList.length > 0) {
          const p = profileList[0];
          const today = todayIso();
          setForm((prev) => ({
            ...prev,
            business_profile_id: p.id,
            issue_date: today,
            due_date: addDaysIso(today, p.default_invoice_due_days),
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

  const status: InvoiceStatus = existing?.status ?? "Draft";
  const editable = mode === "create" || status === "Draft";

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
        selected_bank_account_ids: form.selected_bank_account_ids,
        selected_qr_ids: form.selected_qr_ids,
        selected_static_methods: form.selected_static_methods,
        issue_date: form.issue_date,
        due_date: form.due_date,
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
        const saved = await invoiceApi.create(payload);
        savedId = saved.invoice.id;
      } else if (id) {
        const saved = await invoiceApi.update({ id, ...payload });
        savedId = saved.invoice.id;
      } else {
        throw new Error("missing id in edit mode");
      }
      if (andMarkSent) {
        await invoiceApi.markSent(savedId);
      }
      navigate("/invoices");
    } catch (e) {
      setError(formatErr(e));
      setSaving(false);
    }
  }

  async function runTransition(
    label: string,
    fn: () => Promise<Invoice>,
    needConfirm: boolean,
  ) {
    if (!id) return;
    if (needConfirm && !window.confirm(`确认${label}？`)) return;
    setSaving(true);
    setError(null);
    try {
      const updated = await fn();
      setExisting(updated);
      navigate("/invoices");
    } catch (e) {
      setError(formatErr(e));
      setSaving(false);
    }
  }

  function buildTransitions(): TransitionButton[] {
    if (!id) return [];
    const r: TransitionButton[] = [];
    const tx = (
      label: string,
      variant: TransitionButton["variant"],
      fn: () => Promise<Invoice>,
      needConfirm = true,
    ) =>
      r.push({
        label,
        variant,
        action: () => runTransition(label, fn, needConfirm),
      });
    switch (status) {
      case "Sent":
        tx("标记部分付款", "blue", () => invoiceApi.markPartialPaid(id));
        tx("标记已付款", "green", () => invoiceApi.markPaid(id));
        tx("标记逾期", "orange", () => invoiceApi.markOverdue(id));
        tx("作废", "gray", () => invoiceApi.markVoid(id));
        break;
      case "PartialPaid":
        tx("标记已付款", "green", () => invoiceApi.markPaid(id));
        tx("标记逾期", "orange", () => invoiceApi.markOverdue(id));
        tx("作废", "gray", () => invoiceApi.markVoid(id));
        break;
      case "Overdue":
        tx("取消逾期", "blue", () => invoiceApi.cancelOverdue(id));
        tx("标记部分付款", "blue", () => invoiceApi.markPartialPaid(id));
        tx("标记已付款", "green", () => invoiceApi.markPaid(id));
        tx("作废", "gray", () => invoiceApi.markVoid(id));
        break;
      case "Paid":
        tx("作废", "gray", () => invoiceApi.markVoid(id));
        break;
      case "Void":
        tx("恢复 (Restore)", "blue", () => invoiceApi.restoreVoid(id));
        break;
      case "Draft":
      default:
        break;
    }
    return r;
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
    form.lines.every(
      (l) => l.description.trim() !== "" && Number(l.quantity) > 0,
    );

  const transitions = buildTransitions();
  const canPreview = form.customer_id !== "" && form.lines.length > 0;

  return (
    <div className="flex h-full">
      <div className="flex-1 overflow-y-auto p-6">
        <div className="mb-6 flex flex-wrap items-center justify-between gap-3">
        <h1 className="text-2xl font-semibold">
          {mode === "create" ? "New Invoice" : `Invoice ${existing?.number ?? ""}`}
        </h1>
        {existing && (
          <span
            className={`inline-block rounded-md px-3 py-1 text-sm ${invoiceStatusBadgeClass(
              existing.status,
            )}`}
          >
            {invoiceStatusLabel(existing.status)}
          </span>
        )}
      </div>

      {existing?.source_quotation_id && (
        <p className="mb-4 text-sm text-muted-foreground">
          由 Quotation 转换而来 ·{" "}
          <Link
            to={`/quotations/${existing.source_quotation_id}`}
            className="text-primary hover:underline"
          >
            查看原 Quotation
          </Link>
        </p>
      )}

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
            <Field label="到期日 *">
              <input
                type="date"
                className={inputCls}
                value={form.due_date}
                onChange={(e) => updateField("due_date", e.target.value)}
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
                    <th className="w-36 px-3 py-2 text-right font-medium">小计</th>
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
                  onChange={(e) => updateField("tax_enabled", e.target.checked)}
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
            <div
              className={`mt-3 grid gap-3 border-t border-border pt-3 font-mono text-right ${
                existing && existing.paid_amount > 0
                  ? "grid-cols-4"
                  : "grid-cols-3"
              }`}
            >
              <div>
                <div className="text-xs font-sans text-muted-foreground">小计</div>
                <div>{formatMoney(totals.subtotal, form.currency)}</div>
              </div>
              <div>
                <div className="text-xs font-sans text-muted-foreground">税额</div>
                <div>{formatMoney(totals.taxAmount, form.currency)}</div>
              </div>
              <div>
                <div className="text-xs font-sans text-muted-foreground">合计</div>
                <div className="text-base font-semibold">
                  {formatMoney(totals.total, form.currency)}
                </div>
              </div>
              {existing && existing.paid_amount > 0 && (
                <div>
                  <div className="text-xs font-sans text-muted-foreground">
                    已付款
                  </div>
                  <div
                    className={
                      existing.paid_amount > existing.total
                        ? "text-red-600"
                        : existing.paid_amount >= existing.total &&
                            existing.total > 0
                          ? "text-green-600"
                          : existing.paid_amount > 0
                            ? "text-yellow-600"
                            : ""
                    }
                  >
                    <div>
                      {formatMoney(existing.paid_amount, form.currency)}
                    </div>
                    {existing.paid_amount > existing.total && (
                      <div className="text-xs">
                        (超付{" "}
                        {formatMoney(
                          existing.paid_amount - existing.total,
                          form.currency,
                        )}
                        )
                      </div>
                    )}
                  </div>
                </div>
              )}
            </div>
          </div>

          {/* Payment options picker — pulled from the selected business profile. */}
          {(() => {
            const sp = profiles.find(
              (p) => p.id === form.business_profile_id,
            );
            const banks = sp?.bank_accounts ?? [];
            const qrs = sp?.qrs ?? [];
            const toggle = (arr: string[], v: string): string[] =>
              arr.includes(v) ? arr.filter((x) => x !== v) : [...arr, v];
            return (
              <div className="space-y-3 rounded-md border border-border p-4">
                <div className="text-sm font-medium">付款方式（PDF 上显示）</div>
                <p className="text-xs text-muted-foreground">
                  从所选公司资料里勾选要出现在 Invoice PDF 底部的项目。
                </p>

                {banks.length > 0 && (
                  <div>
                    <div className="mb-1 text-xs font-medium text-muted-foreground">
                      银行账户
                    </div>
                    <div className="space-y-1">
                      {banks.map((b) => (
                        <label
                          key={b.id}
                          className="flex items-center gap-2 text-sm"
                        >
                          <input
                            type="checkbox"
                            checked={form.selected_bank_account_ids.includes(
                              b.id,
                            )}
                            onChange={() =>
                              updateField(
                                "selected_bank_account_ids",
                                toggle(form.selected_bank_account_ids, b.id),
                              )
                            }
                          />
                          <span>
                            {b.bank_name} · {b.account_number} (
                            {b.account_holder})
                          </span>
                        </label>
                      ))}
                    </div>
                  </div>
                )}

                {qrs.length > 0 && (
                  <div>
                    <div className="mb-1 text-xs font-medium text-muted-foreground">
                      QR
                    </div>
                    <div className="space-y-1">
                      {qrs.map((q) => (
                        <label
                          key={q.id}
                          className="flex items-center gap-2 text-sm"
                        >
                          <input
                            type="checkbox"
                            checked={form.selected_qr_ids.includes(q.id)}
                            onChange={() =>
                              updateField(
                                "selected_qr_ids",
                                toggle(form.selected_qr_ids, q.id),
                              )
                            }
                          />
                          <span>
                            {q.kind}
                            {q.label ? ` · ${q.label}` : ""}
                          </span>
                        </label>
                      ))}
                    </div>
                  </div>
                )}

                <div>
                  <div className="mb-1 text-xs font-medium text-muted-foreground">
                    其它
                  </div>
                  <div className="flex flex-wrap gap-3">
                    {STATIC_METHOD_CHOICES.map((m) => (
                      <label
                        key={m}
                        className="flex items-center gap-2 text-sm"
                      >
                        <input
                          type="checkbox"
                          checked={form.selected_static_methods.includes(m)}
                          onChange={() =>
                            updateField(
                              "selected_static_methods",
                              toggle(form.selected_static_methods, m),
                            )
                          }
                        />
                        {m}
                      </label>
                    ))}
                  </div>
                </div>

                {banks.length === 0 && qrs.length === 0 && sp && (
                  <p className="text-xs text-muted-foreground">
                    该公司资料没有银行账户或 QR — 可去「公司资料」页添加。
                  </p>
                )}
              </div>
            );
          })()}

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

        {existing && status !== "Draft" && (
          <PaymentVoucherSection
            invoiceId={existing.id}
            currency={existing.currency}
            canEdit={status !== "Void"}
          />
        )}

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
              onClick={() => navigate("/invoices")}
              className="rounded-md border border-input px-4 py-2 text-sm hover:bg-accent"
            >
              {editable ? "取消" : "返回"}
            </button>
          </div>

          {transitions.length > 0 && (
            <div className="flex flex-wrap gap-2">
              {transitions.map((t) => (
                <button
                  key={t.label}
                  type="button"
                  onClick={t.action}
                  disabled={saving}
                  className={`rounded-md ${variantClass[t.variant]} px-4 py-2 text-sm hover:opacity-90 disabled:opacity-50`}
                >
                  {t.label}
                </button>
              ))}
            </div>
          )}
        </div>
      </form>
      </div>
      <div className="w-[45%] min-w-[400px] border-l border-border">
        <PdfPreviewPanel
          docType="Invoice"
          preview={
            <InvoicePreview
              profile={selectedProfile}
              assets={assets}
              customer={selectedCustomer}
              number={existing?.number ?? ""}
              status={existing?.status ?? "Draft"}
              issueDate={form.issue_date}
              dueDate={form.due_date}
              currency={form.currency}
              taxEnabled={form.tax_enabled}
              taxRate={form.tax_rate}
              lines={form.lines}
              notes={form.notes}
              terms={form.terms}
              payments={savedPayments}
              paidAmount={existing?.paid_amount ?? 0}
              selectedBankAccountIds={form.selected_bank_account_ids}
              selectedQrIds={form.selected_qr_ids}
              selectedStaticMethods={form.selected_static_methods}
            />
          }
          canPreview={canPreview}
          notReadyReason="请先选择客户并添加至少一行项目。"
          defaultFilename={
            existing
              ? `${existing.number}-${sanitizeFilenamePart(
                  customerSnapshotName(existing.customer_snapshot),
                )}.pdf`
              : "invoice.pdf"
          }
          onGeneratePdf={
            existing
              ? async (templateId, targetPath) => {
                  await pdfRenderApi.renderInvoice(
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
