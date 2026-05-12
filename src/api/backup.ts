import { invoke } from "@tauri-apps/api/core";

export const backupApi = {
  /** Zip up DB + assets + custom templates into `targetPath`. */
  exportZip: (targetPath: string) =>
    invoke<void>("backup_export_zip", { targetPath }),
  /** Validate + stage `zipPath`. Actual swap happens on next app launch. */
  restoreZip: (zipPath: string) =>
    invoke<void>("backup_restore_zip", { zipPath }),
};
