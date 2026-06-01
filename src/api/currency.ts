import { invoke } from "@tauri-apps/api/core";
import type { ExchangeRate } from "@/types/bindings/ExchangeRate";

export const currencyApi = {
  getRate: (from: string, to: string) =>
    invoke<number>("currency_get_rate", { from, to }),
  convert: (amount: number, from: string, to: string) =>
    invoke<number>("currency_convert", { amount, from, to }),
  refresh: () => invoke<number>("currency_refresh"),
  listCached: () => invoke<ExchangeRate[]>("currency_list_cached"),
};
