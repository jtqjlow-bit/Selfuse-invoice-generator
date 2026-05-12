import { invoke } from "@tauri-apps/api/core";
import type { Quotation } from "@/types/bindings/Quotation";
import type { QuotationWithLines } from "@/types/bindings/QuotationWithLines";
import type { CreateQuotationInput } from "@/types/bindings/CreateQuotationInput";
import type { UpdateQuotationInput } from "@/types/bindings/UpdateQuotationInput";

export const quotationApi = {
  create: (payload: CreateQuotationInput) =>
    invoke<QuotationWithLines>("quotation_create", { payload }),
  update: (payload: UpdateQuotationInput) =>
    invoke<QuotationWithLines>("quotation_update", { payload }),
  findById: (id: string) =>
    invoke<QuotationWithLines>("quotation_find_by_id", { id }),
  list: () => invoke<Quotation[]>("quotation_list"),
  listByCustomer: (customerId: string) =>
    invoke<Quotation[]>("quotation_list_by_customer", { customerId }),
  markSent: (id: string) => invoke<Quotation>("quotation_mark_sent", { id }),
  markAccepted: (id: string) =>
    invoke<Quotation>("quotation_mark_accepted", { id }),
  markRejected: (id: string) =>
    invoke<Quotation>("quotation_mark_rejected", { id }),
  markExpired: (id: string) =>
    invoke<Quotation>("quotation_mark_expired", { id }),
};
