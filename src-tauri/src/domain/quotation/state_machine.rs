use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::error::AppError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/bindings/")]
pub enum QuotationStatus {
    Draft,
    Sent,
    Accepted,
    Rejected,
    Expired,
}

impl QuotationStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            QuotationStatus::Draft => "Draft",
            QuotationStatus::Sent => "Sent",
            QuotationStatus::Accepted => "Accepted",
            QuotationStatus::Rejected => "Rejected",
            QuotationStatus::Expired => "Expired",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "Draft" => Some(Self::Draft),
            "Sent" => Some(Self::Sent),
            "Accepted" => Some(Self::Accepted),
            "Rejected" => Some(Self::Rejected),
            "Expired" => Some(Self::Expired),
            _ => None,
        }
    }
}

pub fn can_transition(from: QuotationStatus, to: QuotationStatus) -> bool {
    use QuotationStatus::*;
    matches!(
        (from, to),
        (Draft, Sent) | (Sent, Accepted) | (Sent, Rejected) | (Sent, Expired)
    )
}

pub fn transition(current: QuotationStatus, to: QuotationStatus) -> Result<QuotationStatus, AppError> {
    if !can_transition(current, to) {
        return Err(AppError::InvalidTransition {
            from: current.as_str().into(),
            to: to.as_str().into(),
        });
    }
    Ok(to)
}
