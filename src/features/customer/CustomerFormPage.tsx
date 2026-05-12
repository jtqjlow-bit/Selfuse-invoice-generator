import { useEffect, useState, type FormEvent } from "react";
import { useNavigate, useParams } from "react-router-dom";
import { customerApi } from "@/api/customer";
import type { Customer } from "@/types/bindings/Customer";
import type { CustomerType } from "@/types/bindings/CustomerType";

type Mode = "create" | "edit";

interface FormState {
  type: CustomerType;
  name: string;
  contact_person: string;
  email: string;
  phone: string;
  address: string;
  ssm_no: string;
  nric: string;
  tax_no: string;
  notes: string;
}

function emptyForm(): FormState {
  return {
    type: "Company",
    name: "",
    contact_person: "",
    email: "",
    phone: "",
    address: "",
    ssm_no: "",
    nric: "",
    tax_no: "",
    notes: "",
  };
}

function fromCustomer(c: Customer): FormState {
  return {
    type: c.type,
    name: c.name,
    contact_person: c.contact_person ?? "",
    email: c.email ?? "",
    phone: c.phone ?? "",
    address: c.address ?? "",
    ssm_no: c.ssm_no ?? "",
    nric: c.nric ?? "",
    tax_no: c.tax_no ?? "",
    notes: c.notes ?? "",
  };
}

function toOpt(s: string): string | null {
  const t = s.trim();
  return t === "" ? null : t;
}

export function CustomerFormPage() {
  const { id } = useParams<{ id?: string }>();
  const mode: Mode = id ? "edit" : "create";
  const navigate = useNavigate();

  const [form, setForm] = useState<FormState>(emptyForm());
  const [archived, setArchived] = useState(false);
  const [loading, setLoading] = useState(mode === "edit");
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (mode === "edit" && id) {
      customerApi
        .findById(id)
        .then((c) => {
          setForm(fromCustomer(c));
          setArchived(c.archived);
        })
        .catch((e) => setError(formatErr(e)))
        .finally(() => setLoading(false));
    }
  }, [id, mode]);

  function update<K extends keyof FormState>(k: K, v: FormState[K]) {
    setForm((p) => ({ ...p, [k]: v }));
  }

  async function onSubmit(e: FormEvent) {
    e.preventDefault();
    setSaving(true);
    setError(null);
    try {
      if (mode === "create") {
        await customerApi.create({
          type: form.type,
          name: form.name,
          contact_person: toOpt(form.contact_person),
          email: toOpt(form.email),
          phone: toOpt(form.phone),
          address: toOpt(form.address),
          ssm_no: toOpt(form.ssm_no),
          nric: toOpt(form.nric),
          tax_no: toOpt(form.tax_no),
          notes: toOpt(form.notes),
        });
      } else if (id) {
        await customerApi.update({
          id,
          type: form.type,
          name: form.name,
          contact_person: toOpt(form.contact_person),
          email: toOpt(form.email),
          phone: toOpt(form.phone),
          address: toOpt(form.address),
          ssm_no: toOpt(form.ssm_no),
          nric: toOpt(form.nric),
          tax_no: toOpt(form.tax_no),
          notes: toOpt(form.notes),
        });
      }
      navigate("/customers");
    } catch (e) {
      setError(formatErr(e));
      setSaving(false);
    }
  }

  async function onArchive() {
    if (!id) return;
    try {
      if (archived) {
        await customerApi.unarchive(id);
      } else {
        await customerApi.archive(id);
      }
      navigate("/customers");
    } catch (e) {
      setError(formatErr(e));
    }
  }

  if (loading) {
    return <p className="p-8 text-muted-foreground">加载中…</p>;
  }

  const isCompany = form.type === "Company";

  return (
    <div className="mx-auto max-w-2xl p-8">
      <h1 className="mb-6 text-2xl font-semibold">
        {mode === "create" ? "新增客户" : "编辑客户"}
      </h1>

      <form onSubmit={onSubmit} className="space-y-4">
        <Field label="类型 *">
          <select
            className={inputCls}
            value={form.type}
            onChange={(e) => update("type", e.target.value as CustomerType)}
          >
            <option value="Company">公司 (Company)</option>
            <option value="Individual">个人 (Individual)</option>
          </select>
        </Field>

        <Field label={isCompany ? "公司名称 *" : "姓名 *"}>
          <input
            className={inputCls}
            value={form.name}
            onChange={(e) => update("name", e.target.value)}
            required
          />
        </Field>

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
              required
            />
          </Field>
        )}

        <div className="grid grid-cols-2 gap-4">
          <Field label="联系人">
            <input
              className={inputCls}
              value={form.contact_person}
              onChange={(e) => update("contact_person", e.target.value)}
            />
          </Field>
          <Field label="税号 (SST 等)">
            <input
              className={inputCls}
              value={form.tax_no}
              onChange={(e) => update("tax_no", e.target.value)}
            />
          </Field>
        </div>

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

        <Field label="地址">
          <textarea
            className={inputCls + " min-h-20"}
            value={form.address}
            onChange={(e) => update("address", e.target.value)}
          />
        </Field>

        <Field label="备注">
          <textarea
            className={inputCls + " min-h-16"}
            value={form.notes}
            onChange={(e) => update("notes", e.target.value)}
          />
        </Field>

        {error && <p className="text-sm text-red-600">{error}</p>}

        <div className="flex items-center justify-between pt-2">
          <div className="flex gap-2">
            <button
              type="submit"
              disabled={saving}
              className="rounded-md bg-primary px-4 py-2 text-sm text-primary-foreground hover:opacity-90 disabled:opacity-50"
            >
              {saving ? "保存中…" : "保存"}
            </button>
            <button
              type="button"
              onClick={() => navigate("/customers")}
              className="rounded-md border border-input px-4 py-2 text-sm hover:bg-accent"
            >
              取消
            </button>
          </div>
          {mode === "edit" && (
            <button
              type="button"
              onClick={onArchive}
              className="text-sm text-muted-foreground hover:underline"
            >
              {archived ? "取消归档" : "归档此客户"}
            </button>
          )}
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

function formatErr(e: unknown): string {
  if (e && typeof e === "object" && "message" in e) {
    return String((e as { message: string }).message);
  }
  return String(e);
}
