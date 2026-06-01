use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/bindings/")]
pub struct ExchangeRate {
    pub base: String,
    pub target: String,
    pub rate: f64,
    /// RFC3339 UTC timestamp of when this rate was last fetched.
    pub fetched_at: String,
}
