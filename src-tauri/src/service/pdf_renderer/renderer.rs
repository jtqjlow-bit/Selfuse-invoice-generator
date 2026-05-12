//! PDF renderer = Tera (HTML template) + headless_chrome (HTML → PDF).
//!
//! Public API: `render_quotation` / `render_invoice` / `render_payment_voucher`.
//! Each function:
//!   1. Fetches the doc (with lines / PVs) and company settings from `Db`
//!   2. Loads the template HTML via `pdf_template::get_renderable`
//!   3. Builds a `serde_json::Value` context
//!   4. Renders Tera → HTML string
//!   5. Writes HTML to a temp file under `app_data_dir/.pdf_tmp/`
//!   6. Launches headless Chrome / Edge, navigates to file://, prints to PDF
//!   7. Writes the PDF to the caller-specified `target_path`
use std::path::{Path, PathBuf};

use base64::{engine::general_purpose::STANDARD as B64, Engine};
use headless_chrome::{types::PrintToPdfOptions, Browser, LaunchOptionsBuilder};
use serde_json::json;
use tera::Tera;
use uuid::Uuid;

use crate::domain::business_profile::BusinessProfile;
use crate::domain::business_profile::EntityType;
use crate::domain::{business_profile, invoice, payment_voucher, pdf_template, quotation};
use crate::error::{AppError, AppResult};
use crate::infra::{file_system, Db};

use super::filters;
use super::types::{
    InvoicePreviewInput, PaymentVoucherPreviewInput, QuotationPreviewInput, RenderResult,
};
use crate::service::tax_calc::{self, LineForTotals};

/// Serialize BusinessProfile to a JSON value with the on-disk logo/qr files
/// inlined as `data:image/<ext>;base64,...` URLs under `logo_data_url` and
/// `qr_data_url`. Inlining lets the preview iframe (origin-less sandbox) and
/// the headless-Chrome PDF run both render the images uniformly without any
/// file:// permission shenanigans.
///
/// Also aliases `name` to `company_name` so the existing preset templates
/// keep working without edits.
fn build_company_context(company: &BusinessProfile) -> AppResult<serde_json::Value> {
    let mut v = serde_json::to_value(company)
        .map_err(|e| AppError::Internal(format!("serialize company: {e}")))?;
    // Templates still reference `company.company_name`. Keep that working.
    v["company_name"] = serde_json::Value::String(company.name.clone());
    if let Some(p) = company.logo_path.as_deref() {
        if let Some(url) = file_to_data_url(p) {
            v["logo_data_url"] = serde_json::Value::String(url);
        }
    }
    if let Some(p) = company.qr_path.as_deref() {
        if let Some(url) = file_to_data_url(p) {
            v["qr_data_url"] = serde_json::Value::String(url);
        }
    }
    Ok(v)
}

/// Resolve the doc's business_profile_id to a profile. Falls back to a blank
/// placeholder so legacy docs (created before multi-profile rollout, or with
/// a since-deleted profile) still render without erroring.
fn resolve_profile(db: &Db, profile_id: Option<&str>) -> BusinessProfile {
    if let Some(id) = profile_id {
        if let Ok(p) = business_profile::find_by_id(db, id) {
            return p;
        }
    }
    blank_profile()
}

fn blank_profile() -> BusinessProfile {
    BusinessProfile {
        id: String::new(),
        entity_type: EntityType::Company,
        name: String::new(),
        address: None,
        email: None,
        phone: None,
        ssm_no: None,
        nric: None,
        sst_no: None,
        logo_path: None,
        qr_path: None,
        bank_accounts: Vec::new(),
        qrs: Vec::new(),
        enabled_payment_methods: Vec::new(),
        default_tax_rate: None,
        default_quotation_valid_days: 30,
        default_invoice_due_days: 30,
        data_dir: String::new(),
        created_at: String::new(),
        updated_at: String::new(),
    }
}

