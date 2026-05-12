import { useEffect, useRef, useState, type FormEvent } from "react";
import { useNavigate, useParams } from "react-router-dom";
import {
  businessProfileApi,
  fileToBase64,
  MAX_LOGO_BYTES,
  MAX_QR_BYTES,
} from "@/api/business_profile";
import { formatErr } from "@/common/utils/format";
import type { BankAccount } from "@/types/bindings/BankAccount";
import type { BusinessProfile } from "@/types/bindings/BusinessProfile";
import type { CreateBusinessProfileInput } from "@/types/bindings/CreateBusinessProfileInput";
import type { EntityType } from "@/types/bindings/EntityType";
import type { ProfileAssetDataUrls } from "@/types/bindings/ProfileAssetDataUrls";
import type { QrKind } from "@/types/bindings/QrKind";

function formatMb(bytes: number): string {
  return `${(bytes / 1_048_576).toFixed(1)} MB`;
}

type Mode = "create" | "edit";

interface FormState {
  entity_type: EntityType;
  name: string;
  address: string;
  email: string;
  phone: string;
  ssm_no: string;
  nric: string;
  sst_no: string;
  bank_accounts: BankAccount[];
  enabled_payment_methods: string[];
  default_tax_rate: number | null;
  default_quotation_valid_days: number;
  default_invoice_due_days: number;
  data_dir: string;
}

function emptyForm(): FormState {
  return {
    entity_type: "Company",
    name: "",
    address: "",
    email: "",
    phone: "",
    ssm_no: "",
    nric: "",
    sst_no: "",
    bank_accounts: [],
    enabled_payment_methods: [],
    default_tax_rate: null,
    default_quotation_valid_days: 30,
    default_invoice_due_days: 30,
    data_dir: "",
  };
}

function fromProfile(p: BusinessProfile): FormState {
  return {
    entity_type: p.entity_type,
    name: p.name,
    address: p.address ?? "",
    email: p.email ?? "",
    phone: p.phone ?? "",
    ssm_no: p.ssm_no ?? "",
    nric: p.nric ?? "",
    sst_no: p.sst_no ?? "",
    bank_accounts: p.bank_accounts,
    enabled_payment_methods: p.enabled_payment_methods,
    default_tax_rate: p.default_tax_rate,
    default_quotation_valid_days: p.default_quotation_valid_days,
    default_invoice_due_days: p.default_invoice_due_days,
    data_dir: p.data_dir,
  };
}

