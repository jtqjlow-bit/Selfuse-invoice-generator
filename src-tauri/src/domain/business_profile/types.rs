use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/bindings/")]
pub enum EntityType {
    Company,
    Individual,
}

impl EntityType {
    pub fn as_str(self) -> &'static str {
        match self {
            EntityType::Company => "Company",
            EntityType::Individual => "Individual",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "Company" => Some(Self::Company),
            "Individual" => Some(Self::Individual),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/bindings/")]
pub struct BankAccount {
    /// UUID. Empty string for entries created before Slice B; service::*
    /// auto-fills a fresh UUID on next save.
    #[serde(default)]
    pub id: String,
    pub bank_name: String,
    pub account_number: String,
    pub account_holder: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/bindings/")]
pub enum QrKind {
    Bank,
    Tng,
    Boost,
    GrabPay,
    Other,
}

impl QrKind {
    pub fn as_str(self) -> &'static str {
        match self {
            QrKind::Bank => "Bank",
            QrKind::Tng => "Tng",
            QrKind::Boost => "Boost",
            QrKind::GrabPay => "GrabPay",
            QrKind::Other => "Other",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "Bank" => Some(Self::Bank),
            "Tng" => Some(Self::Tng),
            "Boost" => Some(Self::Boost),
            "GrabPay" => Some(Self::GrabPay),
            "Other" => Some(Self::Other),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/bindings/")]
pub struct Qr {
    pub id: String,
    pub kind: QrKind,
    pub label: String,
    pub file_path: String,
}

/// Per-QR data URL payload returned by `business_profile_get_asset_data_urls`.
/// Lets the frontend render QR thumbnails + live preview images without
/// re-encoding on every render.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/bindings/")]
pub struct QrDataUrl {
    pub id: String,
    pub kind: QrKind,
    pub label: String,
    pub data_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/bindings/")]
pub struct ProfileAssetDataUrls {
    pub logo_data_url: Option<String>,
    pub qrs: Vec<QrDataUrl>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/bindings/")]
pub struct BusinessProfile {
    pub id: String,
    pub entity_type: EntityType,
    /// Holds company name OR person name depending on entity_type.
    pub name: String,
    pub address: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    /// Required when entity_type=Company.
    pub ssm_no: Option<String>,
    /// Required when entity_type=Individual.
    pub nric: Option<String>,
    pub sst_no: Option<String>,
    /// Path to copied logo file under `<data_dir>/assets/`. Company only.
    pub logo_path: Option<String>,
    /// Legacy single-QR field, kept for migration; new code uses `qrs`.
    pub qr_path: Option<String>,
    pub bank_accounts: Vec<BankAccount>,
    /// Multi-QR list. Each entry is an image + kind label.
    pub qrs: Vec<Qr>,
    pub enabled_payment_methods: Vec<String>,
    pub default_tax_rate: Option<f64>,
    pub default_quotation_valid_days: i32,
    pub default_invoice_due_days: i32,
    pub data_dir: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/bindings/")]
pub struct CreateBusinessProfileInput {
    pub entity_type: EntityType,
    pub name: String,
    pub address: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub ssm_no: Option<String>,
    pub nric: Option<String>,
    pub sst_no: Option<String>,
    pub bank_accounts: Vec<BankAccount>,
    pub enabled_payment_methods: Vec<String>,
    pub default_tax_rate: Option<f64>,
    pub default_quotation_valid_days: i32,
    pub default_invoice_due_days: i32,
    pub data_dir: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/bindings/")]
pub struct UpdateBusinessProfileInput {
    pub id: String,
    pub entity_type: EntityType,
    pub name: String,
    pub address: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub ssm_no: Option<String>,
    pub nric: Option<String>,
    pub sst_no: Option<String>,
    pub bank_accounts: Vec<BankAccount>,
    pub enabled_payment_methods: Vec<String>,
    pub default_tax_rate: Option<f64>,
    pub default_quotation_valid_days: i32,
    pub default_invoice_due_days: i32,
    pub data_dir: String,
}
