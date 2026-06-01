import { invoke } from "@tauri-apps/api/core";
import type { ImportReport } from "@/types/bindings/ImportReport";

export const importExportApi = {
  importCustomersCsv: (filePath: string) =>
    invoke<ImportReport>("import_customers_csv", { filePath }),
  exportAllExcel: (targetPath: string) =>
    invoke<void>("export_all_excel", { targetPath }),
};
