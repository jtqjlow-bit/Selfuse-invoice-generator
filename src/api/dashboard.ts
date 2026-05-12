import { invoke } from "@tauri-apps/api/core";
import type { DashboardData } from "@/types/bindings/DashboardData";

export const dashboardApi = {
  getData: () => invoke<DashboardData>("dashboard_get_data"),
};
