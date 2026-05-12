import type { BusinessProfile } from "@/types/bindings/BusinessProfile";
import type { ProfileAssetDataUrls } from "@/types/bindings/ProfileAssetDataUrls";
import type { Customer } from "@/types/bindings/Customer";

/** Subset of fields each preview needs out of the form state. Tighter than
 *  passing the whole form so a preview can't accidentally depend on UI-only
 *  fields. */
export interface LineDraft {
  description: string;
  quantity: number;
  unit_price: number;
}

export interface CommonPreviewProps {
  profile: BusinessProfile | null;
  /** Data URLs for profile.logo_path + each profile.qrs[*].file_path. Loaded
   *  once via business_profile_get_asset_data_urls; updated only when the
   *  selected profile changes. */
  assets: ProfileAssetDataUrls | null;
  /** Selected customer object (or null if not selected yet). For new docs we
   *  render the live customer; for saved docs the parent passes a synthetic
   *  Customer built from `invoice.customer_snapshot` so the preview always
   *  shows what will appear on the PDF. */
  customer: Customer | null;
}

/** Format DD MMM YYYY to match Tera `date(format="%d %b %Y")`. Accepts an ISO
 *  date string (YYYY-MM-DD) or empty string. */
export function formatDocDate(iso: string): string {
  if (!iso) return "";
  const [y, m, d] = iso.split("-").map(Number);
  if (!y || !m || !d) return iso;
  const month = ["Jan", "Feb", "Mar", "Apr", "May", "Jun",
                 "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"][m - 1];
  return `${String(d).padStart(2, "0")} ${month} ${y}`;
}
