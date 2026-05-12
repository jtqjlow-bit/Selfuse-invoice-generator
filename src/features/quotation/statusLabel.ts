import type { QuotationStatus } from "@/types/bindings/QuotationStatus";

const labels: Record<QuotationStatus, string> = {
  Draft: "草稿",
  Sent: "已发送",
  Accepted: "已接受",
  Rejected: "已拒绝",
  Expired: "已过期",
};

const classes: Record<QuotationStatus, string> = {
  Draft: "bg-muted text-muted-foreground",
  Sent: "bg-blue-100 text-blue-700",
  Accepted: "bg-green-100 text-green-700",
  Rejected: "bg-red-100 text-red-700",
  Expired: "bg-orange-100 text-orange-700",
};

export function quotationStatusLabel(s: QuotationStatus): string {
  return labels[s];
}

export function quotationStatusBadgeClass(s: QuotationStatus): string {
  return classes[s];
}
