use tempfile::tempdir;

use crate::domain::customer;
use crate::domain::quotation;
use crate::error::AppError;
use crate::infra::Db;

use super::service;
use super::state_machine::{can_transition, InvoiceStatus};
use super::types::{
    CreateFromQuotationInput, CreateInvoiceInput, UpdateInvoiceInput,
};
use quotation::LineItemInput;

fn fresh_db() -> (tempfile::TempDir, Db) {
    let dir = tempdir().unwrap();
    let db = Db::open(dir.path().join("test.db")).unwrap();
    db.run_migrations().unwrap();
    (dir, db)
}

fn seed_customer(db: &Db, name: &str) -> String {
    customer::create(
        db,
        customer::CreateCustomerInput {
            type_: customer::CustomerType::Company,
            name: name.into(),
            contact_person: None,
            email: None,
            phone: None,
            address: None,
            ssm_no: Some("SSM-X".into()),
            nric: None,
            tax_no: None,
            notes: None,
        },
    )
    .unwrap()
    .id
}

fn sample_create(customer_id: String) -> CreateInvoiceInput {
    CreateInvoiceInput {
        customer_id,
        business_profile_id: None,
        issue_date: "2026-05-12".into(),
        due_date: "2026-06-11".into(),
        currency: "MYR".into(),
        tax_enabled: true,
        tax_rate: Some(0.06),
        lines: vec![
            LineItemInput {
                description: "Editing".into(),
                quantity: 1.0,
                unit_price: 3000.0,
            },
            LineItemInput {
                description: "Color".into(),
                quantity: 1.0,
                unit_price: 1000.0,
            },
        ],
        notes: None,
        terms: None,
            selected_bank_account_ids: vec![],
            selected_qr_ids: vec![],
            selected_static_methods: vec![],
    }
}

fn make_accepted_quotation(db: &Db, customer_id: String) -> String {
    let q = quotation::create(
        db,
        quotation::CreateQuotationInput {
            customer_id,
            business_profile_id: None,
            issue_date: "2026-05-01".into(),
            valid_until: "2026-05-31".into(),
            currency: "MYR".into(),
            tax_enabled: true,
            tax_rate: Some(0.06),
            lines: vec![LineItemInput {
                description: "Wedding video".into(),
                quantity: 1.0,
                unit_price: 5000.0,
            }],
            notes: Some("preliminary".into()),
            terms: None,
        },
    )
    .unwrap();
    quotation::mark_sent(db, &q.quotation.id).unwrap();
    quotation::mark_accepted(db, &q.quotation.id).unwrap();
    q.quotation.id
}

#[test]
fn create_assigns_number_and_computes_totals() {
    let (_d, db) = fresh_db();
    let cid = seed_customer(&db, "Acme");
    let r = service::create(&db, sample_create(cid)).unwrap();
    assert!(r.invoice.number.starts_with("INV-"));
    assert_eq!(r.invoice.status, InvoiceStatus::Draft);
    assert_eq!(r.invoice.paid_amount, 0.0);
    assert!((r.invoice.subtotal - 4000.0).abs() < 1e-6);
    assert!((r.invoice.tax_amount - 240.0).abs() < 1e-6);
    assert!((r.invoice.total - 4240.0).abs() < 1e-6);
    assert_eq!(r.lines.len(), 2);
}

#[test]
fn invoice_and_quotation_numbering_independent() {
    let (_d, db) = fresh_db();
    let cid = seed_customer(&db, "Acme");
    let q = quotation::create(
        &db,
        quotation::CreateQuotationInput {
            customer_id: cid.clone(),
            business_profile_id: None,
            issue_date: "2026-05-01".into(),
            valid_until: "2026-05-31".into(),
            currency: "MYR".into(),
            tax_enabled: false,
            tax_rate: None,
            lines: vec![LineItemInput {
                description: "x".into(),
                quantity: 1.0,
                unit_price: 1.0,
            }],
            notes: None,
            terms: None,
        },
    )
    .unwrap();
    let inv = service::create(&db, sample_create(cid)).unwrap();
    assert!(q.quotation.number.starts_with("QUO-"));
    assert!(inv.invoice.number.starts_with("INV-"));
}

