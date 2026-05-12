import { useState } from "react";
import { open, save } from "@tauri-apps/plugin-dialog";
import { backupApi } from "@/api/backup";
import { formatErr } from "@/common/utils/format";

function defaultBackupName(): string {
  const d = new Date();
  const yyyy = d.getFullYear();
  const mm = String(d.getMonth() + 1).padStart(2, "0");
  const dd = String(d.getDate()).padStart(2, "0");
  const hh = String(d.getHours()).padStart(2, "0");
  const min = String(d.getMinutes()).padStart(2, "0");
  return `invoice-backup-${yyyy}${mm}${dd}-${hh}${min}.zip`;
}

export function BackupPage() {
  const [busy, setBusy] = useState<"export" | "restore" | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  async function onExport() {
    setError(null);
    setNotice(null);
    let target: string | null;
    try {
      target = await save({
        title: "保存备份",
        defaultPath: defaultBackupName(),
        filters: [{ name: "Zip", extensions: ["zip"] }],
      });
    } catch (e) {
      setError(formatErr(e));
      return;
    }
    if (!target) return;
    setBusy("export");
    try {
      await backupApi.exportZip(target);
      setNotice(`备份已保存到 ${target}`);
    } catch (e) {
      setError(formatErr(e));
    } finally {
      setBusy(null);
    }
  }

  async function onRestore() {
    setError(null);
    setNotice(null);
    if (
      !window.confirm(
        "恢复后，当前的全部数据（客户 / 单据 / 公司资料 / 自定义模板 / Logo / QR）" +
          "都会被备份文件覆盖。\n\n确定继续？",
      )
    ) {
      return;
    }
    let picked: string | string[] | null;
    try {
      picked = await open({
        title: "选择备份文件",
        multiple: false,
        filters: [{ name: "Zip", extensions: ["zip"] }],
      });
    } catch (e) {
      setError(formatErr(e));
      return;
    }
    if (!picked) return;
    const zipPath = Array.isArray(picked) ? picked[0] : picked;
    setBusy("restore");
    try {
      await backupApi.restoreZip(zipPath);
      setNotice(
        "✅ 备份文件已通过验证并暂存。请关闭并重新打开应用程序完成恢复。" +
          "（重启后，之前的数据会被替换。）",
      );
    } catch (e) {
      setError(formatErr(e));
    } finally {
      setBusy(null);
    }
  }

  return (
    <div className="mx-auto max-w-3xl p-8">
      <h1 className="mb-2 text-2xl font-semibold">备份 / 恢复</h1>
      <p className="mb-6 text-sm text-muted-foreground">
        把全部数据（DB + 公司资料的 Logo/QR + 自定义模板）打包成一个 zip
        文件，或者从一个 zip 文件中恢复。
      </p>

      <section className="mb-6 rounded-md border border-border bg-card p-4">
        <h2 className="mb-2 text-sm font-medium">导出备份</h2>
        <p className="mb-3 text-xs text-muted-foreground">
          推荐每月手动导一次，保存到云盘 / 外接硬盘。导出过程不影响应用使用。
        </p>
        <button
          type="button"
          onClick={onExport}
          disabled={busy !== null}
          className="rounded-md bg-primary px-4 py-2 text-sm text-primary-foreground hover:opacity-90 disabled:opacity-50"
        >
          {busy === "export" ? "导出中…" : "导出为 Zip…"}
        </button>
      </section>

      <section className="rounded-md border border-border bg-card p-4">
        <h2 className="mb-2 text-sm font-medium">从备份恢复</h2>
        <p className="mb-3 text-xs text-muted-foreground">
          选择一个之前导出的 zip 文件。恢复需要**重启应用程序**才能完成
          —— 因为 DB
          文件被打开期间无法替换。当前数据会被备份覆盖，操作前请先导一次备份。
        </p>
        <button
          type="button"
          onClick={onRestore}
          disabled={busy !== null}
          className="rounded-md border border-red-300 bg-red-50 px-4 py-2 text-sm text-red-700 hover:bg-red-100 disabled:opacity-50"
        >
          {busy === "restore" ? "处理中…" : "选择 Zip 并恢复…"}
        </button>
      </section>

      {notice && (
        <p className="mt-4 break-all rounded-md border border-green-300 bg-green-50 px-3 py-2 text-sm text-green-700">
          {notice}
        </p>
      )}
      {error && (
        <p className="mt-4 break-all rounded-md border border-red-300 bg-red-50 px-3 py-2 text-sm text-red-700">
          {error}
        </p>
      )}
    </div>
  );
}
