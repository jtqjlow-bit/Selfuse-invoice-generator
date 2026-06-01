/** Free-text match against a record's customer name or amount. Empty query matches all. */
export function matchesSearch(name: string, amount: number, query: string): boolean {
  const q = query.trim().toLowerCase();
  if (!q) return true;
  return name.toLowerCase().includes(q) || String(amount).includes(q);
}

/** Whether an ISO `YYYY-MM-DD` date falls within an optional [from, to] range. */
export function inDateRange(date: string, from: string, to: string): boolean {
  if (from && date < from) return false;
  if (to && date > to) return false;
  return true;
}
