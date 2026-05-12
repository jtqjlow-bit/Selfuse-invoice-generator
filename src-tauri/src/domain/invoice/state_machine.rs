use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::error::AppError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/bindings/")]
pub enum InvoiceStatus {
    Draft,
    Sent,
    PartialPaid,
    Paid,
    Overdue,
    Void,
}

impl InvoiceStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            InvoiceStatus::Draft => "Draft",
            InvoiceStatus::Sent => "Sent",
            InvoiceStatus::PartialPaid => "PartialPaid",
            InvoiceStatus::Paid => "Paid",
            InvoiceStatus::Overdue => "Overdue",
            InvoiceStatus::Void => "Void",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "Draft" => Some(Self::Draft),
            "Sent" => Some(Self::Sent),
            "PartialPaid" => Some(Self::PartialPaid),
            "Paid" => Some(Self::Paid),
            "Overdue" => Some(Self::Overdue),
            "Void" => Some(Self::Void),
            _ => None,
        }
    }
}

/// Per CLAUDE.md §6.6:
///   Draft → Sent → (PartialPaid → Paid)
///                   ↓
///                  Overdue (auto; cancellable back to Sent / PartialPaid)
///                   ↓
///                  Void (manual; from anywhere except Draft)
pub fn can_transition(from: InvoiceStatus, to: InvoiceStatus) -> bool {
    use InvoiceStatus::*;
    matches!(
        (from, to),
        (Draft, Sent)
            | (Sent, PartialPaid)
            | (Sent, Paid)
            | (Sent, Overdue)
            | (Sent, Void)
            | (PartialPaid, Paid)
            | (PartialPaid, Overdue)
            | (PartialPaid, Void)
            | (Overdue, Sent)
            | (Overdue, PartialPaid)
            | (Overdue, Paid)
            | (Overdue, Void)
            | (Paid, Void)
    )
}

pub fn transition(current: InvoiceStatus, to: InvoiceStatus) -> Result<InvoiceStatus, AppError> {
    if !can_transition(current, to) {
        return Err(AppError::InvalidTransition {
            from: current.as_str().into(),
            to: to.as_str().into(),
        });
    }
    Ok(to)
}