/// Resolve a doc's selected_bank_account_ids / selected_qr_ids /
/// selected_static_methods against `profile`. Returns (bank_accounts, qrs, statics)
/// with qrs already embedded as data URLs. Missing IDs are silently skipped.
fn resolve_selected_payments(
    profile: &BusinessProfile,
    bank_ids: &[String],
    qr_ids: &[String],
    statics: &[String],
) -> (serde_json::Value, serde_json::Value, serde_json::Value) {
    let banks: Vec<serde_json::Value> = bank_ids
        .iter()
        .filter_map(|id| profile.bank_accounts.iter().find(|b| &b.id == id))
        .map(|b| {
            serde_json::json!({
                "bank_name": b.bank_name,
                "account_number": b.account_number,
                "account_holder": b.account_holder,
            })
        })
        .collect();
    let qrs: Vec<serde_json::Value> = qr_ids
        .iter()
        .filter_map(|id| profile.qrs.iter().find(|q| &q.id == id))
        .map(|q| {
            let url = file_to_data_url(&q.file_path).unwrap_or_default();
            serde_json::json!({
                "kind": q.kind.as_str(),
                "label": q.label,
                "data_url": url,
            })
        })
        .collect();
    let sm: Vec<serde_json::Value> = statics
        .iter()
        .map(|s| serde_json::Value::String(s.clone()))
        .collect();
    (
        serde_json::Value::Array(banks),
        serde_json::Value::Array(qrs),
        serde_json::Value::Array(sm),
    )
}

fn file_to_data_url(path: &str) -> Option<String> {
    let p = Path::new(path);
    let bytes = std::fs::read(p).ok()?;
    if bytes.is_empty() {
        return None;
    }
    let ext = p
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("png")
        .to_lowercase();
    let mime = match ext.as_str() {
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "svg" => "image/svg+xml",
        _ => "application/octet-stream",
    };
    Some(format!("data:{mime};base64,{}", B64.encode(&bytes)))
}

pub fn render_quotation(
    db: &Db,
    app_data_dir: &Path,
    quotation_id: &str,
    template_id: &str,
    target_path: &Path,
) -> AppResult<RenderResult> {
    let qwl = quotation::find_by_id(db, quotation_id)?;
    let company = resolve_profile(db, qwl.quotation.business_profile_id.as_deref());
    let template_html = pdf_template::get_renderable(db, template_id)?;

    let context = json!({
        "company": build_company_context(&company)?,
        "customer": &qwl.quotation.customer_snapshot,
        "quotation": {
            "number": qwl.quotation.number,
            "issue_date": qwl.quotation.issue_date,
            "valid_until": qwl.quotation.valid_until,
            "currency": qwl.quotation.currency,
            "tax_enabled": qwl.quotation.tax_enabled,
            "tax_rate": qwl.quotation.tax_rate.unwrap_or(0.0),
            "subtotal": qwl.quotation.subtotal,
            "tax_amount": qwl.quotation.tax_amount,
            "total": qwl.quotation.total,
            "notes": qwl.quotation.notes,
            "terms": qwl.quotation.terms,
            "status": qwl.quotation.status.as_str(),
        },
        "lines": &qwl.lines,
    });

    render_to_pdf(&template_html, &context, app_data_dir, target_path)?;
    Ok(RenderResult {
        output_path: target_path.to_string_lossy().to_string(),
    })
}

pub fn render_invoice(
    db: &Db,
    app_data_dir: &Path,
    invoice_id: &str,
    template_id: &str,
    target_path: &Path,
) -> AppResult<RenderResult> {
    let iwl = invoice::find_by_id(db, invoice_id)?;
    let company = resolve_profile(db, iwl.invoice.business_profile_id.as_deref());
    let payments = payment_voucher::list_by_invoice(db, invoice_id)?;
    let template_html = pdf_template::get_renderable(db, template_id)?;
    let (sel_banks, sel_qrs, sel_statics) = resolve_selected_payments(
        &company,
        &iwl.invoice.selected_bank_account_ids,
        &iwl.invoice.selected_qr_ids,
        &iwl.invoice.selected_static_methods,
    );

    let balance = iwl.invoice.total - iwl.invoice.paid_amount;

    let context = json!({
        "company": build_company_context(&company)?,
        "customer": &iwl.invoice.customer_snapshot,
        "invoice": {
            "number": iwl.invoice.number,
            "issue_date": iwl.invoice.issue_date,
            "due_date": iwl.invoice.due_date,
            "currency": iwl.invoice.currency,
            "tax_enabled": iwl.invoice.tax_enabled,
            "tax_rate": iwl.invoice.tax_rate.unwrap_or(0.0),
            "subtotal": iwl.invoice.subtotal,
            "tax_amount": iwl.invoice.tax_amount,
            "total": iwl.invoice.total,
            "paid_amount": iwl.invoice.paid_amount,
            "balance": balance,
            "notes": iwl.invoice.notes,
            "terms": iwl.invoice.terms,
            "status": iwl.invoice.status.as_str(),
        },
        "lines": &iwl.lines,
        "payments": &payments,
        // Per-invoice picker resolves against the current profile at render
        // time. Missing IDs (deleted bank/qr) are silently skipped.
        "selected_bank_accounts": sel_banks,
        "selected_qrs": sel_qrs,
        "selected_static_methods": sel_statics,
    });

    render_to_pdf(&template_html, &context, app_data_dir, target_path)?;
    Ok(RenderResult {
        output_path: target_path.to_string_lossy().to_string(),
    })
}

