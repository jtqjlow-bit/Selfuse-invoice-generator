use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/bindings/")]
pub enum CustomerType {
    Company,
    Individual,
}

impl CustomerType {
    pub fn as_str(self) -> &'static str {
        match self {
            CustomerType::Company => "Company",
            CustomerType::Individual => "Individual",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "Company" => Some(CustomerType::Company),
            "Individual" => Some(CustomerType::Individual),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/bindings/")]
pub struct Customer {
    pub id: String,
    #[serde(rename = "type")]
    pub type_: CustomerType,
    pub name: String,
    pub contact_person: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub ssm_no: Option<String>,
    pub nric: Option<String>,
    pub tax_no: Option<String>,
    pub notes: Option<String>,
    pub archived: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/bindings/")]
pub struct CreateCustomerInput {
    #[serde(rename = "type")]
    pub type_: CustomerType,
    pub name: String,
    pub contact_person: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub ssm_no: Option<String>,
    pub nric: Option<String>,
    pub tax_no: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/bindings/")]
pub struct UpdateCustomerInput {
    pub id: String,
    #[serde(rename = "type")]
    pub type_: CustomerType,
    pub name: String,
    pub contact_person: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub ssm_no: Option<String>,
    pub nric: Option<String>,
    pub tax_no: Option<String>,
    pub notes: Option<String>,
}