#[test]
fn create_from_quotation_copies_lines_and_links() {
    let (_d, db) = fresh_db();
    let cid = seed_customer(&db, "Acme");
    let qid = make_accepted_quotation(&db, cid);

    let r = service::create_from_quotation(
        &db,
        CreateFromQuotationInput {
            quotation_id: qid.clone(),
            business_profile_id: None,
            issue_date: "2026-05-12".into(),
            due_date: "2026-06-11".into(),
        },
    )
    .unwrap();

    assert_eq!(r.invoice.source_quotation_id.as_deref(), Some(qid.as_str()));
    assert_eq!(r.lines.len(), 1);
    assert!((r.invoice.subtotal - 5000.0).abs() < 1e-6);
    assert!((r.invoice.tax_amount - 300.0).abs() < 1e-6);
    assert_eq!(r.invoice.notes.as_deref(), Some("preliminary"));

    // quotation.converted_invoice_id is now set
    let q_after = quotation::find_by_id(&db, &qid).unwrap();
    assert_eq!(
        q_after.quotation.converted_invoice_id.as_deref(),
        Some(r.invoice.id.as_str())
    );
}

#[test]
fn create_from_quotation_rejects_non_accepted() {
    let (_d, db) = fresh_db();
    let cid = seed_customer(&db, "Acme");
    let q = quotation::create(
        &db,
        quotation::CreateQuotationInput {
            customer_id: cid,
            business_profile_id: None,
            issue_date: "2026-05-01".into(),
            valid_until: "2026-05-31".into(),
            currency: "MYR".into(),
            tax_enabled: false,
            tax_rate: None,
            lines: vec![LineItemInput {
                description: "x".into(),
                quantity: 1.0,
                unit_price: 100.0,
            }],
            notes: None,
            terms: None,
        },
    )
    .unwrap();
    // Still Draft
    let err = service::create_from_quotation(
        &db,
        CreateFromQuotationInput {
            quotation_id: q.quotation.id,
            business_profile_id: None,
            issue_date: "2026-05-12".into(),
            due_date: "2026-06-11".into(),
        },
    )
    .unwrap_err();
    assert!(matches!(err, AppError::Validation(_)));
}

#[test]
fn create_from_quotation_rejects_double_conversion() {
    let (_d, db) = fresh_db();
    let cid = seed_customer(&db, "Acme");
    let qid = make_accepted_quotation(&db, cid);

    service::create_from_quotation(
        &db,
        CreateFromQuotationInput {
            quotation_id: qid.clone(),
            business_profile_id: None,
            issue_date: "2026-05-12".into(),
            due_date: "2026-06-11".into(),
        },
    )
    .unwrap();
    let err = service::create_from_quotation(
        &db,
        CreateFromQuotationInput {
            quotation_id: qid,
            business_profile_id: None,
            issue_date: "2026-05-13".into(),
            due_date: "2026-06-12".into(),
        },
    )
    .unwrap_err();
    assert!(matches!(err, AppError::Validation(_)));
}

#[test]
fn update_rejects_non_draft() {
    let (_d, db) = fresh_db();
    let cid = seed_customer(&db, "Acme");
    let inv = service::create(&db, sample_create(cid.clone())).unwrap();
    service::mark_sent(&db, &inv.invoice.id).unwrap();

    let err = service::update(
        &db,
        UpdateInvoiceInput {
            id: inv.invoice.id,
            customer_id: cid,
            business_profile_id: None,
            issue_date: "2026-05-12".into(),
            due_date: "2026-06-11".into(),
            currency: "MYR".into(),
            tax_enabled: false,
            tax_rate: None,
            lines: vec![LineItemInput {
                description: "x".into(),
                quantity: 1.0,
                unit_price: 1.0,
            }],
            notes: None,
            terms: None,
            selected_bank_account_ids: vec![],
            selected_qr_ids: vec![],
            selected_static_methods: vec![],
        },
    )
    .unwrap_err();
    assert!(matches!(err, AppError::Validation(_)));
}

