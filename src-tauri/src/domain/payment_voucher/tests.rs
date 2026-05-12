use tempfile::tempdir;

use crate::domain::{customer, invoice, quotation};
use crate::error::AppError;
use crate::infra::Db;

use super::service;
use super::types::{CreatePaymentVoucherInput, UpdatePaymentVoucherInput};
use quotation::LineItemInput;

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

fn make_invoice(db: &Db, customer_id: String, total: f64) -> String {
    invoice::create(
        db,
        invoice::CreateInvoiceInput {
            customer_id,
            business_profile_id: None,
            issue_date: "2026-05-12".into(),
            due_date: "2026-06-11".into(),
            currency: "MYR".into(),
            tax_enabled: false,
            tax_rate: None,
            lines: vec![LineItemInput {
                description: "service".into(),
                quantity: 1.0,
                unit_price: total,
            }],
            notes: None,
            terms: None,
            selected_bank_account_ids: vec![],
            selected_qr_ids: vec![],
            selected_static_methods: vec![],
        },
    )
    .unwrap()
    .invoice
    .id
}

fn make_sent_invoice(db: &Db, customer_id: String, total: f64) -> String {
    let id = make_invoice(db, customer_id, total);
    invoice::mark_sent(db, &id).unwrap();
    id
}

fn pv_input(invoice_id: &str, amount: f64) -> CreatePaymentVoucherInput {
    CreatePaymentVoucherInput {
        invoice_id: Some(invoice_id.into()),
        customer_id: None,
        currency: None,
        business_profile_id: None,
        date: "2026-05-13".into(),
        amount,
        payment_method: "FPX".into(),
        notes: None,
    }
}

fn standalone_input(customer_id: &str, amount: f64) -> CreatePaymentVoucherInput {
    CreatePaymentVoucherInput {
        invoice_id: None,
        customer_id: Some(customer_id.into()),
        currency: Some("MYR".into()),
        business_profile_id: None,
        date: "2026-05-13".into(),
        amount,
        payment_method: "Cash".into(),
        notes: None,
    }
}

#[test]
fn partial_payment_auto_marks_partial() {
    let (_d, db) = fresh_db();
    let cid = seed_customer(&db);
    let iid = make_sent_invoice(&db, cid, 1000.0);

    let pv = service::create(&db, pv_input(&iid, 400.0)).unwrap();
    assert!(pv.number.starts_with("PV-"));

    let inv = invoice::find_by_id(&db, &iid).unwrap().invoice;
    assert_eq!(inv.paid_amount, 400.0);
    assert_eq!(inv.status, invoice::InvoiceStatus::PartialPaid);
}

#[test]
fn full_payment_auto_marks_paid() {
    let (_d, db) = fresh_db();
    let cid = seed_customer(&db);
    let iid = make_sent_invoice(&db, cid, 1000.0);

    service::create(&db, pv_input(&iid, 1000.0)).unwrap();

    let inv = invoice::find_by_id(&db, &iid).unwrap().invoice;
    assert_eq!(inv.paid_amount, 1000.0);
    assert_eq!(inv.status, invoice::InvoiceStatus::Paid);
}

#[test]
fn overpayment_still_marks_paid() {
    let (_d, db) = fresh_db();
    let cid = seed_customer(&db);
    let iid = make_sent_invoice(&db, cid, 1000.0);

    service::create(&db, pv_input(&iid, 1500.0)).unwrap();

    let inv = invoice::find_by_id(&db, &iid).unwrap().invoice;
    assert_eq!(inv.paid_amount, 1500.0);
    assert_eq!(inv.status, invoice::InvoiceStatus::Paid);
}

#[test]
fn multiple_pvs_sum_correctly() {
    let (_d, db) = fresh_db();
    let cid = seed_customer(&db);
    let iid = make_sent_invoice(&db, cid, 1000.0);

    service::create(&db, pv_input(&iid, 300.0)).unwrap();
    service::create(&db, pv_input(&iid, 200.0)).unwrap();
    service::create(&db, pv_input(&iid, 500.0)).unwrap();

    let inv = invoice::find_by_id(&db, &iid).unwrap().invoice;
    assert_eq!(inv.paid_amount, 1000.0);
    assert_eq!(inv.status, invoice::InvoiceStatus::Paid);

    let pvs = service::list_by_invoice(&db, &iid).unwrap();
    assert_eq!(pvs.len(), 3);
    assert_eq!(service::sum_by_invoice(&db, &iid).unwrap(), 1000.0);
}

#[test]
fn update_pv_recalcs_invoice() {
    let (_d, db) = fresh_db();
    let cid = seed_customer(&db);
    let iid = make_sent_invoice(&db, cid, 1000.0);

    let pv = service::create(&db, pv_input(&iid, 400.0)).unwrap();
    let inv = invoice::find_by_id(&db, &iid).unwrap().invoice;
    assert_eq!(inv.status, invoice::InvoiceStatus::PartialPaid);

    // Bump payment to full
    service::update(
        &db,
        UpdatePaymentVoucherInput {
            id: pv.id,
            date: pv.date,
            amount: 1000.0,
            payment_method: pv.payment_method,
            notes: None,
        },
    )
    .unwrap();

    let inv = invoice::find_by_id(&db, &iid).unwrap().invoice;
    assert_eq!(inv.paid_amount, 1000.0);
    assert_eq!(inv.status, invoice::InvoiceStatus::Paid);
}

