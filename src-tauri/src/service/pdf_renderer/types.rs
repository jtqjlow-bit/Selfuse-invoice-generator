use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::domain::quotation::LineItemInput;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/bindings/")]
pub struct RenderResult {
    /// Absolute path of the PDF written to disk.
    pub output_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/bindings/")]
pub struct QuotationPreviewInput {
    pub template_id: String,
    pub business_profile_id: Option<String>,
    pub customer_id: String,
    pub issue_date: String,
    pub valid_until: String,
    pub currency: String,
    pub tax_enabled: bool,
    pub tax_rate: Option<f64>,
    pub lines: Vec<LineItemInput>,
    pub notes: Option<String>,
    pub terms: Option<String>,
    /// Edit mode shows the saved number; create mode passes None and the
    /// preview falls back to a `(预览)` placeholder.
    pub number: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/bindings/")]
pub struct InvoicePreviewInput {
    pub template_id: String,
    pub business_profile_id: Option<String>,
    pub customer_id: String,
    pub issue_date: String,
    pub due_date: String,
    pub currency: String,
    pub tax_enabled: bool,
    pub tax_rate: Option<f64>,
    pub lines: Vec<LineItemInput>,
    pub notes: Option<String>,
    pub terms: Option<String>,
    pub number: Option<String>,
    pub status: Option<String>,
    /// When present (edit mode), the preview embeds the saved Payment Vouchers.
    pub invoice_id: Option<String>,
    #[serde(default)]
    pub selected_bank_account_ids: Vec<String>,
    #[serde(default)]
    pub selected_qr_ids: Vec<String>,
    #[serde(default)]
    pub selected_static_methods: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/bindings/")]
pub struct PaymentVoucherPreviewInput {
    pub template_id: String,
    pub business_profile_id: Option<String>,
    /// Some = linked PV: customer + currency derived from invoice; balance shown.
    /// None = standalone PV: caller must provide customer_id and currency.
    pub invoice_id: Option<String>,
    pub customer_id: Option<String>,
    pub currency: Option<String>,
    pub date: String,
    pub amount: f64,
    pub payment_method: String,
    pub notes: Option<String>,
    pub number: Option<String>,
}
