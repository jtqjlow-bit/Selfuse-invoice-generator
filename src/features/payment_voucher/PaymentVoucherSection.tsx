import { useEffect, useState } from "react";
import { Link } from "react-router-dom";
import { paymentVoucherApi } from "@/api/payment_voucher";
import { formatErr, formatMoney } from "@/common/utils/format";
import type { PaymentVoucher } from "@/types/bindings/PaymentVoucher";

interface Props {
  invoiceId: string;
  currency: string;
  /** Allow adding new PVs AND editing existing ones. False = read-only history (Void parent). */
  canEdit: boolean;
}

/**
 * Embeddable card showing Payment Vouchers for one specific invoice.
 * Used inside the InvoiceFormPage when the invoice is in a non-Draft, non-Void state.
 */
export function PaymentVoucherSection({ invoiceId, currency, canEdit }: Props) {
  const [items, setItems] = useState<PaymentVoucher[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    paymentVoucherApi
      .listByInvoice(invoiceId)
      .then(setItems)
      .catch((e) => setError(formatErr(e)))
      .finally(() => setLoading(false));
  }, [invoiceId]);

  return (
    <div className="rounded-md border border-border p-4">
      <div className="mb-3 flex items-center justify-between">
        <h2 className="text-sm font-medium">付款记录</h2>
        {canEdit && (
          <Link
            to={`/payment-vouchers/new?invoice_id=${invoiceId}`}
            className="rounded-md bg-primary px-3 py-1.5 text-xs text-primary-foreground hover:opacity-90"
          >
            + 记录付款
          </Link>
        )}
      </div>

      {error && <p className="mb-2 text-sm text-red-600">{error}</p>}

      {loading ? (
        <p className="text-sm text-muted-foreground">加载中…</p>
      ) : items.length === 0 ? (
        <p className="text-sm text-muted-foreground">
          {canEdit
            ? "还没有付款记录。点右上角'+ 记录付款'添加。"
            : "还没有付款记录。"}
        </p>
      ) : (
        <table className="w-full text-sm">
          <thead className="text-left text-muted-foreground">
            <tr>
              <th className="px-3 py-1 font-normal">编号</th>
              <th className="px-3 py-1 font-normal">日期</th>
              <th className="px-3 py-1 text-right font-normal">金额</th>
              <th className="px-3 py-1 font-normal">方式</th>
              <th className="px-3 py-1 font-normal">备注</th>
              <th className="px-3 py-1 font-normal"></th>
            </tr>
          </thead>
          <tbody>
            {items.map((pv) => (
              <tr key={pv.id} className="border-t border-border">
                <td className="px-3 py-2 font-mono text-xs">{pv.number}</td>
                <td className="px-3 py-2 text-muted-foreground">{pv.date}</td>
                <td className="px-3 py-2 text-right font-mono">
                  {formatMoney(pv.amount, currency)}
                </td>
                <td className="px-3 py-2 text-muted-foreground">
                  {pv.payment_method}
                </td>
                <td
                  className="max-w-xs truncate px-3 py-2 text-muted-foreground"
                  title={pv.notes ?? ""}
                >
                  {pv.notes ?? "—"}
                </td>
                <td className="px-3 py-2 text-right">
                  {canEdit ? (
                    <Link
                      to={`/payment-vouchers/${pv.id}`}
                      className="text-xs text-primary hover:underline"
                    >
                      编辑
                    </Link>
                  ) : (
                    <span className="text-xs text-muted-foreground">
                      只读
                    </span>
                  )}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </div>
  );
}
