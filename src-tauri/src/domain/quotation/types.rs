use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::state_machine::QuotationStatus;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/bindings/")]
pub struct Quotation {
    pub id: String,
    pub number: String,
    pub customer_id: String,
    pub customer_snapshot: serde_json::Value,
    /// None = legacy doc created before multi-profile rollout. Renderer treats
    /// missing profile as blank company info.
    pub business_profile_id: Option<String>,
    pub issue_date: String,
    pub valid_until: String,
    pub currency: String,
    pub tax_enabled: bool,
    pub tax_rate: Option<f64>,
    pub subtotal: f64,
    pub tax_amount: f64,
    pub total: f64,
    pub status: QuotationStatus,
    pub converted_invoice_id: Option<String>,
    pub notes: Option<String>,
    pub terms: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/bindings/")]
pub struct QuotationLineItem {
    pub id: String,
    pub quotation_id: String,
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
pub struct QuotationWithLines {
    pub quotation: Quotation,
    pub lines: Vec<QuotationLineItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/bindings/")]
pub struct LineItemInput {
    pub description: String,
    pub quantity: f64,
    pub unit_price: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/bindings/")]
pub struct CreateQuotationInput {
    pub customer_id: String,
    pub business_profile_id: Option<String>,
    pub issue_date: String,
    pub valid_until: String,
    pub currency: String,
    pub tax_enabled: bool,
    pub tax_rate: Option<f64>,
    pub lines: Vec<LineItemInput>,
    pub notes: Option<String>,
    pub terms: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/bindings/")]
pub struct UpdateQuotationInput {
    pub id: String,
    pub customer_id: String,
    pub business_profile_id: Option<String>,
    pub issue_date: String,
    pub valid_until: String,
    pub currency: String,
    pub tax_enabled: bool,
    pub tax_rate: Option<f64>,
    pub lines: Vec<LineItemInput>,
    pub notes: Option<String>,
    pub terms: Option<String>,
}