#[test]
fn delete_pv_drops_paid_amount_but_does_not_downgrade_status() {
    let (_d, db) = fresh_db();
    let cid = seed_customer(&db);
    let iid = make_sent_invoice(&db, cid, 1000.0);

    let pv = service::create(&db, pv_input(&iid, 400.0)).unwrap();
    let inv = invoice::find_by_id(&db, &iid).unwrap().invoice;
    assert_eq!(inv.status, invoice::InvoiceStatus::PartialPaid);

    service::delete(&db, &pv.id).unwrap();
    let inv = invoice::find_by_id(&db, &iid).unwrap().invoice;
    assert_eq!(inv.paid_amount, 0.0);
    // Status stays PartialPaid because PartialPaid → Sent is not in the state machine.
    assert_eq!(inv.status, invoice::InvoiceStatus::PartialPaid);
}

#[test]
fn create_pv_rejects_draft_invoice() {
    let (_d, db) = fresh_db();
    let cid = seed_customer(&db);
    let iid = make_invoice(&db, cid, 1000.0); // Draft

    let err = service::create(&db, pv_input(&iid, 100.0)).unwrap_err();
    assert!(matches!(err, AppError::Validation(_)));
}

#[test]
fn create_pv_rejects_void_invoice() {
    let (_d, db) = fresh_db();
    let cid = seed_customer(&db);
    let iid = make_sent_invoice(&db, cid, 1000.0);
    invoice::mark_void(&db, &iid).unwrap();

    let err = service::create(&db, pv_input(&iid, 100.0)).unwrap_err();
    assert!(matches!(err, AppError::Validation(_)));
}

#[test]
fn create_pv_rejects_zero_or_negative_amount() {
    let (_d, db) = fresh_db();
    let cid = seed_customer(&db);
    let iid = make_sent_invoice(&db, cid, 1000.0);

    let err = service::create(&db, pv_input(&iid, 0.0)).unwrap_err();
    assert!(matches!(err, AppError::Validation(_)));
    let err = service::create(&db, pv_input(&iid, -100.0)).unwrap_err();
    assert!(matches!(err, AppError::Validation(_)));
}

#[test]
fn create_pv_rejects_bad_date() {
    let (_d, db) = fresh_db();
    let cid = seed_customer(&db);
    let iid = make_sent_invoice(&db, cid, 1000.0);

    let mut p = pv_input(&iid, 100.0);
    p.date = "13-05-2026".into();
    let err = service::create(&db, p).unwrap_err();
    assert!(matches!(err, AppError::Validation(_)));
}

#[test]
fn create_pv_rejects_empty_payment_method() {
    let (_d, db) = fresh_db();
    let cid = seed_customer(&db);
    let iid = make_sent_invoice(&db, cid, 1000.0);

    let mut p = pv_input(&iid, 100.0);
    p.payment_method = "   ".into();
    let err = service::create(&db, p).unwrap_err();
    assert!(matches!(err, AppError::Validation(_)));
}

#[test]
fn pv_inherits_invoice_currency_and_customer_snapshot() {
    let (_d, db) = fresh_db();
    let cid = seed_customer(&db);
    let iid = make_sent_invoice(&db, cid.clone(), 1000.0);

    let pv = service::create(&db, pv_input(&iid, 500.0)).unwrap();
    assert_eq!(pv.currency, "MYR");
    assert_eq!(pv.customer_id, cid);
    assert_eq!(pv.customer_snapshot["name"], "Acme");
}

#[test]
fn pv_numbering_sequence() {
    let (_d, db) = fresh_db();
    let cid = seed_customer(&db);
    let iid = make_sent_invoice(&db, cid, 1000.0);

    let a = service::create(&db, pv_input(&iid, 100.0)).unwrap();
    let b = service::create(&db, pv_input(&iid, 100.0)).unwrap();
    let a_seq = a.number.rsplit('-').next().unwrap().parse::<i32>().unwrap();
    let b_seq = b.number.rsplit('-').next().unwrap().parse::<i32>().unwrap();
    assert_eq!(a_seq + 1, b_seq);
}

#[test]
fn pv_on_overdue_invoice_auto_marks_partial_or_paid() {
    let (_d, db) = fresh_db();
    let cid = seed_customer(&db);
    let iid = make_sent_invoice(&db, cid, 1000.0);
    invoice::mark_overdue(&db, &iid).unwrap();

    // partial payment on Overdue → PartialPaid
    let pv = service::create(&db, pv_input(&iid, 400.0)).unwrap();
    let inv = invoice::find_by_id(&db, &iid).unwrap().invoice;
    assert_eq!(inv.status, invoice::InvoiceStatus::PartialPaid);

    // top up to full
    service::update(
        &db,
        UpdatePaymentVoucherInput {
            id: pv.id,
            date: pv.date,
            amount: 1000.0,
            payment_method: pv.payment_method,
            notes: None,
        },
    )
    .unwrap();
    let inv = invoice::find_by_id(&db, &iid).unwrap().invoice;
    assert_eq!(inv.status, invoice::InvoiceStatus::Paid);
}

