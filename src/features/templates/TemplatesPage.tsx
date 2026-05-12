import { useEffect, useMemo, useRef, useState } from "react";
import { pdfTemplateApi } from "@/api/pdf";
import { formatErr } from "@/common/utils/format";
import type { PdfDocType } from "@/types/bindings/PdfDocType";
import type { PdfTemplate } from "@/types/bindings/PdfTemplate";

const DOC_TYPES: { value: PdfDocType; label: string }[] = [
  { value: "Quotation", label: "Quotation" },
  { value: "Invoice", label: "Invoice" },
  { value: "PaymentVoucher", label: "Payment Voucher" },
];

/** Card-sized thumbnail. A4-aspect iframe scaled to fit; iframe mounts once
 *  and never reloads. We deliberately render at modest scale (0.3×) — large
 *  enough to recognise the layout, small enough to stay snappy. */
function TemplateThumbnail({ html }: { html: string | null }) {
  return (
    <div className="relative h-[340px] w-full overflow-hidden border-b border-border bg-white">
      {html ? (
        <iframe
          title="thumbnail"
          srcDoc={html}
          sandbox=""
          style={{
            width: "794px",
            height: "1123px",
            transform: "scale(0.3)",
            transformOrigin: "0 0",
            border: "none",
            pointerEvents: "none",
          }}
        />
      ) : (
        <div className="flex h-full w-full items-center justify-center text-xs text-muted-foreground">
          加载中…
        </div>
      )}
    </div>
  );
}