#[test]
fn state_machine_matrix_matches_claude_md() {
    use InvoiceStatus::*;

    let allowed = [
        (Draft, Sent),
        (Sent, PartialPaid),
        (Sent, Paid),
        (Sent, Overdue),
        (Sent, Void),
        (PartialPaid, Paid),
        (PartialPaid, Overdue),
        (PartialPaid, Void),
        (Overdue, Sent),
        (Overdue, PartialPaid),
        (Overdue, Paid),
        (Overdue, Void),
        (Paid, Void),
    ];
    let all = [Draft, Sent, PartialPaid, Paid, Overdue, Void];
    for &from in &all {
        for &to in &all {
            let expected = allowed.contains(&(from, to));
            assert_eq!(
                can_transition(from, to),
                expected,
                "can_transition({from:?} -> {to:?}) should be {expected}"
            );
        }
    }
}

#[test]
fn mark_paid_then_void() {
    let (_d, db) = fresh_db();
    let cid = seed_customer(&db, "Acme");
    let r = service::create(&db, sample_create(cid)).unwrap();
    service::mark_sent(&db, &r.invoice.id).unwrap();
    let paid = service::mark_paid(&db, &r.invoice.id).unwrap();
    assert_eq!(paid.status, InvoiceStatus::Paid);
    let voided = service::mark_void(&db, &r.invoice.id).unwrap();
    assert_eq!(voided.status, InvoiceStatus::Void);
}

#[test]
fn cancel_overdue_goes_back_to_sent_when_unpaid() {
    let (_d, db) = fresh_db();
    let cid = seed_customer(&db, "Acme");
    let r = service::create(&db, sample_create(cid)).unwrap();
    service::mark_sent(&db, &r.invoice.id).unwrap();
    service::mark_overdue(&db, &r.invoice.id).unwrap();
    let back = service::cancel_overdue(&db, &r.invoice.id).unwrap();
    assert_eq!(back.status, InvoiceStatus::Sent);
}

#[test]
fn cancel_overdue_goes_to_partial_when_paid_amount_positive() {
    let (_d, db) = fresh_db();
    let cid = seed_customer(&db, "Acme");
    let r = service::create(&db, sample_create(cid)).unwrap();
    let inv_id = r.invoice.id.clone();
    service::mark_sent(&db, &inv_id).unwrap();
    service::mark_overdue(&db, &inv_id).unwrap();

    // Simulate a payment recorded on this invoice (Slice 6 will do this for real).
    db.with_conn(|c| {
        c.execute(
            "UPDATE invoice SET paid_amount = 100.0 WHERE id = ?1",
            rusqlite::params![inv_id],
        )?;
        Ok(())
    })
    .unwrap();

    let back = service::cancel_overdue(&db, &inv_id).unwrap();
    assert_eq!(back.status, InvoiceStatus::PartialPaid);
}

#[test]
fn cancel_overdue_rejects_non_overdue() {
    let (_d, db) = fresh_db();
    let cid = seed_customer(&db, "Acme");
    let r = service::create(&db, sample_create(cid)).unwrap();
    let err = service::cancel_overdue(&db, &r.invoice.id).unwrap_err();
    assert!(matches!(err, AppError::Validation(_)));
}

