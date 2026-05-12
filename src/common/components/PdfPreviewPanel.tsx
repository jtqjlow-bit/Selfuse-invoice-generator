import { useEffect, useState, type ReactNode } from "react";
import { save } from "@tauri-apps/plugin-dialog";
import { pdfTemplateApi } from "@/api/pdf";
import { formatErr } from "@/common/utils/format";
import type { PdfDocType } from "@/types/bindings/PdfDocType";
import type { PdfTemplate } from "@/types/bindings/PdfTemplate";

interface Props {
  docType: PdfDocType;
  /** Live preview rendered as React. Parent constructs it from form state; we
   *  just mount it inside a scrollable container styled like A4 paper.
   *  Rendering happens in-process (no IPC / no Tera / no iframe reload), so
   *  it can re-render per keystroke without lag. */
  preview: ReactNode;
  /** Called when the user clicks 生成 PDF and picks a save path. Panel handles
   *  the save-file dialog itself; this callback only needs to perform the
   *  render. Null = doc isn't saved yet (button hidden). */
  onGeneratePdf?: (templateId: string, targetPath: string) => Promise<void>;
  /** Suggested filename used by the save dialog (without path, with .pdf). */
  defaultFilename?: string;
  /** If false, the panel just shows "保存后才能预览" (e.g., customer not chosen yet). */
  canPreview: boolean;
  notReadyReason?: string;
}

export function PdfPreviewPanel({
  docType,
  preview,
  onGeneratePdf,
  defaultFilename,
  canPreview,
  notReadyReason,
}: Props) {
  const [templates, setTemplates] = useState<PdfTemplate[]>([]);
  const [templateId, setTemplateId] = useState<string>("");
  const [generating, setGenerating] = useState(false);
  const [generateError, setGenerateError] = useState<string | null>(null);
  const [generateNotice, setGenerateNotice] = useState<string | null>(null);
  const [listError, setListError] = useState<string | null>(null);

  // Load templates once per docType change. Only needed for the PDF export
  // path — live preview is React-only and doesn't consult the template list.
  useEffect(() => {
    pdfTemplateApi
      .listByDocType(docType)
      .then((list) => {
        setTemplates(list);
        if (list.length > 0 && templateId === "") {
          setTemplateId(list[0].id);
        }
      })
      .catch((e) => setListError(formatErr(e)));
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [docType]);

  async function onGenerate() {
    if (!onGeneratePdf || !templateId) return;
    setGenerateError(null);
    setGenerateNotice(null);
    let targetPath: string | null;
    try {
      targetPath = await save({
        title: "保存 PDF",
        defaultPath: defaultFilename ?? "document.pdf",
        filters: [{ name: "PDF", extensions: ["pdf"] }],
      });
    } catch (e) {
      setGenerateError(formatErr(e));
      return;
    }
    if (!targetPath) return; // user cancelled
    setGenerating(true);
    try {
      await onGeneratePdf(templateId, targetPath);
      setGenerateNotice(`已保存到 ${targetPath}`);
    } catch (e) {
      setGenerateError(formatErr(e));
    } finally {
      setGenerating(false);
    }
  }

  return (
    <div className="flex h-full flex-col">
      <div className="border-b border-border bg-card p-3">
        <div className="mb-2 flex items-center gap-2">
          <label className="text-xs font-medium text-muted-foreground">
            导出模板
          </label>
          <select
            className="flex-1 rounded-md border border-input bg-background px-2 py-1 text-sm"
            value={templateId}
            onChange={(e) => setTemplateId(e.target.value)}
          >
            {templates.map((t) => (
              <option key={t.id} value={t.id}>
                {t.name} {t.type === "Preset" ? "（预设）" : "（自定义）"}
              </option>
            ))}
          </select>
        </div>
        <div className="flex items-center justify-between gap-2">
          <span className="text-xs text-muted-foreground">实时预览</span>
          {onGeneratePdf && (
            <button
              type="button"
              onClick={onGenerate}
              disabled={!templateId || generating}
              className="rounded-md bg-primary px-3 py-1 text-xs text-primary-foreground hover:opacity-90 disabled:opacity-50"
            >
              {generating ? "生成中…" : "生成 PDF…"}
            </button>
          )}
        </div>
        {generating && (
          <div className="mt-2">
            <div className="h-1.5 w-full overflow-hidden rounded-full bg-muted">
              <div className="pdf-progress-bar h-full w-1/3 rounded-full bg-primary" />
            </div>
            <p className="mt-1 text-xs text-muted-foreground">
              正在生成 PDF…
            </p>
          </div>
        )}
        {generateNotice && (
          <p className="mt-2 break-all text-xs text-green-600">
            {generateNotice}
          </p>
        )}
        {generateError && (
          <p className="mt-2 text-xs text-red-600">{generateError}</p>
        )}
        {listError && <p className="mt-2 text-xs text-red-600">{listError}</p>}
      </div>

      <div className="flex-1 overflow-auto bg-muted/40 p-4">
        {!canPreview ? (
          <p className="text-sm text-muted-foreground">
            {notReadyReason ?? "请先填写必填字段才能预览。"}
          </p>
        ) : (
          preview
        )}
      </div>
    </div>
  );
}
