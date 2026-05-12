import { memo } from "react";
import "./preview.css";
import { formatMoney } from "@/common/utils/format";
import type { PaymentVoucher } from "@/types/bindings/PaymentVoucher";
import { BillTo } from "./BillTo";
import { CompanyHeader } from "./CompanyHeader";
import { formatDocDate, type CommonPreviewProps, type LineDraft } from "./types";

interface InvoicePreviewProps extends CommonPreviewProps {
  number: string;
  status: string;
  issueDate: string;
  dueDate: string;
  currency: string;
  taxEnabled: boolean;
  taxRate: number | null;
  lines: LineDraft[];
  notes: string;
  terms: string;
  /** Saved PVs against this invoice (empty for create mode). */
  payments: PaymentVoucher[];
  paidAmount: number;
  /** Per-invoice payment picker selections. */
  selectedBankAccountIds: string[];
  selectedQrIds: string[];
  selectedStaticMethods: string[];
}

function InvoicePreviewInner({
  profile,
  assets,
  customer,
  number,
  status,
  issueDate,
  dueDate,
  currency,
  taxEnabled,
  taxRate,
  lines,
  notes,
  terms,
  payments,
  paidAmount,
  selectedBankAccountIds,
  selectedQrIds,
  selectedStaticMethods,
}: InvoicePreviewProps) {
  const subtotal = lines.reduce(
    (acc, l) => acc + (Number(l.quantity) || 0) * (Number(l.unit_price) || 0),
    0,
  );
  const effectiveRate = taxEnabled ? taxRate ?? 0 : 0;
  const taxAmount = subtotal * effectiveRate;
  const total = subtotal + taxAmount;
  const balance = total - paidAmount;

  const banks =
    profile?.bank_accounts.filter((b) =>
      selectedBankAccountIds.includes(b.id),
    ) ?? [];
  const qrs =
    profile?.qrs
      .filter((q) => selectedQrIds.includes(q.id))
      .map((q) => ({
        ...q,
        data_url:
          assets?.qrs.find((a) => a.id === q.id)?.data_url ?? "",
      })) ?? [];
  const hasPaymentBlock =
    banks.length > 0 || qrs.length > 0 || selectedStaticMethods.length > 0;
  const hasPayments = payments.length > 0;

  return (
    <div className="dp-page">
      <CompanyHeader
        profile={profile}
        logoDataUrl={assets?.logo_data_url ?? null}
        title="INVOICE"
        rows={[
          { label: "编号", value: number || "(预览)" },
          { label: "出票日期", value: formatDocDate(issueDate) },
          { label: "到期日", value: formatDocDate(dueDate) },
        ]}
        status={status}
      />

      <BillTo label="BILL TO" customer={customer} />

      <table className="dp-items">
        <thead>
          <tr>
            <th className="dp-num">#</th>
            <th>描述</th>
            <th className="dp-qty">数量</th>
            <th className="dp-price">单价</th>
            <th className="dp-amt">小计</th>
          </tr>
        </thead>
        <tbody>
          {lines.map((l, i) => (
            <tr key={i}>
              <td className="dp-num">{i + 1}</td>
              <td>
                <div className="dp-desc">{l.description}</div>
              </td>
              <td className="dp-qty">{l.quantity}</td>
              <td className="dp-price">{formatMoney(l.unit_price, currency)}</td>
              <td className="dp-amt">
                {formatMoney(
                  (Number(l.quantity) || 0) * (Number(l.unit_price) || 0),
                  currency,
                )}
              </td>
            </tr>
          ))}
        </tbody>
      </table>

      <div className="dp-totals">
        <table>
          <tbody>
            <tr>
              <td className="dp-tl">小计</td>
              <td className="dp-tv">{formatMoney(subtotal, currency)}</td>
            </tr>
            {taxEnabled && (
              <tr>
                <td className="dp-tl">税额 ({(effectiveRate * 100).toFixed(0)}%)</td>
                <td className="dp-tv">{formatMoney(taxAmount, currency)}</td>
              </tr>
            )}
            <tr className="dp-grand">
              <td className="dp-tl">合计</td>
              <td className="dp-tv">{formatMoney(total, currency)}</td>
            </tr>
            {paidAmount > 0 && (
              <>
                <tr>
                  <td className="dp-tl">已付</td>
                  <td className="dp-tv">{formatMoney(paidAmount, currency)}</td>
                </tr>
                <tr className={`dp-balance${balance <= 0 ? " dp-zero" : ""}`}>
                  <td className="dp-tl">余额</td>
                  <td className="dp-tv">{formatMoney(balance, currency)}</td>
                </tr>
              </>
            )}
          </tbody>
        </table>
      </div>

      <div className="dp-pay-info">
        <div>
          {hasPaymentBlock && <h4>PAYMENT METHODS</h4>}
          {selectedStaticMethods.length > 0 && (
            <div className="dp-methods">
              {selectedStaticMethods.join(" · ")}
            </div>
          )}
          {banks.length > 0 && (
            <div className="dp-bank">
              {banks.map((b) => (
                <div key={b.id} className="dp-line">
                  {b.bank_name} · {b.account_number} ({b.account_holder})
                </div>
              ))}
            </div>
          )}
          {qrs.length > 0 && (
            <div className="dp-qr-grid">
              {qrs.map((q) => (
                <div key={q.id} className="dp-qr-item">
                  {q.data_url && (
                    <img src={q.data_url} alt={`${q.kind} QR`} />
                  )}
                  <div className="dp-kind">{q.kind}</div>
                  {q.label && <div className="dp-qr-label">{q.label}</div>}
                </div>
              ))}
            </div>
          )}
        </div>
        <div>
          {hasPayments && (
            <>
              <h4>PAYMENT HISTORY</h4>
              <table className="dp-payments">
                <thead>
                  <tr>
                    <th>编号</th>
                    <th>日期</th>
                    <th className="dp-amt">金额</th>
                  </tr>
                </thead>
                <tbody>
                  {payments.map((pv) => (
                    <tr key={pv.id}>
                      <td>{pv.number}</td>
                      <td>{formatDocDate(pv.date)}</td>
                      <td className="dp-amt">
                        {formatMoney(pv.amount, currency)}
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </>
          )}
        </div>
      </div>

      {notes && (
        <div className="dp-footer-block">
          <div className="dp-label">NOTES</div>
          <div className="dp-body">{notes}</div>
        </div>
      )}
      {terms && (
        <div className="dp-footer-block">
          <div className="dp-label">TERMS</div>
          <div className="dp-body">{terms}</div>
        </div>
      )}

      <div className="dp-thanks">— Thank you for your business —</div>
    </div>
  );
}

export const InvoicePreview = memo(InvoicePreviewInner);
