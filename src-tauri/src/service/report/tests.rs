use tempfile::tempdir;

use crate::domain::customer::{self, CreateCustomerInput, CustomerType};
use crate::domain::invoice::{self, CreateInvoiceInput};
use crate::domain::payment_voucher::{self, CreatePaymentVoucherInput};
use crate::domain::quotation::LineItemInput;
use crate::infra::Db;

use super::service;

fn fresh_db() -> (tempfile::TempDir, Db) {
    let dir = tempdir().unwrap();
    let db = Db::open(dir.path().join("test.db")).unwrap();
    db.run_migrations().unwrap();
    (dir, db)
}

fn make_customer(db: &Db) -> String {
    customer::create(
        db,
        CreateCustomerInput {
            type_: CustomerType::Company,
            name: "Acme".into(),
            contact_person: None,
            email: None,
            phone: None,
            address: None,
            ssm_no: Some("SSM-1".into()),
            nric: None,
            tax_no: None,
            notes: None,
        },
    )
    .unwrap()
    .id
}

fn make_invoice(db: &Db, customer_id: &str, total: f64, due_date: &str) -> String {
    let iwl = invoice::create(
        db,
        CreateInvoiceInput {
            customer_id: customer_id.into(),
            business_profile_id: None,
            selected_bank_account_ids: vec![],
            selected_qr_ids: vec![],
            selected_static_methods: vec![],
            issue_date: "2026-01-01".into(),
            due_date: due_date.into(),
            currency: "MYR".into(),
            tax_enabled: false,
            tax_rate: None,
            lines: vec![LineItemInput {
                description: "x".into(),
                quantity: 1.0,
                unit_price: total,
            }],
            notes: None,
            terms: None,
        },
    )
    .unwrap();
    invoice::mark_sent(db, &iwl.invoice.id).unwrap();
    iwl.invoice.id
}

fn add_pv(db: &Db, invoice_id: &str, date: &str, amount: f64) {
    payment_voucher::create(
        db,
        CreatePaymentVoucherInput {
            invoice_id: Some(invoice_id.into()),
            customer_id: None,
            currency: None,
            business_profile_id: None,
            date: date.into(),
            amount,
            payment_method: "Cash".into(),
            notes: None,
        },
    )
    .unwrap();
}

#[test]
fn empty_db_returns_empty_report() {
    let (_d, db) = fresh_db();
    let r = service::yearly_revenue(&db, 2026).unwrap();
    assert_eq!(r.months.len(), 12);
    assert_eq!(r.total_revenue.len(), 0);
    let o = service::outstanding_invoices(&db).unwrap();
    assert_eq!(o.invoices.len(), 0);
}

#[test]
fn monthly_revenue_sums_pvs_in_month_only() {
    let (_d, db) = fresh_db();
    let cid = make_customer(&db);
    let inv = make_invoice(&db, &cid, 1000.0, "2026-02-01");
    add_pv(&db, &inv, "2026-01-15", 300.0); // in
    add_pv(&db, &inv, "2026-01-20", 200.0); // in
    add_pv(&db, &inv, "2026-02-05", 500.0); // out (different month)

    let row = service::monthly_revenue(&db, 2026, 1).unwrap();
    assert_eq!(row.pv_count, 2);
    assert_eq!(row.revenue.len(), 1);
    assert_eq!(row.revenue[0].currency, "MYR");
    assert!((row.revenue[0].amount - 500.0).abs() < 1e-6);
}

#[test]
fn yearly_total_matches_sum_of_months() {
    let (_d, db) = fresh_db();
    let cid = make_customer(&db);
    let inv = make_invoice(&db, &cid, 5000.0, "2026-12-31");
    add_pv(&db, &inv, "2026-03-01", 1000.0);
    add_pv(&db, &inv, "2026-07-15", 2000.0);

    let r = service::yearly_revenue(&db, 2026).unwrap();
    assert_eq!(r.total_revenue.len(), 1);
    assert!((r.total_revenue[0].amount - 3000.0).abs() < 1e-6);
}

#[test]
fn outstanding_excludes_void_paid_draft() {
    let (_d, db) = fresh_db();
    let cid = make_customer(&db);
    let i1 = make_invoice(&db, &cid, 100.0, "2099-01-01"); // Sent, unpaid → in
    let i2 = make_invoice(&db, &cid, 200.0, "2099-01-01");
    add_pv(&db, &i2, "2026-01-01", 200.0); // fully paid → out
    let i3 = make_invoice(&db, &cid, 300.0, "2099-01-01");
    invoice::mark_void(&db, &i3).unwrap(); // void → out

    let report = service::outstanding_invoices(&db).unwrap();
    let ids: Vec<_> = report.invoices.iter().map(|r| r.invoice.id.clone()).collect();
    assert!(ids.contains(&i1));
    assert!(!ids.contains(&i2));
    assert!(!ids.contains(&i3));
    assert_eq!(report.total_outstanding.len(), 1);
    assert!((report.total_outstanding[0].amount - 100.0).abs() < 1e-6);
}

#[test]
fn outstanding_sorted_most_overdue_first() {
    let (_d, db) = fresh_db();
    let cid = make_customer(&db);
    // Use past due dates so days_overdue > 0 today.
    let a = make_invoice(&db, &cid, 100.0, "2024-01-01"); // very overdue
    let b = make_invoice(&db, &cid, 100.0, "2025-06-01"); // less overdue
    let report = service::outstanding_invoices(&db).unwrap();
    assert_eq!(report.invoices.len(), 2);
    assert_eq!(report.invoices[0].invoice.id, a);
    assert_eq!(report.invoices[1].invoice.id, b);
    assert!(report.invoices[0].days_overdue > report.invoices[1].days_overdue);
}