pub fn render_payment_voucher(
    db: &Db,
    app_data_dir: &Path,
    pv_id: &str,
    template_id: &str,
    target_path: &Path,
) -> AppResult<RenderResult> {
    let pv = payment_voucher::find_by_id(db, pv_id)?;
    let company = resolve_profile(db, pv.business_profile_id.as_deref());
    let template_html = pdf_template::get_renderable(db, template_id)?;

    let (invoice_block, balance_after) = match &pv.invoice_id {
        Some(inv_id) => {
            let iwl = invoice::find_by_id(db, inv_id)?;
            // "Balance after this payment" = invoice.total - sum of all PV amounts up to and
            // including this one. v1 simple approximation: use invoice.paid_amount.
            let balance = iwl.invoice.total - iwl.invoice.paid_amount;
            (
                json!({
                    "number": iwl.invoice.number,
                    "issue_date": iwl.invoice.issue_date,
                    "total": iwl.invoice.total,
                    "paid_amount": iwl.invoice.paid_amount,
                }),
                Some(balance),
            )
        }
        None => (serde_json::Value::Null, None),
    };

    let context = json!({
        "company": build_company_context(&company)?,
        "customer": &pv.customer_snapshot,
        "pv": {
            "number": pv.number,
            "date": pv.date,
            "amount": pv.amount,
            "currency": pv.currency,
            "payment_method": pv.payment_method,
            "notes": pv.notes,
        },
        "invoice": invoice_block,
        "balance_after": balance_after,
    });

    render_to_pdf(&template_html, &context, app_data_dir, target_path)?;
    Ok(RenderResult {
        output_path: target_path.to_string_lossy().to_string(),
    })
}

/// Pure: Tera-render a template HTML against `context`. Shared by both the
/// PDF pipeline (high-fidelity, slow) and the live preview command (fast).
fn render_tera_html(template_html: &str, context: &serde_json::Value) -> AppResult<String> {
    let mut tera = Tera::default();
    filters::register_all(&mut tera);
    let tera_ctx = tera::Context::from_value(context.clone())
        .map_err(|e| AppError::Internal(format!("tera context build: {e}")))?;
    tera.render_str(template_html, &tera_ctx)
        .map_err(|e| AppError::Validation(format!("模板渲染失败：{e}")))
}

/// Tera-render the template against `context`, then drive Chrome to produce a PDF.
fn render_to_pdf(
    template_html: &str,
    context: &serde_json::Value,
    app_data_dir: &Path,
    output_path: &Path,
) -> AppResult<()> {
    let html = render_tera_html(template_html, context)?;

    // Write rendered HTML to a temp file the browser can navigate to via file://
    let tmp_dir = app_data_dir.join(".pdf_tmp");
    file_system::ensure_dir(&tmp_dir)?;
    let tmp_html = tmp_dir.join(format!("{}.html", Uuid::new_v4()));
    file_system::write_file(&tmp_html, &html)?;

    let pdf_bytes = html_to_pdf(&tmp_html);
    // Best-effort cleanup of the temp HTML before propagating any error.
    let _ = file_system::delete_file(&tmp_html);
    let pdf_bytes = pdf_bytes?;

    if let Some(parent) = output_path.parent() {
        file_system::ensure_dir(parent)?;
    }
    file_system::write_bytes(output_path, &pdf_bytes)?;
    Ok(())
}

