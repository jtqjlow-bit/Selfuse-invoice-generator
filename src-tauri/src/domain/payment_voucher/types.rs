use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/bindings/")]
pub struct PaymentVoucher {
    pub id: String,
    pub number: String,
    /// None = standalone PV not tied to any invoice (ad-hoc receipt).
    pub invoice_id: Option<String>,
    pub customer_id: String,
    pub customer_snapshot: serde_json::Value,
    /// None = legacy doc created before multi-profile rollout. Linked PVs
    /// always inherit from the parent invoice's profile_id.
    pub business_profile_id: Option<String>,
    pub date: String,
    pub amount: f64,
    pub currency: String,
    pub payment_method: String,
    pub notes: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/bindings/")]
pub struct CreatePaymentVoucherInput {
    /// Some = linked to an invoice (customer/currency derived from invoice;
    /// invoice.paid_amount is recalculated).
    /// None = standalone PV. Caller MUST then provide customer_id and currency.
    pub invoice_id: Option<String>,
    pub customer_id: Option<String>,
    pub currency: Option<String>,
    /// Used only when invoice_id is None. Linked PVs inherit from the parent
    /// invoice's profile.
    pub business_profile_id: Option<String>,
    pub date: String,
    pub amount: f64,
    pub payment_method: String,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/bindings/")]
pub struct UpdatePaymentVoucherInput {
    pub id: String,
    pub date: String,
    pub amount: f64,
    pub payment_method: String,
    pub notes: Option<String>,
}
