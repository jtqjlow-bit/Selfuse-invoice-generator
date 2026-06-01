import { NavLink, Outlet } from "react-router-dom";
import { cn } from "@/lib/utils";

const navItems = [
  { to: "/dashboard", label: "Dashboard" },
  { to: "/customers", label: "客户" },
  { to: "/quotations", label: "Quotation" },
  { to: "/invoices", label: "Invoice" },
  { to: "/payment-vouchers", label: "Payment Voucher" },
  { to: "/templates", label: "PDF 模板" },
  { to: "/business-profiles", label: "公司资料" },
  { to: "/report", label: "报表" },
  { to: "/backup", label: "备份 / 恢复" },
];

export function AppLayout() {
  return (
    <div className="flex h-screen bg-background text-foreground">
      <aside className="w-48 shrink-0 overflow-y-auto border-r border-border bg-card p-4">
        <h2 className="mb-4 text-sm font-semibold text-muted-foreground">
          Invoice System
        </h2>
        <nav className="flex flex-col gap-1">
          {navItems.map((it) => (
            <NavLink
              key={it.to}
              to={it.to}
              className={({ isActive }) =>
                cn(
                  "rounded-md px-3 py-2 text-sm transition-colors",
                  isActive
                    ? "bg-primary text-primary-foreground"
                    : "text-muted-foreground hover:bg-accent hover:text-accent-foreground",
                )
              }
            >
              {it.label}
            </NavLink>
          ))}
        </nav>
      </aside>
      <main className="flex-1 overflow-auto">
        <Outlet />
      </main>
    </div>
  );
}