fn html_to_pdf(html_file: &Path) -> AppResult<Vec<u8>> {
    let browser = launch_browser()?;
    let tab = browser
        .new_tab()
        .map_err(|e| AppError::Internal(format!("new tab: {e}")))?;
    let file_url = file_url_for(html_file);
    tab.navigate_to(&file_url)
        .map_err(|e| AppError::Internal(format!("navigate: {e}")))?;
    tab.wait_until_navigated()
        .map_err(|e| AppError::Internal(format!("wait nav: {e}")))?;

    let opts = PrintToPdfOptions {
        print_background: Some(true),
        prefer_css_page_size: Some(true),
        ..Default::default()
    };
    let pdf = tab
        .print_to_pdf(Some(opts))
        .map_err(|e| AppError::Internal(format!("print to pdf: {e}")))?;
    Ok(pdf)
}

fn launch_browser() -> AppResult<Browser> {
    let path = find_browser_path();
    let mut builder = LaunchOptionsBuilder::default();
    builder.headless(true).sandbox(false);
    if let Some(p) = path {
        builder.path(Some(p));
    }
    let opts = builder
        .build()
        .map_err(|e| AppError::Internal(format!("build launch options: {e}")))?;
    Browser::new(opts).map_err(|e| {
        AppError::Internal(format!(
            "启动浏览器失败：{e}\n请确保系统已装 Chrome 或 Microsoft Edge"
        ))
    })
}

fn find_browser_path() -> Option<PathBuf> {
    // Chrome first (better PDF fidelity historically), then Edge fallback (Win11 default).
    let candidates = [
        r"C:\Program Files\Google\Chrome\Application\chrome.exe",
        r"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe",
        r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe",
        r"C:\Program Files\Microsoft\Edge\Application\msedge.exe",
    ];
    for c in candidates {
        if Path::new(c).exists() {
            return Some(PathBuf::from(c));
        }
    }
    None
}

fn file_url_for(path: &Path) -> String {
    // Windows-style backslashes need to be flipped for file:/// URLs.
    let s = path.to_string_lossy().replace('\\', "/");
    if s.starts_with('/') {
        format!("file://{s}")
    } else {
        format!("file:///{s}")
    }
}

// ============================================================================
// Live HTML previews (no PDF generation). Cheap (~ms), called on every form
// change via Tauri commands.
// ============================================================================

fn line_totals(lines: &[crate::domain::quotation::LineItemInput]) -> Vec<LineForTotals> {
    lines
        .iter()
        .map(|l| LineForTotals {
            quantity: l.quantity,
            unit_price: l.unit_price,
        })
        .collect()
}

fn lines_as_json(
    lines: &[crate::domain::quotation::LineItemInput],
) -> Vec<serde_json::Value> {
    lines
        .iter()
        .enumerate()
        .map(|(i, l)| {
            json!({
                "position": i + 1,
                "description": l.description,
                "quantity": l.quantity,
                "unit_price": l.unit_price,
                "line_total": l.quantity * l.unit_price,
            })
        })
        .collect()
}

pub fn render_quotation_html_preview(
    db: &Db,
    input: &QuotationPreviewInput,
) -> AppResult<String> {
    let cust = crate::domain::customer::find_by_id(db, &input.customer_id)?;
    let company = resolve_profile(db, input.business_profile_id.as_deref());
    let template_html = pdf_template::get_renderable(db, &input.template_id)?;
    let totals = tax_calc::document_totals(
        &line_totals(&input.lines),
        input.tax_enabled,
        input.tax_rate,
    );

    let context = json!({
        "company": build_company_context(&company)?,
        "customer": &cust,
        "quotation": {
            "number": input.number.clone().unwrap_or_else(|| "(预览)".to_string()),
            "issue_date": input.issue_date,
            "valid_until": input.valid_until,
            "currency": input.currency,
            "tax_enabled": input.tax_enabled,
            "tax_rate": input.tax_rate.unwrap_or(0.0),
            "subtotal": totals.subtotal,
            "tax_amount": totals.tax_amount,
            "total": totals.total,
            "notes": input.notes,
            "terms": input.terms,
            "status": input.status.clone().unwrap_or_else(|| "Draft".to_string()),
        },
        "lines": lines_as_json(&input.lines),
    });

    render_tera_html(&template_html, &context)
}

