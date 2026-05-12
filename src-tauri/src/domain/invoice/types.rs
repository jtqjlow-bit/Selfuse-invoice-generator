use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::domain::quotation::LineItemInput;

use super::state_machine::InvoiceStatus;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/bindings/")]
pub struct Invoice {
    pub id: String,
    pub number: String,
    pub customer_id: String,
    pub customer_snapshot: serde_json::Value,
    /// None = legacy doc created before multi-profile rollout.
    pub business_profile_id: Option<String>,
    pub source_quotation_id: Option<String>,
    pub issue_date: String,
    pub due_date: String,
    pub currency: String,
    pub tax_enabled: bool,
    pub tax_rate: Option<f64>,
    pub subtotal: f64,
    pub tax_amount: f64,
    pub total: f64,
    pub paid_amount: f64,
    pub payment_methods_snapshot: serde_json::Value,
    /// IDs of bank accounts (from the chosen business profile) to show on the PDF.
    pub selected_bank_account_ids: Vec<String>,
    /// IDs of QRs (from the chosen business profile) to show on the PDF.
    pub selected_qr_ids: Vec<String>,
    /// Plain-string payment methods to show (e.g. "Cash", "Cheque", "FPX").
    pub selected_static_methods: Vec<String>,
    pub status: InvoiceStatus,
    pub notes: Option<String>,
    pub terms: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/bindings/")]
pub struct InvoiceLineItem {
    pub id: String,
    pub invoice_id: String,
    pub position: i32,
    pub description: String,
    pub quantity: f64,
    pub unit_price: f64,
    pub line_total: f64,
    pub line_currency: String,
    pub exchange_rate_to_doc_currency: Option<f64>,
    pub tax_rate: Option<f64>,
    pub discount_rate: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/bindings/")]
pub struct InvoiceWithLines {
    pub invoice: Invoice,
    pub lines: Vec<InvoiceLineItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/bindings/")]
pub struct CreateInvoiceInput {
    pub customer_id: String,
    pub business_profile_id: Option<String>,
    pub issue_date: String,
    pub due_date: String,
    pub currency: String,
    pub tax_enabled: bool,
    pub tax_rate: Option<f64>,
    pub lines: Vec<LineItemInput>,
    pub notes: Option<String>,
    pub terms: Option<String>,
    #[serde(default)]
    pub selected_bank_account_ids: Vec<String>,
    #[serde(default)]
    pub selected_qr_ids: Vec<String>,
    #[serde(default)]
    pub selected_static_methods: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/bindings/")]
pub struct UpdateInvoiceInput {
    pub id: String,
    pub customer_id: String,
    pub business_profile_id: Option<String>,
    pub issue_date: String,
    pub due_date: String,
    pub currency: String,
    pub tax_enabled: bool,
    pub tax_rate: Option<f64>,
    pub lines: Vec<LineItemInput>,
    pub notes: Option<String>,
    pub terms: Option<String>,
    #[serde(default)]
    pub selected_bank_account_ids: Vec<String>,
    #[serde(default)]
    pub selected_qr_ids: Vec<String>,
    #[serde(default)]
    pub selected_static_methods: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/bindings/")]
pub struct CreateFromQuotationInput {
    pub quotation_id: String,
    pub business_profile_id: Option<String>,
    pub issue_date: String,
    pub due_date: String,
}