export function BusinessProfileFormPage() {
  const { id } = useParams<{ id?: string }>();
  const mode: Mode = id ? "edit" : "create";
  const navigate = useNavigate();

  const [form, setForm] = useState<FormState>(emptyForm());
  const [current, setCurrent] = useState<BusinessProfile | null>(null);
  const [assets, setAssets] = useState<ProfileAssetDataUrls | null>(null);
  const [loading, setLoading] = useState(mode === "edit");
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  // Section-local errors. Sit next to the upload button so users see them
  // immediately instead of having to scroll to the form's footer.
  const [logoError, setLogoError] = useState<string | null>(null);
  const [qrError, setQrError] = useState<string | null>(null);
  const logoInputRef = useRef<HTMLInputElement>(null);

  // Multi-QR upload form state
  const [newQrKind, setNewQrKind] = useState<QrKind>("Bank");
  const [newQrLabel, setNewQrLabel] = useState("");
  const [newQrFile, setNewQrFile] = useState<File | null>(null);
  const newQrInputRef = useRef<HTMLInputElement>(null);

  async function refreshAssets(profileId: string) {
    try {
      const a = await businessProfileApi.getAssetDataUrls(profileId);
      setAssets(a);
    } catch (e) {
      // Non-fatal: we just won't show thumbnails.
      console.warn("getAssetDataUrls failed:", e);
    }
  }

  useEffect(() => {
    if (mode === "edit" && id) {
      businessProfileApi
        .findById(id)
        .then((p) => {
          setCurrent(p);
          setForm(fromProfile(p));
          return refreshAssets(p.id);
        })
        .catch((e) => setError(formatErr(e)))
        .finally(() => setLoading(false));
    }
  }, [id, mode]);

  function update<K extends keyof FormState>(key: K, value: FormState[K]) {
    setForm((prev) => ({ ...prev, [key]: value }));
  }

  function updateBank(i: number, patch: Partial<BankAccount>) {
    setForm((prev) => ({
      ...prev,
      bank_accounts: prev.bank_accounts.map((b, idx) =>
        idx === i ? { ...b, ...patch } : b,
      ),
    }));
  }

  function addBank() {
    setForm((prev) => ({
      ...prev,
      bank_accounts: [
        ...prev.bank_accounts,
        { id: "", bank_name: "", account_number: "", account_holder: "" },
      ],
    }));
  }

  function removeBank(i: number) {
    setForm((prev) => ({
      ...prev,
      bank_accounts: prev.bank_accounts.filter((_, idx) => idx !== i),
    }));
  }

  function buildPayload(): CreateBusinessProfileInput {
    return {
      entity_type: form.entity_type,
      name: form.name,
      address: form.address.trim() || null,
      email: form.email.trim() || null,
      phone: form.phone.trim() || null,
      ssm_no: form.entity_type === "Company" ? form.ssm_no.trim() || null : null,
      nric:
        form.entity_type === "Individual" ? form.nric.trim() || null : null,
      sst_no: form.sst_no.trim() || null,
      bank_accounts: form.bank_accounts,
      enabled_payment_methods: form.enabled_payment_methods,
      default_tax_rate: form.default_tax_rate,
      default_quotation_valid_days: form.default_quotation_valid_days,
      default_invoice_due_days: form.default_invoice_due_days,
      data_dir: form.data_dir,
    };
  }

  async function onSubmit(e: FormEvent) {
    e.preventDefault();
    setSaving(true);
    setError(null);
    setNotice(null);
    try {
      const payload = buildPayload();
      if (mode === "create") {
        await businessProfileApi.create(payload);
        navigate("/business-profiles");
      } else if (id) {
        const saved = await businessProfileApi.update({ id, ...payload });
        setCurrent(saved);
        setForm(fromProfile(saved));
        setNotice(`已保存于 ${saved.updated_at}`);
      }
    } catch (err) {
      setError(formatErr(err));
    } finally {
      setSaving(false);
    }
  }

  async function onUploadLogo(e: React.ChangeEvent<HTMLInputElement>) {
    if (!current) return;
    const file = e.target.files?.[0];
    if (!file) return;
    setLogoError(null);
    if (file.size > MAX_LOGO_BYTES) {
      setLogoError(
        `Logo 文件过大：${formatMb(file.size)}（上限 ${formatMb(MAX_LOGO_BYTES)}）。请压缩后再上传。`,
      );
      if (logoInputRef.current) logoInputRef.current.value = "";
      return;
    }
    setSaving(true);
    setNotice(null);
    try {
      const ext = file.name.split(".").pop()?.toLowerCase() ?? "png";
      const b64 = await fileToBase64(file);
      const saved = await businessProfileApi.setLogo(current.id, b64, ext);
      setCurrent(saved);
      await refreshAssets(saved.id);
      setNotice("Logo 已上传");
    } catch (err) {
      setLogoError(formatErr(err));
    } finally {
      setSaving(false);
      if (logoInputRef.current) logoInputRef.current.value = "";
    }
  }

  async function onClearLogo() {
    if (!current) return;
    if (!window.confirm("确认移除当前 Logo？")) return;
    setSaving(true);
    try {
      const saved = await businessProfileApi.clearLogo(current.id);
      setCurrent(saved);
      await refreshAssets(saved.id);
      setNotice("Logo 已移除");
    } catch (err) {
      setError(formatErr(err));
    } finally {
      setSaving(false);
    }
  }

  async function onClearQr() {
    if (!current) return;
    if (!window.confirm("确认移除当前 QR？")) return;
    setSaving(true);
    try {
      const saved = await businessProfileApi.clearQr(current.id);
      setCurrent(saved);
      await refreshAssets(saved.id);
      setNotice("QR 已移除");
    } catch (err) {
      setError(formatErr(err));
    } finally {
      setSaving(false);
    }
  }

  async function onAddMultiQr() {
    if (!current || !newQrFile) return;
    setQrError(null);
    if (newQrFile.size > MAX_QR_BYTES) {
      setQrError(
        `QR 文件过大：${formatMb(newQrFile.size)}（上限 ${formatMb(MAX_QR_BYTES)}）。请压缩后再上传。`,
      );
      return;
    }
    setSaving(true);
    setNotice(null);
    try {
      const ext = newQrFile.name.split(".").pop()?.toLowerCase() ?? "png";
      const b64 = await fileToBase64(newQrFile);
      const saved = await businessProfileApi.addQr(
        current.id,
        newQrKind,
        newQrLabel.trim(),
        b64,
        ext,
      );
      setCurrent(saved);
      await refreshAssets(saved.id);
      setNewQrLabel("");
      setNewQrFile(null);
      if (newQrInputRef.current) newQrInputRef.current.value = "";
      setNotice("QR 已添加");
    } catch (err) {
      setQrError(formatErr(err));
    } finally {
      setSaving(false);
    }
  }

  async function onRemoveQr(qrId: string) {
    if (!current) return;
    if (!window.confirm("确认删除这个 QR？")) return;
    setSaving(true);
    setError(null);
    try {
      const saved = await businessProfileApi.removeQr(current.id, qrId);
      setCurrent(saved);
      await refreshAssets(saved.id);
      setNotice("QR 已删除");
    } catch (err) {
      setError(formatErr(err));
    } finally {
      setSaving(false);
    }
  }

  if (loading) return <p className="p-8 text-muted-foreground">加载中…</p>;

  const isCompany = form.entity_type === "Company";
  const nameLabel = isCompany ? "公司名 *" : "姓名 *";

  return (
    <div className="mx-auto max-w-3xl p-8">
      <h1 className="mb-6 text-2xl font-semibold">
        {mode === "create" ? "新增公司资料" : `编辑 ${current?.name ?? ""}`}
      </h1>
      <form onSubmit={onSubmit} className="space-y-5">
        <Field label="类型 *">
          <select
            className={inputCls}
            value={form.entity_type}
            onChange={(e) => {
              const v = e.target.value as EntityType;
              update("entity_type", v);
              if (v === "Company") update("nric", "");
              else update("ssm_no", "");
            }}
          >
            <option value="Company">公司 (Company)</option>
            <option value="Individual">个人 (Individual)</option>
          </select>
        </Field>

        <Field label={nameLabel}>
          <input
            className={inputCls}
            value={form.name}
            onChange={(e) => update("name", e.target.value)}
            required
          />
        </Field>

        <Field label="地址">
          <textarea
            className={inputCls + " min-h-20"}
            value={form.address}
            onChange={(e) => update("address", e.target.value)}
          />
        </Field>

        <div className="grid grid-cols-2 gap-4">
          <Field label="Email">
            <input
              className={inputCls}
              type="email"
              value={form.email}
              onChange={(e) => update("email", e.target.value)}
            />
          </Field>
          <Field label="电话">
            <input
              className={inputCls}
              value={form.phone}
              onChange={(e) => update("phone", e.target.value)}
            />
          </Field>
        </div>

        <div className="grid grid-cols-2 gap-4">
          {isCompany ? (
            <Field label="SSM 号 *">
              <input
                className={inputCls}
                value={form.ssm_no}
                onChange={(e) => update("ssm_no", e.target.value)}
                required
              />
            </Field>
          ) : (
            <Field label="NRIC *">
              <input
                className={inputCls}
                value={form.nric}
                onChange={(e) => update("nric", e.target.value)}
                placeholder="例如 900101-01-1234"
                required
              />
            </Field>
          )}
          <Field label="SST 号">
            <input
              className={inputCls}
              value={form.sst_no}
              onChange={(e) => update("sst_no", e.target.value)}
            />
          </Field>
        </div>

        {/* Logo upload — only in edit mode (need an id to attach), Company only */}
        {isCompany && mode === "edit" && current && (
          <div className="rounded-md border border-border p-4">
            <div className="mb-2 text-sm font-medium">Logo</div>
            <div className="flex items-start gap-4">
              <div className="flex h-24 w-32 items-center justify-center overflow-hidden rounded-md border border-border bg-muted/40 text-xs text-muted-foreground">
                {assets?.logo_data_url ? (
                  <img
                    src={assets.logo_data_url}
                    alt="logo"
                    className="max-h-full max-w-full object-contain"
                  />
                ) : current.logo_path ? (
                  <span className="px-2 text-center">载入中…</span>
                ) : (
                  <span>未上传</span>
                )}
              </div>
              <div className="flex-1 space-y-2">
                <input
                  ref={logoInputRef}
                  type="file"
                  accept="image/*"
                  onChange={onUploadLogo}
                  className="block w-full text-sm file:mr-3 file:rounded-md file:border-0 file:bg-primary file:px-3 file:py-1.5 file:text-sm file:text-primary-foreground hover:file:opacity-90"
                />
                <p className="text-xs text-muted-foreground">
                  推荐 PNG/JPG，长边 ≤ 400px，文件 ≤ {formatMb(MAX_LOGO_BYTES)}。会嵌入到 PDF 头部。
                </p>
                {logoError && (
                  <p className="rounded-md border border-red-300 bg-red-50 px-2 py-1 text-xs text-red-700">
                    {logoError}
                  </p>
                )}
                {current.logo_path && (
                  <button
                    type="button"
                    onClick={onClearLogo}
                    className="text-xs text-red-600 hover:underline"
                  >
                    移除 Logo
                  </button>
                )}
              </div>
            </div>
          </div>
        )}

        {/* Multi-QR — only in edit mode */}
        {mode === "edit" && current && (
          <div className="rounded-md border border-border p-4">
            <div className="mb-3 text-sm font-medium">收款 QR (多张)</div>

            {/* List of saved QRs */}
            {current.qrs.length === 0 ? (
              <p className="mb-3 text-xs text-muted-foreground">
                还没有 QR。下面表单可上传。
              </p>
            ) : (
              <div className="mb-4 grid grid-cols-2 gap-3 md:grid-cols-3">
                {current.qrs.map((q) => {
                  const dataUrl = assets?.qrs.find((x) => x.id === q.id)
                    ?.data_url;
                  return (
                    <div
                      key={q.id}
                      className="flex items-center gap-3 rounded-md border border-border p-2"
                    >
                      <div className="flex h-16 w-16 items-center justify-center overflow-hidden rounded-md border border-border bg-muted/40 text-[10px] text-muted-foreground">
                        {dataUrl ? (
                          <img
                            src={dataUrl}
                            alt={`${q.kind} QR`}
                            className="max-h-full max-w-full object-contain"
                          />
                        ) : (
                          <span>载入中…</span>
                        )}
                      </div>
                      <div className="flex-1">
                        <div className="text-sm font-medium">{q.kind}</div>
                        {q.label && (
                          <div className="text-xs text-muted-foreground">
                            {q.label}
                          </div>
                        )}
                        <button
                          type="button"
                          onClick={() => onRemoveQr(q.id)}
                          className="mt-1 text-xs text-red-600 hover:underline"
                        >
                          删除
                        </button>
                      </div>
                    </div>
                  );
                })}
              </div>
            )}

            {/* Add new QR */}
            <div className="grid grid-cols-1 gap-2 rounded-md border border-dashed border-border p-3 md:grid-cols-4">
              <select
                className={inputCls}
                value={newQrKind}
                onChange={(e) => setNewQrKind(e.target.value as QrKind)}
              >
                <option value="Bank">Bank QR</option>
                <option value="Tng">TNG QR</option>
                <option value="Boost">Boost QR</option>
                <option value="GrabPay">GrabPay QR</option>
                <option value="Other">其它 QR</option>
              </select>
              <input
                className={inputCls}
                placeholder="备注 (可选)"
                value={newQrLabel}
                onChange={(e) => setNewQrLabel(e.target.value)}
              />
              <input
                ref={newQrInputRef}
                type="file"
                accept="image/*"
                onChange={(e) => setNewQrFile(e.target.files?.[0] ?? null)}
                className="block w-full text-sm file:mr-3 file:rounded-md file:border-0 file:bg-primary file:px-3 file:py-1.5 file:text-sm file:text-primary-foreground hover:file:opacity-90"
              />
              <button
                type="button"
                onClick={onAddMultiQr}
                disabled={!newQrFile || saving}
                className="rounded-md bg-primary px-3 py-2 text-sm text-primary-foreground hover:opacity-90 disabled:opacity-50"
              >
                添加 QR
              </button>
            </div>
            <p className="mt-2 text-xs text-muted-foreground">
              Invoice 表单上可逐条勾选要显示在 PDF 上的 QR。文件 ≤ {formatMb(MAX_QR_BYTES)}。
            </p>
            {qrError && (
              <p className="mt-2 rounded-md border border-red-300 bg-red-50 px-2 py-1 text-xs text-red-700">
                {qrError}
              </p>
            )}
          </div>
        )}

        {/* Legacy single-QR (Slice A): hidden but kept so old data clears via API. */}
        {mode === "edit" && current && current.qr_path && (
          <p className="rounded-md border border-amber-200 bg-amber-50 px-3 py-2 text-xs text-amber-700">
            旧版单 QR 数据存在 —{" "}
            <button
              type="button"
              onClick={onClearQr}
              className="text-red-600 hover:underline"
            >
              清除
            </button>
          </p>
        )}

        {/* Bank accounts */}
        <div className="rounded-md border border-border p-4">
          <div className="mb-3 flex items-center justify-between">
            <div className="text-sm font-medium">银行账户</div>
            <button
              type="button"
              onClick={addBank}
              className="text-xs text-primary hover:underline"
            >
              + 添加账户
            </button>
          </div>
          {form.bank_accounts.length === 0 ? (
            <p className="text-xs text-muted-foreground">
              没有账户。Invoice PDF 底部会跳过 Bank 区。
            </p>
          ) : (
            <div className="space-y-3">
              {form.bank_accounts.map((b, i) => (
                <div
                  key={i}
                  className="grid grid-cols-1 gap-2 rounded-md border border-border p-3 md:grid-cols-4"
                >
                  <input
                    className={inputCls}
                    placeholder="银行名 (如 Maybank)"
                    value={b.bank_name}
                    onChange={(e) =>
                      updateBank(i, { bank_name: e.target.value })
                    }
                  />
                  <input
                    className={inputCls}
                    placeholder="账号"
                    value={b.account_number}
                    onChange={(e) =>
                      updateBank(i, { account_number: e.target.value })
                    }
                  />
                  <input
                    className={inputCls}
                    placeholder="账户名"
                    value={b.account_holder}
                    onChange={(e) =>
                      updateBank(i, { account_holder: e.target.value })
                    }
                  />
                  <button
                    type="button"
                    onClick={() => removeBank(i)}
                    className="text-xs text-red-600 hover:underline"
                  >
                    删除
                  </button>
                </div>
              ))}
            </div>
          )}
        </div>

        <Field label="默认税率（0.0 ~ 1.0，留空表示无税）">
          <input
            className={inputCls}
            type="number"
            step="0.01"
            min="0"
            max="1"
            value={form.default_tax_rate ?? ""}
            onChange={(e) => {
              const v = e.target.value;
              update("default_tax_rate", v === "" ? null : Number(v));
            }}
          />
        </Field>

        <div className="grid grid-cols-2 gap-4">
          <Field label="报价默认有效天数">
            <input
              className={inputCls}
              type="number"
              min="0"
              value={form.default_quotation_valid_days}
              onChange={(e) =>
                update(
                  "default_quotation_valid_days",
                  Number(e.target.value),
                )
              }
            />
          </Field>
          <Field label="发票默认到期天数">
            <input
              className={inputCls}
              type="number"
              min="0"
              value={form.default_invoice_due_days}
              onChange={(e) =>
                update("default_invoice_due_days", Number(e.target.value))
              }
            />
          </Field>
        </div>

        <Field label="数据/PDF 输出目录">
          <input
            className={inputCls}
            value={form.data_dir}
            onChange={(e) => update("data_dir", e.target.value)}
            placeholder="例如 C:\\Users\\you\\InvoiceData"
          />
        </Field>

        {error && <p className="text-sm text-red-600">{error}</p>}
        {notice && <p className="text-sm text-green-600">{notice}</p>}

        <div className="flex items-center gap-3 pt-2">
          <button
            type="submit"
            disabled={saving}
            className="rounded-md bg-primary px-4 py-2 text-sm text-primary-foreground hover:opacity-90 disabled:opacity-50"
          >
            {saving ? "保存中…" : "保存"}
          </button>
          <button
            type="button"
            onClick={() => navigate("/business-profiles")}
            className="rounded-md border border-input px-4 py-2 text-sm hover:bg-accent"
          >
            返回列表
          </button>
        </div>
      </form>
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