#[test]
fn auto_mark_overdue_flips_past_due_only() {
    let (_d, db) = fresh_db();
    let cid = seed_customer(&db, "Acme");

    // Simulate the historical case: an invoice was marked Sent on a future
    // due_date that has since slipped into the past. (mark_sent itself now
    // auto-flips, so we bypass it with a raw SQL date rewrite to exercise the
    // recovery path that auto_mark_overdue_all exists for.)
    let mut past = sample_create(cid.clone());
    past.due_date = "2099-12-31".into();
    let past_inv = service::create(&db, past).unwrap();
    service::mark_sent(&db, &past_inv.invoice.id).unwrap();
    db.with_conn(|c| {
        c.execute(
            "UPDATE invoice SET due_date = '2020-01-01' WHERE id = ?1",
            rusqlite::params![past_inv.invoice.id],
        )?;
        Ok(())
    })
    .unwrap();

    // future-due, Sent → should NOT flip
    let mut future = sample_create(cid.clone());
    future.due_date = "2099-12-31".into();
    let future_inv = service::create(&db, future).unwrap();
    service::mark_sent(&db, &future_inv.invoice.id).unwrap();

    // past-due, Draft → should NOT flip (not Sent/PartialPaid yet)
    let mut draft = sample_create(cid);
    draft.due_date = "2020-01-01".into();
    let draft_inv = service::create(&db, draft).unwrap();

    let flipped = service::auto_mark_overdue_all(&db).unwrap();
    assert_eq!(flipped, 1);

    assert_eq!(
        service::find_by_id(&db, &past_inv.invoice.id)
            .unwrap()
            .invoice
            .status,
        InvoiceStatus::Overdue
    );
    assert_eq!(
        service::find_by_id(&db, &future_inv.invoice.id)
            .unwrap()
            .invoice
            .status,
        InvoiceStatus::Sent
    );
    assert_eq!(
        service::find_by_id(&db, &draft_inv.invoice.id)
            .unwrap()
            .invoice
            .status,
        InvoiceStatus::Draft
    );
}

#[test]
fn list_by_customer_filters() {
    let (_d, db) = fresh_db();
    let a = seed_customer(&db, "Acme");
    let b = seed_customer(&db, "Beta");
    service::create(&db, sample_create(a.clone())).unwrap();
    service::create(&db, sample_create(a.clone())).unwrap();
    service::create(&db, sample_create(b.clone())).unwrap();
    assert_eq!(service::list(&db).unwrap().len(), 3);
    assert_eq!(service::list_by_customer(&db, &a).unwrap().len(), 2);
    assert_eq!(service::list_by_customer(&db, &b).unwrap().len(), 1);
}

#[test]
fn create_rejects_bad_date() {
    let (_d, db) = fresh_db();
    let cid = seed_customer(&db, "Acme");
    let mut inp = sample_create(cid);
    inp.due_date = "31-12-2026".into();
    let err = service::create(&db, inp).unwrap_err();
    assert!(matches!(err, AppError::Validation(_)));
}

// ---- restore_void escape hatch ----

fn make_void_invoice_with_due(db: &Db, due_date: &str) -> String {
    let cid = seed_customer(db, "Acme");
    let mut inp = sample_create(cid);
    inp.due_date = due_date.into();
    let inv = service::create(db, inp).unwrap();
    service::mark_sent(db, &inv.invoice.id).unwrap();
    service::mark_void(db, &inv.invoice.id).unwrap();
    inv.invoice.id
}

#[test]
fn mark_sent_auto_flips_to_overdue_when_due_date_past() {
    let (_d, db) = fresh_db();
    let cid = seed_customer(&db, "Acme");
    let mut inp = sample_create(cid);
    inp.due_date = "2020-01-01".into(); // past
    let r = service::create(&db, inp).unwrap();
    let sent = service::mark_sent(&db, &r.invoice.id).unwrap();
    assert_eq!(sent.status, InvoiceStatus::Overdue);
}

#[test]
fn mark_sent_stays_sent_when_due_date_future() {
    let (_d, db) = fresh_db();
    let cid = seed_customer(&db, "Acme");
    let mut inp = sample_create(cid);
    inp.due_date = "2099-12-31".into();
    let r = service::create(&db, inp).unwrap();
    let sent = service::mark_sent(&db, &r.invoice.id).unwrap();
    assert_eq!(sent.status, InvoiceStatus::Sent);
}

