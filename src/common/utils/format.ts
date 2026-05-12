export function formatMoney(amount: number, currency: string): string {
  return `${currency} ${amount.toLocaleString(undefined, {
    minimumFractionDigits: 2,
    maximumFractionDigits: 2,
  })}`;
}

export function formatErr(e: unknown): string {
  if (e && typeof e === "object" && "message" in e) {
    return String((e as { message: string }).message);
  }
  return String(e);
}

export function customerSnapshotName(snapshot: unknown): string {
  if (snapshot && typeof snapshot === "object" && "name" in snapshot) {
    const n = (snapshot as { name?: unknown }).name;
    if (typeof n === "string") return n;
  }
  return "(未知客户)";
}

/** Strip Windows-illegal filename chars (\ / : * ? " < > |) and trim spaces/dots. */
export function sanitizeFilenamePart(s: string): string {
  return s.replace(/[\\/:*?"<>|]/g, "_").replace(/[\s.]+$/g, "").trim();
}
