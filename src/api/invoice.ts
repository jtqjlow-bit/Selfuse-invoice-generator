import { invoke } from "@tauri-apps/api/core";
import type { Invoice } from "@/types/bindings/Invoice";
import type { InvoiceWithLines } from "@/types/bindings/InvoiceWithLines";
import type { CreateInvoiceInput } from "@/types/bindings/CreateInvoiceInput";
import type { UpdateInvoiceInput } from "@/types/bindings/UpdateInvoiceInput";
import type { CreateFromQuotationInput } from "@/types/bindings/CreateFromQuotationInput";

export const invoiceApi = {
  create: (payload: CreateInvoiceInput) =>
    invoke<InvoiceWithLines>("invoice_create", { payload }),
  createFromQuotation: (payload: CreateFromQuotationInput) =>
    invoke<InvoiceWithLines>("invoice_create_from_quotation", { payload }),
  update: (payload: UpdateInvoiceInput) =>
    invoke<InvoiceWithLines>("invoice_update", { payload }),
  findById: (id: string) =>
    invoke<InvoiceWithLines>("invoice_find_by_id", { id }),
  list: () => invoke<Invoice[]>("invoice_list"),
  listByCustomer: (customerId: string) =>
    invoke<Invoice[]>("invoice_list_by_customer", { customerId }),
  markSent: (id: string) => invoke<Invoice>("invoice_mark_sent", { id }),
  markPartialPaid: (id: string) =>
    invoke<Invoice>("invoice_mark_partial_paid", { id }),
  markPaid: (id: string) => invoke<Invoice>("invoice_mark_paid", { id }),
  markOverdue: (id: string) => invoke<Invoice>("invoice_mark_overdue", { id }),
  markVoid: (id: string) => invoke<Invoice>("invoice_mark_void", { id }),
  cancelOverdue: (id: string) =>
    invoke<Invoice>("invoice_cancel_overdue", { id }),
  restoreVoid: (id: string) => invoke<Invoice>("invoice_restore_void", { id }),
};