export function TemplatesPage() {
  const [templates, setTemplates] = useState<PdfTemplate[]>([]);
  const [thumbHtmls, setThumbHtmls] = useState<Record<string, string>>({});
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);

  // Upload form state
  const [uploadDocType, setUploadDocType] = useState<PdfDocType>("Invoice");
  const [uploadName, setUploadName] = useState("");
  const [uploadFile, setUploadFile] = useState<File | null>(null);
  const [uploading, setUploading] = useState(false);
  const fileInputRef = useRef<HTMLInputElement>(null);

  // View-source modal
  const [viewing, setViewing] = useState<{ name: string; html: string } | null>(
    null,
  );

  function reload() {
    setLoading(true);
    setError(null);
    pdfTemplateApi
      .list()
      .then(setTemplates)
      .catch((e) => setError(formatErr(e)))
      .finally(() => setLoading(false));
  }

  useEffect(reload, []);

  // After templates load, fetch a sample-rendered HTML per template for its
  // thumbnail. Fires only once per template id; results cached in state.
  useEffect(() => {
    let cancelled = false;
    for (const t of templates) {
      if (thumbHtmls[t.id]) continue;
      pdfTemplateApi
        .renderSample(t.id)
        .then((html) => {
          if (!cancelled) {
            setThumbHtmls((prev) => ({ ...prev, [t.id]: html }));
          }
        })
        .catch(() => {
          // Non-fatal: thumbnail just stays blank.
        });
    }
    return () => {
      cancelled = true;
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [templates]);

  const grouped = useMemo(() => {
    const map = new Map<PdfDocType, PdfTemplate[]>();
    for (const dt of DOC_TYPES) map.set(dt.value, []);
    for (const t of templates) {
      map.get(t.doc_type)?.push(t);
    }
    return map;
  }, [templates]);

  async function onUpload(e: React.FormEvent) {
    e.preventDefault();
    if (!uploadFile || !uploadName.trim()) return;
    setUploading(true);
    setError(null);
    setNotice(null);
    try {
      const html = await uploadFile.text();
      await pdfTemplateApi.uploadCustom({
        doc_type: uploadDocType,
        name: uploadName.trim(),
        html_content: html,
      });
      setNotice(`已上传 "${uploadName.trim()}"`);
      setUploadName("");
      setUploadFile(null);
      if (fileInputRef.current) fileInputRef.current.value = "";
      reload();
    } catch (e) {
      setError(formatErr(e));
    } finally {
      setUploading(false);
    }
  }

  async function onDelete(t: PdfTemplate) {
    if (!window.confirm(`确认删除自定义模板 "${t.name}"？此操作不可撤销。`))
      return;
    setError(null);
    setNotice(null);
    try {
      await pdfTemplateApi.deleteCustom(t.id);
      setNotice(`已删除 "${t.name}"`);
      reload();
    } catch (e) {
      setError(formatErr(e));
    }
  }

  async function onView(t: PdfTemplate) {
    setError(null);
    try {
      const html = await pdfTemplateApi.getRenderable(t.id);
      setViewing({ name: t.name, html });
    } catch (e) {
      setError(formatErr(e));
    }
  }

  return (
    <div className="mx-auto max-w-5xl p-8">
      <div className="mb-6 flex items-end justify-between">
        <div>
          <h1 className="text-2xl font-semibold">PDF 模板</h1>
          <p className="mt-1 text-sm text-muted-foreground">
            预设模板内置 3 套（不可删除）。也可上传自己的 HTML 模板（Tera
            语法，占位符见预设模板源码）。
          </p>
        </div>
      </div>

      {/* Upload form */}
      <form
        onSubmit={onUpload}
        className="mb-8 rounded-md border border-border bg-card p-4"
      >
        <h2 className="mb-3 text-sm font-medium">上传自定义模板</h2>
        <div className="grid grid-cols-1 gap-3 md:grid-cols-3">
          <label className="block">
            <span className="mb-1 block text-xs text-muted-foreground">
              文档类型 *
            </span>
            <select
              className={inputCls}
              value={uploadDocType}
              onChange={(e) => setUploadDocType(e.target.value as PdfDocType)}
            >
              {DOC_TYPES.map((dt) => (
                <option key={dt.value} value={dt.value}>
                  {dt.label}
                </option>
              ))}
            </select>
          </label>
          <label className="block">
            <span className="mb-1 block text-xs text-muted-foreground">
              模板名称 *
            </span>
            <input
              className={inputCls}
              value={uploadName}
              onChange={(e) => setUploadName(e.target.value)}
              placeholder="如：彩色发票模板"
              required
            />
          </label>
          <label className="block">
            <span className="mb-1 block text-xs text-muted-foreground">
              HTML 文件 *
            </span>
            <input
              ref={fileInputRef}
              type="file"
              accept=".html,text/html"
              className="block w-full text-sm file:mr-3 file:rounded-md file:border-0 file:bg-primary file:px-3 file:py-1.5 file:text-sm file:text-primary-foreground hover:file:opacity-90"
              onChange={(e) => setUploadFile(e.target.files?.[0] ?? null)}
              required
            />
          </label>
        </div>
        <div className="mt-3 flex items-center gap-3">
          <button
            type="submit"
            disabled={uploading || !uploadFile || !uploadName.trim()}
            className="rounded-md bg-primary px-4 py-2 text-sm text-primary-foreground hover:opacity-90 disabled:opacity-50"
          >
            {uploading ? "上传中…" : "上传"}
          </button>
          {notice && <span className="text-xs text-green-600">{notice}</span>}
          {error && <span className="text-xs text-red-600">{error}</span>}
        </div>
      </form>

      {loading ? (
        <p className="text-sm text-muted-foreground">加载中…</p>
      ) : (
        <div className="space-y-6">
          {DOC_TYPES.map((dt) => {
            const list = grouped.get(dt.value) ?? [];
            return (
              <section key={dt.value}>
                <h2 className="mb-2 text-sm font-medium">
                  {dt.label}{" "}
                  <span className="text-xs text-muted-foreground">
                    ({list.length})
                  </span>
                </h2>
                {list.length === 0 ? (
                  <p className="text-sm text-muted-foreground">无模板</p>
                ) : (
                  <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
                    {list.map((t) => (
                      <div
                        key={t.id}
                        className="overflow-hidden rounded-md border border-border bg-card shadow-sm"
                      >
                        <TemplateThumbnail html={thumbHtmls[t.id] ?? null} />
                        <div className="p-3">
                          <div className="flex items-start justify-between gap-2">
                            <div className="truncate text-sm font-medium">
                              {t.name}
                            </div>
                            {t.type === "Preset" ? (
                              <span className="shrink-0 rounded-md bg-blue-100 px-2 py-0.5 text-xs text-blue-700">
                                预设
                              </span>
                            ) : (
                              <span className="shrink-0 rounded-md bg-amber-100 px-2 py-0.5 text-xs text-amber-700">
                                自定义
                              </span>
                            )}
                          </div>
                          <div className="mt-1 text-xs text-muted-foreground">
                            更新于 {t.updated_at.slice(0, 10)}
                          </div>
                          <div className="mt-3 flex gap-3 text-xs">
                            <button
                              onClick={() => onView(t)}
                              className="text-primary hover:underline"
                            >
                              查看 HTML
                            </button>
                            {t.type === "Custom" && (
                              <button
                                onClick={() => onDelete(t)}
                                className="text-red-600 hover:underline"
                              >
                                删除
                              </button>
                            )}
                          </div>
                        </div>
                      </div>
                    ))}
                  </div>
                )}
              </section>
            );
          })}
        </div>
      )}

      {viewing && (
        <div
          className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-6"
          onClick={() => setViewing(null)}
        >
          <div
            className="flex h-full max-h-[80vh] w-full max-w-4xl flex-col rounded-md bg-card shadow-xl"
            onClick={(e) => e.stopPropagation()}
          >
            <div className="flex items-center justify-between border-b border-border p-3">
              <h3 className="text-sm font-medium">{viewing.name} — HTML 源码</h3>
              <div className="flex items-center gap-2">
                <button
                  onClick={() => {
                    void navigator.clipboard.writeText(viewing.html);
                    setNotice("已复制到剪贴板");
                  }}
                  className="rounded-md border border-input px-3 py-1 text-xs hover:bg-accent"
                >
                  复制
                </button>
                <button
                  onClick={() => setViewing(null)}
                  className="rounded-md border border-input px-3 py-1 text-xs hover:bg-accent"
                >
                  关闭
                </button>
              </div>
            </div>
            <pre className="flex-1 overflow-auto bg-muted/40 p-3 text-xs">
              <code>{viewing.html}</code>
            </pre>
          </div>
        </div>
      )}
    </div>
  );
}

const inputCls =
  "w-full rounded-md border border-input bg-background px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-ring";
