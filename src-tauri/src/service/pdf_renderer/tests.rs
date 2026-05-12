//! Unit tests for the PDF renderer module. Integration tests that actually
//! launch headless Chrome are marked `#[ignore]` and run via:
//!   cargo test --manifest-path src-tauri/Cargo.toml -- --ignored
//!
//! They require Chrome or Edge to be installed and may take several seconds.
use tempfile::tempdir;

use crate::domain::{customer, invoice, payment_voucher, quotation};
use crate::infra::Db;

fn fresh_db() -> (tempfile::TempDir, Db) {
    let dir = tempdir().unwrap();
    let db = Db::open(dir.path().join("test.db")).unwrap();
    db.run_migrations().unwrap();
    (dir, db)
}

fn seed_customer(db: &Db) -> String {
    customer::create(
        db,
        customer::CreateCustomerInput {
            type_: customer::CustomerType::Company,
            name: "Acme Studios".into(),
            contact_person: Some("Jane Doe".into()),
            email: Some("hello@acme.my".into()),
            phone: Some("+60123456789".into()),
            address: Some("12 Jalan Bukit Bintang\nKuala Lumpur 55100".into()),
            ssm_no: Some("SSM-12345".into()),
            nric: None,
            tax_no: None,
            notes: None,
        },
    )
    .unwrap()
    .id
}

#[test]
#[ignore = "requires Chrome/Edge installed; run with `cargo test -- --ignored`"]
fn render_quotation_writes_pdf_to_disk() {
    let (_d, db) = fresh_db();
    let cid = seed_customer(&db);
    let q = quotation::create(
        &db,
        quotation::CreateQuotationInput {
            customer_id: cid,
            business_profile_id: None,
            issue_date: "2026-05-12".into(),
            valid_until: "2026-06-11".into(),
            currency: "MYR".into(),
            tax_enabled: true,
            tax_rate: Some(0.06),
            lines: vec![
                quotation::LineItemInput {
                    description: "Wedding video editing".into(),
                    quantity: 1.0,
                    unit_price: 3000.0,
                },
                quotation::LineItemInput {
                    description: "Drone footage\n(extra session)".into(),
                    quantity: 2.0,
                    unit_price: 500.0,
                },
            ],
            notes: Some("Delivery within 14 days.".into()),
            terms: Some("50% deposit on acceptance.".into()),
        },
    )
    .unwrap();

    let dir = tempdir().unwrap();
    let target = dir.path().join("out-quotation.pdf");
    let result = super::renderer::render_quotation(
        &db,
        dir.path(),
        &q.quotation.id,
        "preset-quotation-default",
        &target,
    )
    .unwrap();

    let path = std::path::Path::new(&result.output_path);
    assert!(path.exists(), "PDF not written to {result:?}");
    let bytes = std::fs::read(path).unwrap();
    // PDFs start with `%PDF-`
    assert!(bytes.starts_with(b"%PDF-"), "not a PDF: {result:?}");
}

#[test]
#[ignore = "requires Chrome/Edge installed; run with `cargo test -- --ignored`"]
fn render_invoice_writes_pdf_to_disk() {
    let (_d, db) = fresh_db();
    let cid = seed_customer(&db);
    let inv = invoice::create(
        &db,
        invoice::CreateInvoiceInput {
            customer_id: cid,
            business_profile_id: None,
            issue_date: "2026-05-12".into(),
            due_date: "2026-06-11".into(),
            currency: "MYR".into(),
            tax_enabled: true,
            tax_rate: Some(0.06),
            lines: vec![quotation::LineItemInput {
                description: "Editing".into(),
                quantity: 1.0,
                unit_price: 2000.0,
            }],
            notes: None,
            terms: None,
            selected_bank_account_ids: vec![],
            selected_qr_ids: vec![],
            selected_static_methods: vec![],
        },
    )
    .unwrap();
    invoice::mark_sent(&db, &inv.invoice.id).unwrap();
    payment_voucher::create(
        &db,
        payment_voucher::CreatePaymentVoucherInput {
            invoice_id: Some(inv.invoice.id.clone()),
            customer_id: None,
            currency: None,
            business_profile_id: None,
            date: "2026-05-13".into(),
            amount: 500.0,
            payment_method: "FPX".into(),
            notes: None,
        },
    )
    .unwrap();

    let dir = tempdir().unwrap();
    let target = dir.path().join("out-invoice.pdf");
    let result = super::renderer::render_invoice(
        &db,
        dir.path(),
        &inv.invoice.id,
        "preset-invoice-default",
        &target,
    )
    .unwrap();
    let path = std::path::Path::new(&result.output_path);
    assert!(path.exists());
    let bytes = std::fs::read(path).unwrap();
    assert!(bytes.starts_with(b"%PDF-"));
}

#[test]
#[ignore = "requires Chrome/Edge installed; run with `cargo test -- --ignored`"]
fn render_payment_voucher_writes_pdf_to_disk() {
    let (_d, db) = fresh_db();
    let cid = seed_customer(&db);
    let inv = invoice::create(
        &db,
        invoice::CreateInvoiceInput {
            customer_id: cid,
            business_profile_id: None,
            issue_date: "2026-05-12".into(),
            due_date: "2026-06-11".into(),
            currency: "MYR".into(),
            tax_enabled: false,
            tax_rate: None,
            lines: vec![quotation::LineItemInput {
                description: "Service".into(),
                quantity: 1.0,
                unit_price: 1000.0,
            }],
            notes: None,
            terms: None,
            selected_bank_account_ids: vec![],
            selected_qr_ids: vec![],
            selected_static_methods: vec![],
        },
    )
    .unwrap();
    invoice::mark_sent(&db, &inv.invoice.id).unwrap();
    let pv = payment_voucher::create(
        &db,
        payment_voucher::CreatePaymentVoucherInput {
            invoice_id: Some(inv.invoice.id),
            customer_id: None,
            currency: None,
            business_profile_id: None,
            date: "2026-05-13".into(),
            amount: 400.0,
            payment_method: "Bank Transfer".into(),
            notes: Some("Maybank ref 8888".into()),
        },
    )
    .unwrap();

    let dir = tempdir().unwrap();
    let target = dir.path().join("out-pv.pdf");
    let result = super::renderer::render_payment_voucher(
        &db,
        dir.path(),
        &pv.id,
        "preset-pv-default",
        &target,
    )
    .unwrap();
    let path = std::path::Path::new(&result.output_path);
    assert!(path.exists());
    let bytes = std::fs::read(path).unwrap();
    assert!(bytes.starts_with(b"%PDF-"));
}