pub fn render_invoice_html_preview(
    db: &Db,
    input: &InvoicePreviewInput,
) -> AppResult<String> {
    let cust = crate::domain::customer::find_by_id(db, &input.customer_id)?;
    let company = resolve_profile(db, input.business_profile_id.as_deref());
    let template_html = pdf_template::get_renderable(db, &input.template_id)?;
    let totals = tax_calc::document_totals(
        &line_totals(&input.lines),
        input.tax_enabled,
        input.tax_rate,
    );

    // Edit mode: fetch saved PVs to show in payment history.
    let (paid_amount, payments) = if let Some(invoice_id) = &input.invoice_id {
        let pvs = crate::domain::payment_voucher::list_by_invoice(db, invoice_id)?;
        let sum: f64 = pvs.iter().map(|p| p.amount).sum();
        (sum, pvs)
    } else {
        (0.0, Vec::new())
    };
    let balance = totals.total - paid_amount;
    let (sel_banks, sel_qrs, sel_statics) = resolve_selected_payments(
        &company,
        &input.selected_bank_account_ids,
        &input.selected_qr_ids,
        &input.selected_static_methods,
    );

    let context = json!({
        "company": build_company_context(&company)?,
        "customer": &cust,
        "invoice": {
            "number": input.number.clone().unwrap_or_else(|| "(预览)".to_string()),
            "issue_date": input.issue_date,
            "due_date": input.due_date,
            "currency": input.currency,
            "tax_enabled": input.tax_enabled,
            "tax_rate": input.tax_rate.unwrap_or(0.0),
            "subtotal": totals.subtotal,
            "tax_amount": totals.tax_amount,
            "total": totals.total,
            "paid_amount": paid_amount,
            "balance": balance,
            "notes": input.notes,
            "terms": input.terms,
            "status": input.status.clone().unwrap_or_else(|| "Draft".to_string()),
        },
        "lines": lines_as_json(&input.lines),
        "payments": &payments,
        "selected_bank_accounts": sel_banks,
        "selected_qrs": sel_qrs,
        "selected_static_methods": sel_statics,
    });

    render_tera_html(&template_html, &context)
}

/// Render a template against fully-hardcoded sample data so the UI can show
/// a real thumbnail. No DB lookups beyond fetching the template HTML itself —
/// stays fast (~5-15 ms) and works for both presets and custom uploads.
pub fn render_template_sample(db: &Db, template_id: &str) -> AppResult<String> {
    let template = pdf_template::find_by_id(db, template_id)?;
    let template_html = pdf_template::get_renderable(db, template_id)?;
    let context = sample_context(template.doc_type);
    render_tera_html(&template_html, &context)
}

