use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/bindings/")]
pub enum DocType {
    Quotation,
    Invoice,
    PaymentVoucher,
}

impl DocType {
    pub fn prefix(self) -> &'static str {
        match self {
            DocType::Quotation => "QUO",
            DocType::Invoice => "INV",
            DocType::PaymentVoucher => "PV",
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            DocType::Quotation => "Quotation",
            DocType::Invoice => "Invoice",
            DocType::PaymentVoucher => "PaymentVoucher",
        }
    }
}

pub fn format_number(prefix: &str, year: i32, seq: i64) -> String {
    format!("{prefix}-{year:04}-{seq:03}")
}
