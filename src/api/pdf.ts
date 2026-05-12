import { invoke } from "@tauri-apps/api/core";
import type { PdfTemplate } from "@/types/bindings/PdfTemplate";
import type { PdfDocType } from "@/types/bindings/PdfDocType";
import type { UploadCustomTemplateInput } from "@/types/bindings/UploadCustomTemplateInput";
import type { RenderResult } from "@/types/bindings/RenderResult";
import type { QuotationPreviewInput } from "@/types/bindings/QuotationPreviewInput";
import type { InvoicePreviewInput } from "@/types/bindings/InvoicePreviewInput";
import type { PaymentVoucherPreviewInput } from "@/types/bindings/PaymentVoucherPreviewInput";

export const pdfTemplateApi = {
  list: () => invoke<PdfTemplate[]>("pdf_template_list"),
  listByDocType: (docType: PdfDocType) =>
    invoke<PdfTemplate[]>("pdf_template_list_by_doc_type", { docType }),
  findById: (id: string) => invoke<PdfTemplate>("pdf_template_find_by_id", { id }),
  uploadCustom: (payload: UploadCustomTemplateInput) =>
    invoke<PdfTemplate>("pdf_template_upload_custom", { payload }),
  deleteCustom: (id: string) =>
    invoke<void>("pdf_template_delete_custom", { id }),
  getRenderable: (id: string) =>
    invoke<string>("pdf_template_get_renderable", { id }),
  /** Tera-render the template against hardcoded sample data and return the
   *  resulting HTML, for thumbnail display. Cheap (~10 ms). */
  renderSample: (templateId: string) =>
    invoke<string>("pdf_render_template_sample", { templateId }),
};

export const pdfRenderApi = {
  // Final PDF generation (slow, launches headless Chrome). Writes the PDF to
  // the caller-supplied `targetPath`. The caller is responsible for prompting
  // the user (e.g. via @tauri-apps/plugin-dialog `save()`).
  renderQuotation: (quotationId: string, templateId: string, targetPath: string) =>
    invoke<RenderResult>("pdf_render_quotation", {
      quotationId,
      templateId,
      targetPath,
    }),
  renderInvoice: (invoiceId: string, templateId: string, targetPath: string) =>
    invoke<RenderResult>("pdf_render_invoice", {
      invoiceId,
      templateId,
      targetPath,
    }),
  renderPaymentVoucher: (pvId: string, templateId: string, targetPath: string) =>
    invoke<RenderResult>("pdf_render_payment_voucher", {
      pvId,
      templateId,
      targetPath,
    }),

  // Live HTML preview (fast, Tera only, no Chrome).
  previewQuotationHtml: (payload: QuotationPreviewInput) =>
    invoke<string>("pdf_preview_quotation_html", { payload }),
  previewInvoiceHtml: (payload: InvoicePreviewInput) =>
    invoke<string>("pdf_preview_invoice_html", { payload }),
  previewPaymentVoucherHtml: (payload: PaymentVoucherPreviewInput) =>
    invoke<string>("pdf_preview_payment_voucher_html", { payload }),
};