fn sample_context(doc_type: crate::domain::pdf_template::PdfDocType) -> serde_json::Value {
    use crate::domain::pdf_template::PdfDocType;
    let company = json!({
        "company_name": "示例公司 SDN BHD",
        "entity_type": "Company",
        "name": "示例公司 SDN BHD",
        "address": "Kuala Lumpur, Malaysia",
        "email": "hello@example.my",
        "phone": "+60 12 345 6789",
        "ssm_no": "SSM-123456",
        "sst_no": "SST-001",
        "logo_data_url": null,
    });
    let customer = json!({
        "name": "示例客户 SDN BHD",
        "address": "Petaling Jaya",
        "contact_person": "Tan Ah Beng",
        "email": "client@example.my",
        "phone": "+60 13 555 1234",
        "ssm_no": null, "nric": null, "tax_no": null,
    });
    let lines = json!([
        { "position": 1, "description": "示例项目 A", "quantity": 2.0, "unit_price": 150.00, "line_total": 300.00 },
        { "position": 2, "description": "示例项目 B", "quantity": 1.0, "unit_price": 450.00, "line_total": 450.00 },
    ]);
    match doc_type {
        PdfDocType::Quotation => json!({
            "company": company, "customer": customer, "lines": lines,
            "quotation": {
                "number": "QUO-2026-001", "issue_date": "2026-01-15", "valid_until": "2026-02-14",
                "currency": "MYR", "tax_enabled": true, "tax_rate": 0.06,
                "subtotal": 750.0, "tax_amount": 45.0, "total": 795.0,
                "notes": "", "terms": "", "status": "Draft",
            },
        }),
        PdfDocType::Invoice => json!({
            "company": company, "customer": customer, "lines": lines,
            "invoice": {
                "number": "INV-2026-001", "issue_date": "2026-01-15", "due_date": "2026-02-14",
                "currency": "MYR", "tax_enabled": true, "tax_rate": 0.06,
                "subtotal": 750.0, "tax_amount": 45.0, "total": 795.0,
                "paid_amount": 0.0, "balance": 795.0,
                "notes": "", "terms": "", "status": "Draft",
            },
            "payments": [],
            "selected_bank_accounts": [],
            "selected_qrs": [],
            "selected_static_methods": ["FPX", "DuitNow"],
        }),
        PdfDocType::PaymentVoucher => json!({
            "company": company, "customer": customer,
            "pv": {
                "number": "PV-2026-001", "date": "2026-01-15",
                "amount": 795.0, "currency": "MYR",
                "payment_method": "Bank Transfer", "notes": "",
            },
            "invoice": {
                "number": "INV-2026-001", "issue_date": "2026-01-15",
                "total": 795.0, "paid_amount": 795.0,
            },
            "balance_after": 0.0,
        }),
    }
}

pub fn render_payment_voucher_html_preview(
    db: &Db,
    input: &PaymentVoucherPreviewInput,
) -> AppResult<String> {
    let company = resolve_profile(db, input.business_profile_id.as_deref());
    let template_html = pdf_template::get_renderable(db, &input.template_id)?;

    let (customer_snapshot, currency, invoice_block, balance_after) = match &input.invoice_id {
        Some(inv_id) => {
            let iwl = invoice::find_by_id(db, inv_id)?;
            // Balance after this preview PV: invoice.paid_amount already reflects
            // saved PVs. If editing (number provided), subtract this PV's prior
            // saved amount before adding the new one. Otherwise just use
            // invoice.paid_amount + this_amount.
            let prior_paid_excluding_this = if let Some(num) = &input.number {
                let saved = crate::domain::payment_voucher::list_by_invoice(db, inv_id)?;
                let mut sum = 0.0;
                for pv in saved {
                    if &pv.number != num {
                        sum += pv.amount;
                    }
                }
                sum
            } else {
                iwl.invoice.paid_amount
            };
            let balance = iwl.invoice.total - prior_paid_excluding_this - input.amount;
            (
                iwl.invoice.customer_snapshot.clone(),
                iwl.invoice.currency.clone(),
                json!({
                    "number": iwl.invoice.number,
                    "issue_date": iwl.invoice.issue_date,
                    "total": iwl.invoice.total,
                    "paid_amount": iwl.invoice.paid_amount,
                }),
                Some(balance),
            )
        }
        None => {
            let cust_id = input.customer_id.as_deref().ok_or_else(|| {
                AppError::Validation("独立 PV 预览需要 customer_id".into())
            })?;
            let curr = input.currency.clone().unwrap_or_else(|| "MYR".into());
            let cust = crate::domain::customer::find_by_id(db, cust_id)?;
            let snap = serde_json::to_value(&cust)
                .map_err(|e| AppError::Internal(format!("snapshot customer: {e}")))?;
            (snap, curr, serde_json::Value::Null, None)
        }
    };

    let context = json!({
        "company": build_company_context(&company)?,
        "customer": customer_snapshot,
        "pv": {
            "number": input.number.clone().unwrap_or_else(|| "(预览)".to_string()),
            "date": input.date,
            "amount": input.amount,
            "currency": currency,
            "payment_method": input.payment_method,
            "notes": input.notes,
        },
        "invoice": invoice_block,
        "balance_after": balance_after,
    });

    render_tera_html(&template_html, &context)
}
