import { useEffect, useState, type FormEvent } from "react";
import { Link, useNavigate, useParams, useSearchParams } from "react-router-dom";
import { paymentVoucherApi } from "@/api/payment_voucher";
import { invoiceApi } from "@/api/invoice";
import { customerApi } from "@/api/customer";
import { businessProfileApi } from "@/api/business_profile";
import { pdfRenderApi } from "@/api/pdf";
import { CurrencySelect } from "@/common/components/CurrencySelect";
import { PdfPreviewPanel } from "@/common/components/PdfPreviewPanel";
import { PaymentVoucherPreview } from "@/common/components/preview/PaymentVoucherPreview";
import {
  customerSnapshotName,
  formatErr,
  formatMoney,
  sanitizeFilenamePart,
} from "@/common/utils/format";
import type { BusinessProfile } from "@/types/bindings/BusinessProfile";
import type { Customer } from "@/types/bindings/Customer";
import type { Invoice } from "@/types/bindings/Invoice";
import type { PaymentVoucher } from "@/types/bindings/PaymentVoucher";
import type { ProfileAssetDataUrls } from "@/types/bindings/ProfileAssetDataUrls";

type Mode = "create" | "edit";
type Variant = "linked" | "standalone";

const PAYMENT_METHOD_CHOICES = [
  "FPX",
  "DuitNow",
  "Bank Transfer",
  "Bank QR",
  "TNG QR",
  "Boost QR",
  "GrabPay QR",
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

export function PaymentVoucherFormPage() {
  const { id } = useParams<{ id?: string }>();
  const [searchParams] = useSearchParams();
  const queryInvoiceId = searchParams.get("invoice_id");
  const mode: Mode = id ? "edit" : "create";
  const navigate = useNavigate();

  // Variant is decided after initial load (or immediately from URL in create mode).
  const [variant, setVariant] = useState<Variant>(
    queryInvoiceId ? "linked" : "standalone",
  );

  // Linked-mode state
  const [invoice, setInvoice] = useState<Invoice | null>(null);
  const [otherPvSum, setOtherPvSum] = useState<number>(0);

  // Standalone-mode state
  const [customers, setCustomers] = useState<Customer[]>([]);
  const [profiles, setProfiles] = useState<BusinessProfile[]>([]);
  const [customerId, setCustomerId] = useState("");
  const [businessProfileId, setBusinessProfileId] = useState("");
  const [currency, setCurrency] = useState("MYR");

  // Shared form fields
  const [date, setDate] = useState(todayIso());
  const [amount, setAmount] = useState<number>(0);
  const [paymentMethod, setPaymentMethod] = useState("");
  const [notes, setNotes] = useState("");

  const [existing, setExisting] = useState<PaymentVoucher | null>(null);
  const [assets, setAssets] = useState<ProfileAssetDataUrls | null>(null);
  const [invoiceProfile, setInvoiceProfile] = useState<BusinessProfile | null>(
    null,
  );
  const [linkedCustomer, setLinkedCustomer] = useState<Customer | null>(null);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // PV preview profile id: linked → parent invoice's profile; standalone → user pick.
  const activeProfileId =
    variant === "linked"
      ? invoice?.business_profile_id ?? ""
      : businessProfileId;

  // Load profile assets once per active profile change.
  useEffect(() => {
    if (!activeProfileId) {
      setAssets(null);
      return;
    }
    let cancelled = false;
    businessProfileApi
      .getAssetDataUrls(activeProfileId)
      .then((a) => {
        if (!cancelled) setAssets(a);
      })
      .catch(() => {
        if (!cancelled) setAssets(null);
      });
    return () => {
      cancelled = true;
    };
  }, [activeProfileId]);

  // Linked mode also needs the parent profile object + customer for preview.
  useEffect(() => {
    if (variant !== "linked" || !invoice?.business_profile_id) {
      setInvoiceProfile(null);
      return;
    }
    businessProfileApi
      .findById(invoice.business_profile_id)
      .then(setInvoiceProfile)
      .catch(() => setInvoiceProfile(null));
  }, [variant, invoice?.business_profile_id]);

  useEffect(() => {
    if (variant !== "linked" || !invoice?.customer_id) {
      setLinkedCustomer(null);
      return;
    }
    customerApi
      .findById(invoice.customer_id)
      .then(setLinkedCustomer)
      .catch(() => setLinkedCustomer(null));
  }, [variant, invoice?.customer_id]);

  useEffect(() => {
    async function load() {
      try {
        if (mode === "edit" && id) {
          const pv = await paymentVoucherApi.findById(id);
          setExisting(pv);
          setDate(pv.date);
          setAmount(pv.amount);
          setPaymentMethod(pv.payment_method);
          setNotes(pv.notes ?? "");
          setCurrency(pv.currency);
          setCustomerId(pv.customer_id);
          setBusinessProfileId(pv.business_profile_id ?? "");
          if (pv.invoice_id) {
            setVariant("linked");
            const iwl = await invoiceApi.findById(pv.invoice_id);
            setInvoice(iwl.invoice);
            setOtherPvSum(iwl.invoice.paid_amount - pv.amount);
          } else {
            setVariant("standalone");
            const [list, profileList] = await Promise.all([
              customerApi.list(false),
              businessProfileApi.list(),
            ]);
            setCustomers(list);
            setProfiles(profileList);
          }
        } else if (queryInvoiceId) {
          // Create — linked
          setVariant("linked");
          const iwl = await invoiceApi.findById(queryInvoiceId);
          setInvoice(iwl.invoice);
          setOtherPvSum(iwl.invoice.paid_amount);
          const remaining = Math.max(
            0,
            iwl.invoice.total - iwl.invoice.paid_amount,
          );
          setAmount(remaining);
          setCurrency(iwl.invoice.currency);
          setCustomerId(iwl.invoice.customer_id);
        } else {
          // Create — standalone
          setVariant("standalone");
          const [list, profileList] = await Promise.all([
            customerApi.list(false),
            businessProfileApi.list(),
          ]);
          setCustomers(list);
          setProfiles(profileList);
          if (profileList.length > 0) setBusinessProfileId(profileList[0].id);
        }
      } catch (e) {
        setError(formatErr(e));
      } finally {
        setLoading(false);
      }
    }
    load();
  }, [id, mode, queryInvoiceId]);

  async function onSubmit(e: FormEvent) {
    e.preventDefault();
    setSaving(true);
    setError(null);
    try {
      if (mode === "create") {
        if (variant === "linked" && invoice) {
          await paymentVoucherApi.create({
            invoice_id: invoice.id,
            customer_id: null,
            currency: null,
            business_profile_id: null,
            date,
            amount,
            payment_method: paymentMethod,
            notes: notes.trim() || null,
          });
        } else if (variant === "standalone") {
          await paymentVoucherApi.create({
            invoice_id: null,
            customer_id: customerId,
            currency,
            business_profile_id: businessProfileId || null,
            date,
            amount,
            payment_method: paymentMethod,
            notes: notes.trim() || null,
          });
        }
      } else if (mode === "edit" && id) {
        await paymentVoucherApi.update({
          id,
          date,
          amount,
          payment_method: paymentMethod,
          notes: notes.trim() || null,
        });
      }
      if (variant === "linked" && invoice) {
        navigate(`/invoices/${invoice.id}`);
      } else {
        navigate("/payment-vouchers");
      }
    } catch (e) {
      setError(formatErr(e));
      setSaving(false);
    }
  }

  async function onDelete() {
    if (!id || !existing) return;
    const confirmMsg =
      variant === "linked" && invoice
        ? `确认删除这条 Payment Voucher (${existing.number})？Invoice 已付金额会减去 ${formatMoney(existing.amount, existing.currency)}。`
        : `确认删除这条独立 Payment Voucher (${existing.number})？`;
    if (!window.confirm(confirmMsg)) return;
    setSaving(true);
    setError(null);
    try {
      await paymentVoucherApi.delete(id);
      if (variant === "linked" && invoice) {
        navigate(`/invoices/${invoice.id}`);
      } else {
        navigate("/payment-vouchers");
      }
    } catch (e) {
      setError(formatErr(e));
      setSaving(false);
    }
  }

  const previewProfile =
    variant === "linked"
      ? invoiceProfile
      : profiles.find((p) => p.id === businessProfileId) ?? null;
  const previewCustomer =
    variant === "linked"
      ? linkedCustomer
      : customers.find((c) => c.id === customerId) ?? null;

  if (loading) {
    return <p className="p-8 text-muted-foreground">加载中…</p>;
  }

  // Linked mode required invoice context; if missing, error out.
  if (variant === "linked" && !invoice) {
    return (
      <div className="p-8">
        <p className="text-sm text-red-600">
          {error ?? "无法加载 Invoice 上下文"}
        </p>
        <Link
          to="/invoices"
          className="mt-4 inline-block text-primary hover:underline"
        >
          ← 返回 Invoice 列表
        </Link>
      </div>
    );
  }

  const remaining =
    variant === "linked" && invoice
      ? Math.max(0, invoice.total - otherPvSum - amount)
      : 0;
  const readOnly =
    variant === "linked" &&
    invoice != null &&
    (invoice.status === "Void" || invoice.status === "Draft");

  const baseValid = amount > 0 && paymentMethod.trim() !== "" && date !== "";
  const canSave =
    !readOnly &&
    baseValid &&
    (variant === "linked"
      ? invoice != null
      : customerId !== "" && currency !== "" && businessProfileId !== "");
  const canPreview = canSave;

  const displayCurrency =
    variant === "linked" && invoice ? invoice.currency : currency;

  return (
    <div className="flex h-full">
      <div className="flex-1 overflow-y-auto p-6">
        <h1 className="mb-2 text-2xl font-semibold">
          {mode === "create"
            ? variant === "linked"
              ? "Record Payment"
              : "New Payment Voucher"
            : `Payment Voucher ${existing?.number ?? ""}`}
        </h1>

        {variant === "linked" && invoice && (
          <p className="mb-2 text-sm text-muted-foreground">
            Invoice{" "}
            <Link
              to={`/invoices/${invoice.id}`}
              className="font-mono text-primary hover:underline"
            >
              {invoice.number}
            </Link>{" "}
            · 合计 {formatMoney(invoice.total, invoice.currency)} · 其他已付{" "}
            {formatMoney(otherPvSum, invoice.currency)} · 剩余{" "}
            {formatMoney(remaining, invoice.currency)}
          </p>
        )}
        {variant === "standalone" && (
          <p className="mb-2 text-sm text-muted-foreground">
            独立 Payment Voucher — 不绑定任何 Invoice，不影响 Invoice 的已付金额。
          </p>
        )}

        {readOnly && (
          <p className="mb-6 rounded-md border border-orange-200 bg-orange-50 px-3 py-2 text-sm text-orange-700">
            ⚠️ 父 Invoice 状态为
            <strong className="mx-1">
              {invoice?.status === "Void" ? "已作废" : "草稿"}
            </strong>
            ，Payment Voucher 只读 — 不能保存或删除。
            {invoice?.status === "Void" &&
              " 如需修改，请先在 Invoice 详情页点'恢复 (Restore)'。"}
          </p>
        )}
        {!readOnly && <div className="mb-6" />}

        <form onSubmit={onSubmit} className="space-y-4">
          <fieldset
            disabled={readOnly}
            className="space-y-4 disabled:opacity-70"
          >
            {variant === "standalone" && (
              <>
                <Field label="出单方 (公司资料) *">
                  <select
                    className={inputCls}
                    value={businessProfileId}
                    onChange={(e) => setBusinessProfileId(e.target.value)}
                    disabled={mode === "edit"}
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
                <div className="grid grid-cols-2 gap-4">
                <Field label="客户 *">
                  <select
                    className={inputCls}
                    value={customerId}
                    onChange={(e) => setCustomerId(e.target.value)}
                    disabled={mode === "edit"}
                    required
                  >
                    <option value="">— 请选择 —</option>
                    {customers.map((c) => (
                      <option key={c.id} value={c.id}>
                        {c.name} ({c.type === "Company" ? "公司" : "个人"})
                      </option>
                    ))}
                    {/* In edit mode the original customer may be archived /
                        not in the active list; show its id as a fallback. */}
                    {mode === "edit" &&
                      customerId &&
                      !customers.find((c) => c.id === customerId) && (
                        <option value={customerId}>
                          {existing
                            ? customerSnapshotName(existing.customer_snapshot)
                            : customerId}{" "}
                          (原客户)
                        </option>
                      )}
                  </select>
                </Field>
                <Field label="货币 *">
                  <CurrencySelect
                    className={inputCls}
                    value={currency}
                    onChange={setCurrency}
                    disabled={mode === "edit"}
                  />
                </Field>
                </div>
              </>
            )}

            <div className="grid grid-cols-2 gap-4">
              <Field label="收款日期 *">
                <input
                  type="date"
                  className={inputCls}
                  value={date}
                  onChange={(e) => setDate(e.target.value)}
                  required
                />
              </Field>
              <Field label={`金额 (${displayCurrency}) *`}>
                <input
                  type="number"
                  step="0.01"
                  min="0.01"
                  className={inputCls}
                  value={amount}
                  onChange={(e) => setAmount(Number(e.target.value))}
                  required
                />
              </Field>
            </div>

            <Field label="付款方式 *">
              <select
                className={inputCls}
                value={paymentMethod}
                onChange={(e) => setPaymentMethod(e.target.value)}
                required
              >
                <option value="">— 请选择 —</option>
                {PAYMENT_METHOD_CHOICES.map((m) => (
                  <option key={m} value={m}>
                    {m}
                  </option>
                ))}
                {/* Edit mode: preserve a legacy / free-form value if it isn't
                    in the preset list, so it doesn't silently get lost. */}
                {paymentMethod &&
                  !PAYMENT_METHOD_CHOICES.includes(paymentMethod) && (
                    <option value={paymentMethod}>{paymentMethod}</option>
                  )}
              </select>
            </Field>

            <Field label="备注">
              <textarea
                rows={3}
                className={inputCls + " min-h-16"}
                value={notes}
                onChange={(e) => setNotes(e.target.value)}
              />
            </Field>
          </fieldset>

          {error && <p className="text-sm text-red-600">{error}</p>}

          <div className="flex items-center justify-between pt-2">
            <div className="flex gap-2">
              <button
                type="submit"
                disabled={!canSave || saving}
                className="rounded-md bg-primary px-4 py-2 text-sm text-primary-foreground hover:opacity-90 disabled:opacity-50"
              >
                {saving ? "保存中…" : "保存"}
              </button>
              <button
                type="button"
                onClick={() => navigate(-1)}
                className="rounded-md border border-input px-4 py-2 text-sm hover:bg-accent"
              >
                取消
              </button>
            </div>
            {mode === "edit" && !readOnly && (
              <button
                type="button"
                onClick={onDelete}
                disabled={saving}
                className="rounded-md bg-red-600 px-4 py-2 text-sm text-white hover:opacity-90 disabled:opacity-50"
              >
                删除
              </button>
            )}
          </div>
        </form>
      </div>
      <div className="w-[45%] min-w-[400px] border-l border-border">
        <PdfPreviewPanel
          docType="PaymentVoucher"
          preview={
            <PaymentVoucherPreview
              profile={previewProfile}
              assets={assets}
              customer={previewCustomer}
              number={existing?.number ?? ""}
              date={date}
              amount={Number(amount) || 0}
              currency={displayCurrency}
              paymentMethod={paymentMethod}
              notes={notes}
              invoiceRef={
                variant === "linked" && invoice
                  ? {
                      number: invoice.number,
                      issueDate: invoice.issue_date,
                      total: invoice.total,
                    }
                  : undefined
              }
              balanceAfter={
                variant === "linked" && invoice
                  ? invoice.total - otherPvSum - (Number(amount) || 0)
                  : null
              }
            />
          }
          canPreview={canPreview}
          notReadyReason={
            variant === "standalone"
              ? "请先选择客户、货币、填写日期/金额/付款方式。"
              : "请先填写日期、金额、付款方式。"
          }
          defaultFilename={
            existing
              ? `${existing.number}-${sanitizeFilenamePart(
                  customerSnapshotName(existing.customer_snapshot),
                )}.pdf`
              : "payment-voucher.pdf"
          }
          onGeneratePdf={
            existing
              ? async (templateId, targetPath) => {
                  await pdfRenderApi.renderPaymentVoucher(
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
  "w-full rounded-md border border-input bg-background px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-ring";

function Field({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <label className="block">
      <span className="mb-1 block text-sm text-muted-foreground">{label}</span>
      {children}
    </label>
  );
}
