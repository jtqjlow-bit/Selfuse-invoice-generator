import { memo } from "react";
import "./preview.css";
import { formatMoney } from "@/common/utils/format";
import { CompanyHeader } from "./CompanyHeader";
import { formatDocDate, type CommonPreviewProps } from "./types";

interface PaymentVoucherPreviewProps extends CommonPreviewProps {
  number: string;
  date: string;
  amount: number;
  currency: string;
  paymentMethod: string;
  notes: string;
  /** When attached to an invoice, show the reference + post-payment balance. */
  invoiceRef?: {
    number: string;
    issueDate: string;
    total: number;
  };
  balanceAfter?: number | null;
}

function PaymentVoucherPreviewInner({
  profile,
  assets,
  customer,
  number,
  date,
  amount,
  currency,
  paymentMethod,
  notes,
  invoiceRef,
  balanceAfter,
}: PaymentVoucherPreviewProps) {
  return (
    <div className="dp-page">
      <CompanyHeader
        profile={profile}
        logoDataUrl={assets?.logo_data_url ?? null}
        title="PAYMENT VOUCHER"
        rows={[
          { label: "编号", value: number || "(预览)" },
          { label: "日期", value: formatDocDate(date) },
        ]}
      />

      <div className="dp-received-from">
        <div className="dp-label">RECEIVED FROM</div>
        <div className="dp-name">{customer?.name || "(未选择客户)"}</div>
        <div className="dp-meta">
          {[
            customer?.address,
            customer?.ssm_no ? `SSM: ${customer.ssm_no}` : null,
            customer?.nric ? `NRIC: ${customer.nric}` : null,
          ]
            .filter(Boolean)
            .join("\n")}
        </div>
      </div>

      {invoiceRef && (
        <div className="dp-invoice-ref">
          <div className="dp-label">PAYMENT FOR INVOICE</div>
          <div className="dp-ref-line">
            <strong>{invoiceRef.number}</strong> · 合计{" "}
            {formatMoney(invoiceRef.total, currency)} · 出票日期{" "}
            {formatDocDate(invoiceRef.issueDate)}
          </div>
        </div>
      )}

      <div className="dp-amount-block">
        <div className="dp-label">AMOUNT RECEIVED</div>
        <div className="dp-amount">{formatMoney(amount, currency)}</div>
      </div>

      <div className="dp-grid-two">
        <div>
          <div className="dp-label">PAYMENT METHOD</div>
          <div className="dp-value">{paymentMethod || "(未填写)"}</div>
        </div>
        {invoiceRef && balanceAfter != null && (
          <div>
            <div className="dp-label">INVOICE BALANCE AFTER THIS PAYMENT</div>
            <div className="dp-value">{formatMoney(balanceAfter, currency)}</div>
          </div>
        )}
      </div>

      {notes && (
        <div className="dp-footer-block">
          <div className="dp-label">NOTES</div>
          <div className="dp-body">{notes}</div>
        </div>
      )}

      <div className="dp-signature">
        <div className="dp-block">
          <div className="dp-sig-line">&nbsp;</div>
          <div className="dp-sig-name">Received By</div>
        </div>
        <div className="dp-block">
          <div className="dp-sig-line">&nbsp;</div>
          <div className="dp-sig-name">Authorized Signature</div>
        </div>
      </div>
    </div>
  );
}

export const PaymentVoucherPreview = memo(PaymentVoucherPreviewInner);
