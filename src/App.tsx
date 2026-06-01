import { HashRouter, Navigate, Route, Routes } from "react-router-dom";
import { AppLayout } from "@/common/components/AppLayout";
import { DashboardPage } from "@/features/dashboard";
import { CustomerListPage, CustomerFormPage } from "@/features/customer";
import { QuotationListPage, QuotationFormPage } from "@/features/quotation";
import { InvoiceListPage, InvoiceFormPage } from "@/features/invoice";
import {
  PaymentVoucherListPage,
  PaymentVoucherFormPage,
} from "@/features/payment_voucher";
import { TemplatesPage } from "@/features/templates";
import {
  BusinessProfilesListPage,
  BusinessProfileFormPage,
} from "@/features/business-profiles";
import { BackupPage } from "@/features/backup";
import { ReportPage } from "@/features/report";

function App() {
  return (
    <HashRouter>
      <Routes>
        <Route element={<AppLayout />}>
          <Route path="/" element={<Navigate to="/dashboard" replace />} />
          <Route path="/dashboard" element={<DashboardPage />} />
          <Route path="/customers" element={<CustomerListPage />} />
          <Route path="/customers/new" element={<CustomerFormPage />} />
          <Route path="/customers/:id" element={<CustomerFormPage />} />
          <Route path="/quotations" element={<QuotationListPage />} />
          <Route path="/quotations/new" element={<QuotationFormPage />} />
          <Route path="/quotations/:id" element={<QuotationFormPage />} />
          <Route path="/invoices" element={<InvoiceListPage />} />
          <Route path="/invoices/new" element={<InvoiceFormPage />} />
          <Route path="/invoices/:id" element={<InvoiceFormPage />} />
          <Route
            path="/payment-vouchers"
            element={<PaymentVoucherListPage />}
          />
          <Route
            path="/payment-vouchers/new"
            element={<PaymentVoucherFormPage />}
          />
          <Route
            path="/payment-vouchers/:id"
            element={<PaymentVoucherFormPage />}
          />
          <Route path="/templates" element={<TemplatesPage />} />
          <Route
            path="/business-profiles"
            element={<BusinessProfilesListPage />}
          />
          <Route
            path="/business-profiles/new"
            element={<BusinessProfileFormPage />}
          />
          <Route
            path="/business-profiles/:id"
            element={<BusinessProfileFormPage />}
          />
          <Route path="/report" element={<ReportPage />} />
          <Route path="/backup" element={<BackupPage />} />
        </Route>
      </Routes>
    </HashRouter>
  );
}

export default App;
