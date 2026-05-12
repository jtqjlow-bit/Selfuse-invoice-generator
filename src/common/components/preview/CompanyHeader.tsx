import type { BusinessProfile } from "@/types/bindings/BusinessProfile";

interface Props {
  profile: BusinessProfile | null;
  logoDataUrl: string | null;
  title: string;
  rows: { label: string; value: string }[];
  status?: string;
}

/** Top band shared by Invoice / Quotation / PV: logo + company meta on the
 *  left, doc title + numbered rows on the right. */
export function CompanyHeader({ profile, logoDataUrl, title, rows, status }: Props) {
  const lines: string[] = [];
  if (profile?.address) lines.push(profile.address);
  const contactBits: string[] = [];
  if (profile?.email) contactBits.push(profile.email);
  if (profile?.phone) contactBits.push(profile.phone);
  if (contactBits.length) lines.push(contactBits.join(" · "));
  const taxBits: string[] = [];
  if (profile?.entity_type === "Company" && profile?.ssm_no) {
    taxBits.push(`SSM: ${profile.ssm_no}`);
  }
  if (profile?.entity_type === "Individual" && profile?.nric) {
    taxBits.push(`NRIC: ${profile.nric}`);
  }
  if (profile?.sst_no) taxBits.push(`SST: ${profile.sst_no}`);
  if (taxBits.length) lines.push(taxBits.join(" · "));

  return (
    <div className="dp-header">
      <div className="dp-company">
        {logoDataUrl && (
          <img src={logoDataUrl} alt="logo" className="dp-logo" />
        )}
        <div className="dp-name">{profile?.name || "(未设置公司)"}</div>
        <div className="dp-meta">{lines.join("\n")}</div>
      </div>
      <div className="dp-doc-meta">
        <div className="dp-title">{title}</div>
        {rows.map((r) => (
          <div key={r.label} className="dp-row">
            <strong>{r.label}</strong>
            {r.value}
          </div>
        ))}
        {status && <div className="dp-status">{status}</div>}
      </div>
    </div>
  );
}