#[test]
fn recalc_paid_amount_works_after_external_change() {
    let (_d, db) = fresh_db();
    let cid = seed_customer(&db);
    let iid = make_sent_invoice(&db, cid, 1000.0);

    service::create(&db, pv_input(&iid, 300.0)).unwrap();
    service::create(&db, pv_input(&iid, 200.0)).unwrap();

    // Force a recalc explicitly (should be a no-op since auto-recalc already ran).
    let sum = invoice::recalc_paid_amount(&db, &iid).unwrap();
    assert_eq!(sum, 500.0);
    let inv = invoice::find_by_id(&db, &iid).unwrap().invoice;
    assert_eq!(inv.paid_amount, 500.0);
    assert_eq!(inv.status, invoice::InvoiceStatus::PartialPaid);
}

#[test]
fn update_pv_rejects_when_invoice_is_void() {
    let (_d, db) = fresh_db();
    let cid = seed_customer(&db);
    let iid = make_sent_invoice(&db, cid, 1000.0);
    let pv = service::create(&db, pv_input(&iid, 400.0)).unwrap();
    invoice::mark_void(&db, &iid).unwrap();

    let err = service::update(
        &db,
        UpdatePaymentVoucherInput {
            id: pv.id,
            date: pv.date,
            amount: 500.0,
            payment_method: pv.payment_method,
            notes: None,
        },
    )
    .unwrap_err();
    assert!(matches!(err, AppError::Validation(_)));
}

#[test]
fn delete_pv_rejects_when_invoice_is_void() {
    let (_d, db) = fresh_db();
    let cid = seed_customer(&db);
    let iid = make_sent_invoice(&db, cid, 1000.0);
    let pv = service::create(&db, pv_input(&iid, 400.0)).unwrap();
    invoice::mark_void(&db, &iid).unwrap();

    let err = service::delete(&db, &pv.id).unwrap_err();
    assert!(matches!(err, AppError::Validation(_)));
}

#[test]
fn standalone_pv_creates_without_invoice() {
    let (_d, db) = fresh_db();
    let cid = seed_customer(&db);

    let pv = service::create(&db, standalone_input(&cid, 250.0)).unwrap();
    assert!(pv.invoice_id.is_none());
    assert_eq!(pv.customer_id, cid);
    assert_eq!(pv.currency, "MYR");
    assert_eq!(pv.amount, 250.0);
    assert_eq!(pv.customer_snapshot["name"], "Acme");
}

#[test]
fn standalone_pv_requires_customer_id() {
    let (_d, db) = fresh_db();
    let mut input = standalone_input("doesnt-matter", 100.0);
    input.customer_id = None;
    let err = service::create(&db, input).unwrap_err();
    assert!(matches!(err, AppError::Validation(_)));
}

#[test]
fn standalone_pv_requires_currency() {
    let (_d, db) = fresh_db();
    let cid = seed_customer(&db);
    let mut input = standalone_input(&cid, 100.0);
    input.currency = None;
    let err = service::create(&db, input).unwrap_err();
    assert!(matches!(err, AppError::Validation(_)));
}

#[test]
fn standalone_pv_does_not_affect_any_invoice() {
    let (_d, db) = fresh_db();
    let cid = seed_customer(&db);
    let iid = make_sent_invoice(&db, cid.clone(), 1000.0);

    // Independent PV on the same customer
    service::create(&db, standalone_input(&cid, 500.0)).unwrap();

    let inv = invoice::find_by_id(&db, &iid).unwrap().invoice;
    assert_eq!(inv.paid_amount, 0.0);
    assert_eq!(inv.status, invoice::InvoiceStatus::Sent);
}

#[test]
fn delete_standalone_pv_does_not_touch_invoice_layer() {
    let (_d, db) = fresh_db();
    let cid = seed_customer(&db);

    let pv = service::create(&db, standalone_input(&cid, 100.0)).unwrap();
    service::delete(&db, &pv.id).unwrap();
    // Just make sure no panic / error and the PV is gone.
    assert_eq!(service::list(&db).unwrap().len(), 0);
}

#[test]
fn list_by_customer_and_invoice() {
    let (_d, db) = fresh_db();
    let cid = seed_customer(&db);
    let i1 = make_sent_invoice(&db, cid.clone(), 1000.0);
    let i2 = make_sent_invoice(&db, cid.clone(), 2000.0);

    service::create(&db, pv_input(&i1, 100.0)).unwrap();
    service::create(&db, pv_input(&i1, 200.0)).unwrap();
    service::create(&db, pv_input(&i2, 500.0)).unwrap();

    assert_eq!(service::list_by_invoice(&db, &i1).unwrap().len(), 2);
    assert_eq!(service::list_by_invoice(&db, &i2).unwrap().len(), 1);
    assert_eq!(service::list_by_customer(&db, &cid).unwrap().len(), 3);
    assert_eq!(service::list(&db).unwrap().len(), 3);
}
