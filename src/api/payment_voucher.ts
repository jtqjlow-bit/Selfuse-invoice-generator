import { invoke } from "@tauri-apps/api/core";
import type { PaymentVoucher } from "@/types/bindings/PaymentVoucher";
import type { CreatePaymentVoucherInput } from "@/types/bindings/CreatePaymentVoucherInput";
import type { UpdatePaymentVoucherInput } from "@/types/bindings/UpdatePaymentVoucherInput";

export const paymentVoucherApi = {
  create: (payload: CreatePaymentVoucherInput) =>
    invoke<PaymentVoucher>("payment_voucher_create", { payload }),
  update: (payload: UpdatePaymentVoucherInput) =>
    invoke<PaymentVoucher>("payment_voucher_update", { payload }),
  delete: (id: string) => invoke<void>("payment_voucher_delete", { id }),
  findById: (id: string) =>
    invoke<PaymentVoucher>("payment_voucher_find_by_id", { id }),
  list: () => invoke<PaymentVoucher[]>("payment_voucher_list"),
  listByInvoice: (invoiceId: string) =>
    invoke<PaymentVoucher[]>("payment_voucher_list_by_invoice", { invoiceId }),
  listByCustomer: (customerId: string) =>
    invoke<PaymentVoucher[]>("payment_voucher_list_by_customer", {
      customerId,
    }),
  sumByInvoice: (invoiceId: string) =>
    invoke<number>("payment_voucher_sum_by_invoice", { invoiceId }),
};
