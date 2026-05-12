use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Local copy of the doc-type enum. Lives here instead of being borrowed from
/// `service::numbering::DocType` because CLAUDE.md §5 forbids domain → service
/// dependencies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/bindings/")]
pub enum PdfDocType {
    Quotation,
    Invoice,
    PaymentVoucher,
}

impl PdfDocType {
    pub fn as_str(self) -> &'static str {
        match self {
            PdfDocType::Quotation => "Quotation",
            PdfDocType::Invoice => "Invoice",
            PdfDocType::PaymentVoucher => "PaymentVoucher",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "Quotation" => Some(Self::Quotation),
            "Invoice" => Some(Self::Invoice),
            "PaymentVoucher" => Some(Self::PaymentVoucher),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/bindings/")]
pub enum PdfTemplateType {
    Preset,
    Custom,
}

impl PdfTemplateType {
    pub fn as_str(self) -> &'static str {
        match self {
            PdfTemplateType::Preset => "Preset",
            PdfTemplateType::Custom => "Custom",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "Preset" => Some(Self::Preset),
            "Custom" => Some(Self::Custom),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/bindings/")]
pub struct PdfTemplate {
    pub id: String,
    pub doc_type: PdfDocType,
    pub name: String,
    #[serde(rename = "type")]
    pub type_: PdfTemplateType,
    pub file_path: String,
    pub config_json: serde_json::Value,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/bindings/")]
pub struct UploadCustomTemplateInput {
    pub doc_type: PdfDocType,
    pub name: String,
    /// Raw HTML content (with Tera placeholders). Server writes it to disk.
    pub html_content: String,
}
