use rust_xlsxwriter::Workbook;
use serde::Deserialize;

use crate::domain::{customer, invoice, payment_voucher, quotation};
use crate::error::{AppError, AppResult};
use crate::infra::{file_system, Db};

use super::types::{ImportReport, ImportRowError};

#[derive(Debug, Deserialize)]
struct CsvCustomerRow {
    #[serde(rename = "type")]
    type_: String,
    name: String,
    contact_person: Option<String>,
    email: Option<String>,
    phone: Option<String>,
    address: Option<String>,
    ssm_no: Option<String>,
    nric: Option<String>,
    tax_no: Option<String>,
    notes: Option<String>,
}

fn blank_to_none(v: Option<String>) -> Option<String> {
    v.map(|s| s.trim().to_string()).filter(|s| !s.is_empty())
}

/// Import customers from a CSV file. Every data row is created as a new
/// customer (no dedup). Bad rows are skipped and reported; valid rows still
/// import. The expected header is:
/// `type,name,contact_person,email,phone,address,ssm_no,nric,tax_no,notes`.
pub fn import_customers_from_csv(db: &Db, file_path: &str) -> AppResult<ImportReport> {
    let content = file_system::read_file(file_path)?;
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .trim(csv::Trim::All)
        .from_reader(content.as_bytes());

    let mut total = 0u32;
    let mut imported = 0u32;
    let mut errors: Vec<ImportRowError> = Vec::new();

    for (idx, result) in rdr.deserialize::<CsvCustomerRow>().enumerate() {
        total += 1;
        // header is line 1; first data record (idx 0) is line 2.
        let line = idx as u32 + 2;
        let row = match result {
            Ok(r) => r,
            Err(e) => {
                errors.push(ImportRowError {
                    line,
                    message: format!("解析失败: {e}"),
                });
                continue;
            }
        };

        let type_ = match customer::CustomerType::from_str(row.type_.trim()) {
            Some(t) => t,
            None => {
                errors.push(ImportRowError {
                    line,
                    message: format!("非法 type: {}（应为 Company 或 Individual）", row.type_),
                });
                continue;
            }
        };
        if row.name.trim().is_empty() {
            errors.push(ImportRowError {
                line,
                message: "name 不能为空".into(),
            });
            continue;
        }

        let input = customer::CreateCustomerInput {
            type_,
            name: row.name.trim().to_string(),
            contact_person: blank_to_none(row.contact_person),
            email: blank_to_none(row.email),
            phone: blank_to_none(row.phone),
            address: blank_to_none(row.address),
            ssm_no: blank_to_none(row.ssm_no),
            nric: blank_to_none(row.nric),
            tax_no: blank_to_none(row.tax_no),
            notes: blank_to_none(row.notes),
        };

        match customer::create(db, input) {
            Ok(_) => imported += 1,
            Err(e) => errors.push(ImportRowError {
                line,
                message: format!("创建失败: {}", err_msg(&e)),
            }),
        }
    }

    Ok(ImportReport {
        total,
        imported,
        failed: total - imported,
        errors,
    })
}

fn err_msg(e: &AppError) -> String {
    e.to_string()
}

fn snapshot_name(snapshot: &serde_json::Value) -> String {
    snapshot
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string()
}

