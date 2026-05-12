import { memo } from "react";
import "./preview.css";
import { formatMoney } from "@/common/utils/format";
import { BillTo } from "./BillTo";
import { CompanyHeader } from "./CompanyHeader";
import { formatDocDate, type CommonPreviewProps, type LineDraft } from "./types";

interface QuotationPreviewProps extends CommonPreviewProps {
  number: string;
  status: string;
  issueDate: string;
  validUntil: string;
  currency: string;
  taxEnabled: boolean;
  taxRate: number | null;
  lines: LineDraft[];
  notes: string;
  terms: string;
}

function QuotationPreviewInner({
  profile,
  assets,
  customer,
  number,
  status,
  issueDate,
  validUntil,
  currency,
  taxEnabled,
  taxRate,
  lines,
  notes,
  terms,
}: QuotationPreviewProps) {
  const subtotal = lines.reduce(
    (acc, l) => acc + (Number(l.quantity) || 0) * (Number(l.unit_price) || 0),
    0,
  );
  const effectiveRate = taxEnabled ? taxRate ?? 0 : 0;
  const taxAmount = subtotal * effectiveRate;
  const total = subtotal + taxAmount;

  return (
    <div className="dp-page">
      <CompanyHeader
        profile={profile}
        logoDataUrl={assets?.logo_data_url ?? null}
        title="QUOTATION"
        rows={[
          { label: "编号", value: number || "(预览)" },
          { label: "出票日期", value: formatDocDate(issueDate) },
          { label: "有效期至", value: formatDocDate(validUntil) },
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
          </tbody>
        </table>
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

export const QuotationPreview = memo(QuotationPreviewInner);
