use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/bindings/")]
pub struct ImportRowError {
    /// 1-based line number in the source file (header is line 1).
    pub line: u32,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/bindings/")]
pub struct ImportReport {
    pub total: u32,
    pub imported: u32,
    pub failed: u32,
    pub errors: Vec<ImportRowError>,
}