/// Export every customer / quotation / invoice / payment voucher into a
/// multi-sheet .xlsx written to `target_path`.
pub fn export_all_to_excel(db: &Db, target_path: &str) -> AppResult<()> {
    let customers = customer::list(db, true)?;
    let quotations = quotation::list(db)?;
    let invoices = invoice::list(db)?;
    let vouchers = payment_voucher::list(db)?;

    let mut wb = Workbook::new();

    {
        let ws = wb
            .add_worksheet()
            .set_name("客户")
            .map_err(xlsx_err)?;
        let headers = [
            "type", "name", "contact_person", "email", "phone", "address", "ssm_no", "nric",
            "tax_no", "notes", "archived", "created_at",
        ];
        write_headers(ws, &headers)?;
        for (i, c) in customers.iter().enumerate() {
            let r = i as u32 + 1;
            write_str(ws, r, 0, c.type_.as_str())?;
            write_str(ws, r, 1, &c.name)?;
            write_opt(ws, r, 2, &c.contact_person)?;
            write_opt(ws, r, 3, &c.email)?;
            write_opt(ws, r, 4, &c.phone)?;
            write_opt(ws, r, 5, &c.address)?;
            write_opt(ws, r, 6, &c.ssm_no)?;
            write_opt(ws, r, 7, &c.nric)?;
            write_opt(ws, r, 8, &c.tax_no)?;
            write_opt(ws, r, 9, &c.notes)?;
            write_str(ws, r, 10, if c.archived { "是" } else { "否" })?;
            write_str(ws, r, 11, &c.created_at)?;
        }
    }

    {
        let ws = wb
            .add_worksheet()
            .set_name("报价")
            .map_err(xlsx_err)?;
        let headers = [
            "number", "customer", "issue_date", "valid_until", "currency", "subtotal",
            "tax_amount", "total", "status", "created_at",
        ];
        write_headers(ws, &headers)?;
        for (i, q) in quotations.iter().enumerate() {
            let r = i as u32 + 1;
            write_str(ws, r, 0, &q.number)?;
            write_str(ws, r, 1, &snapshot_name(&q.customer_snapshot))?;
            write_str(ws, r, 2, &q.issue_date)?;
            write_str(ws, r, 3, &q.valid_until)?;
            write_str(ws, r, 4, &q.currency)?;
            write_num(ws, r, 5, q.subtotal)?;
            write_num(ws, r, 6, q.tax_amount)?;
            write_num(ws, r, 7, q.total)?;
            write_str(ws, r, 8, &format!("{:?}", q.status))?;
            write_str(ws, r, 9, &q.created_at)?;
        }
    }

    {
        let ws = wb
            .add_worksheet()
            .set_name("发票")
            .map_err(xlsx_err)?;
        let headers = [
            "number", "customer", "issue_date", "due_date", "currency", "subtotal",
            "tax_amount", "total", "paid_amount", "status", "created_at",
        ];
        write_headers(ws, &headers)?;
        for (i, inv) in invoices.iter().enumerate() {
            let r = i as u32 + 1;
            write_str(ws, r, 0, &inv.number)?;
            write_str(ws, r, 1, &snapshot_name(&inv.customer_snapshot))?;
            write_str(ws, r, 2, &inv.issue_date)?;
            write_str(ws, r, 3, &inv.due_date)?;
            write_str(ws, r, 4, &inv.currency)?;
            write_num(ws, r, 5, inv.subtotal)?;
            write_num(ws, r, 6, inv.tax_amount)?;
            write_num(ws, r, 7, inv.total)?;
            write_num(ws, r, 8, inv.paid_amount)?;
            write_str(ws, r, 9, &format!("{:?}", inv.status))?;
            write_str(ws, r, 10, &inv.created_at)?;
        }
    }

    {
        let ws = wb
            .add_worksheet()
            .set_name("收款凭证")
            .map_err(xlsx_err)?;
        let headers = [
            "number", "customer", "date", "amount", "currency", "payment_method", "created_at",
        ];
        write_headers(ws, &headers)?;
        for (i, pv) in vouchers.iter().enumerate() {
            let r = i as u32 + 1;
            write_str(ws, r, 0, &pv.number)?;
            write_str(ws, r, 1, &snapshot_name(&pv.customer_snapshot))?;
            write_str(ws, r, 2, &pv.date)?;
            write_num(ws, r, 3, pv.amount)?;
            write_str(ws, r, 4, &pv.currency)?;
            write_str(ws, r, 5, &pv.payment_method)?;
            write_str(ws, r, 6, &pv.created_at)?;
        }
    }

    let buf = wb.save_to_buffer().map_err(xlsx_err)?;
    file_system::write_bytes(target_path, &buf)?;
    Ok(())
}

fn xlsx_err(e: rust_xlsxwriter::XlsxError) -> AppError {
    AppError::Internal(format!("xlsx 写入失败: {e}"))
}

fn write_headers(ws: &mut rust_xlsxwriter::Worksheet, headers: &[&str]) -> AppResult<()> {
    for (col, h) in headers.iter().enumerate() {
        write_str(ws, 0, col as u16, h)?;
    }
    Ok(())
}

fn write_str(ws: &mut rust_xlsxwriter::Worksheet, row: u32, col: u16, v: &str) -> AppResult<()> {
    ws.write_string(row, col, v).map_err(xlsx_err)?;
    Ok(())
}

fn write_opt(
    ws: &mut rust_xlsxwriter::Worksheet,
    row: u32,
    col: u16,
    v: &Option<String>,
) -> AppResult<()> {
    if let Some(s) = v {
        write_str(ws, row, col, s)?;
    }
    Ok(())
}

fn write_num(ws: &mut rust_xlsxwriter::Worksheet, row: u32, col: u16, v: f64) -> AppResult<()> {
    ws.write_number(row, col, v).map_err(xlsx_err)?;
    Ok(())
}
