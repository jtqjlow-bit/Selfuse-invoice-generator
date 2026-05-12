import type { InvoiceStatus } from "@/types/bindings/InvoiceStatus";

const labels: Record<InvoiceStatus, string> = {
  Draft: "草稿",
  Sent: "已发送",
  PartialPaid: "部分付款",
  Paid: "已付款",
  Overdue: "逾期",
  Void: "作废",
};

const classes: Record<InvoiceStatus, string> = {
  Draft: "bg-muted text-muted-foreground",
  Sent: "bg-blue-100 text-blue-700",
  PartialPaid: "bg-yellow-100 text-yellow-700",
  Paid: "bg-green-100 text-green-700",
  Overdue: "bg-red-100 text-red-700",
  Void: "bg-gray-200 text-gray-500 line-through",
};

export function invoiceStatusLabel(s: InvoiceStatus): string {
  return labels[s];
}

export function invoiceStatusBadgeClass(s: InvoiceStatus): string {
  return classes[s];
}