#[test]
fn restore_void_unpaid_future_due_returns_to_sent() {
    let (_d, db) = fresh_db();
    let id = make_void_invoice_with_due(&db, "2099-12-31");
    let restored = service::restore_void(&db, &id).unwrap();
    assert_eq!(restored.status, InvoiceStatus::Sent);
}

#[test]
fn restore_void_unpaid_past_due_returns_to_overdue() {
    let (_d, db) = fresh_db();
    let id = make_void_invoice_with_due(&db, "2020-01-01");
    let restored = service::restore_void(&db, &id).unwrap();
    assert_eq!(restored.status, InvoiceStatus::Overdue);
}

#[test]
fn restore_void_partial_paid_future_due_returns_to_partial() {
    let (_d, db) = fresh_db();
    let id = make_void_invoice_with_due(&db, "2099-12-31");
    // Sneak a partial paid amount in (Slice 6 PV path is blocked on Void, so
    // we hack the column directly here just to exercise the recompute logic).
    db.with_conn(|c| {
        c.execute(
            "UPDATE invoice SET paid_amount = 1000.0 WHERE id = ?1",
            rusqlite::params![id],
        )?;
        Ok(())
    })
    .unwrap();
    let restored = service::restore_void(&db, &id).unwrap();
    assert_eq!(restored.status, InvoiceStatus::PartialPaid);
}

#[test]
fn restore_void_partial_past_due_returns_to_overdue() {
    let (_d, db) = fresh_db();
    let id = make_void_invoice_with_due(&db, "2020-01-01");
    db.with_conn(|c| {
        c.execute(
            "UPDATE invoice SET paid_amount = 1000.0 WHERE id = ?1",
            rusqlite::params![id],
        )?;
        Ok(())
    })
    .unwrap();
    let restored = service::restore_void(&db, &id).unwrap();
    assert_eq!(restored.status, InvoiceStatus::Overdue);
}

#[test]
fn restore_void_fully_paid_returns_to_paid() {
    let (_d, db) = fresh_db();
    let id = make_void_invoice_with_due(&db, "2020-01-01"); // past due, but paid
    // sample_create makes a 4000 + 6% = 4240 total
    db.with_conn(|c| {
        c.execute(
            "UPDATE invoice SET paid_amount = 4240.0 WHERE id = ?1",
            rusqlite::params![id],
        )?;
        Ok(())
    })
    .unwrap();
    let restored = service::restore_void(&db, &id).unwrap();
    assert_eq!(restored.status, InvoiceStatus::Paid);
}

#[test]
fn restore_void_rejects_non_void() {
    let (_d, db) = fresh_db();
    let cid = seed_customer(&db, "Acme");
    let r = service::create(&db, sample_create(cid)).unwrap();
    // Draft
    let err = service::restore_void(&db, &r.invoice.id).unwrap_err();
    assert!(matches!(err, AppError::Validation(_)));
    // Sent
    service::mark_sent(&db, &r.invoice.id).unwrap();
    let err = service::restore_void(&db, &r.invoice.id).unwrap_err();
    assert!(matches!(err, AppError::Validation(_)));
}

#[test]
fn recalc_paid_amount_returns_zero_when_no_pvs() {
    // Slice 6 wires this up to sum PVs. With no PVs, sum is 0 and the invoice's
    // paid_amount is set to 0 without any status change.
    let (_d, db) = fresh_db();
    let cid = seed_customer(&db, "Acme");
    let r = service::create(&db, sample_create(cid)).unwrap();
    let v = service::recalc_paid_amount(&db, &r.invoice.id).unwrap();
    assert_eq!(v, 0.0);
    let inv = service::find_by_id(&db, &r.invoice.id).unwrap().invoice;
    assert_eq!(inv.paid_amount, 0.0);
    assert_eq!(inv.status, InvoiceStatus::Draft);
}
