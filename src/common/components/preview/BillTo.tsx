import type { Customer } from "@/types/bindings/Customer";

interface Props {
  label: string;
  customer: Customer | null;
}

/** "BILL TO" / "RECEIVED FROM" customer block. */
export function BillTo({ label, customer }: Props) {
  const meta: string[] = [];
  if (customer?.address) meta.push(customer.address);
  if (customer?.contact_person) meta.push(`联系人：${customer.contact_person}`);
  const contactBits: string[] = [];
  if (customer?.email) contactBits.push(customer.email);
  if (customer?.phone) contactBits.push(customer.phone);
  if (contactBits.length) meta.push(contactBits.join(" · "));
  if (customer?.ssm_no) meta.push(`SSM: ${customer.ssm_no}`);
  if (customer?.nric) meta.push(`NRIC: ${customer.nric}`);
  if (customer?.tax_no) meta.push(`税号: ${customer.tax_no}`);

  return (
    <div className="dp-bill-to">
      <div className="dp-label">{label}</div>
      <div className="dp-name">{customer?.name || "(未选择客户)"}</div>
      <div className="dp-meta">{meta.join("\n")}</div>
    </div>
  );
}
