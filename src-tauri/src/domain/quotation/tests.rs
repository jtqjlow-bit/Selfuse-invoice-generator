use tempfile::tempdir;

use crate::domain::customer;
use crate::error::AppError;
use crate::infra::Db;

use super::service;
use super::state_machine::{can_transition, QuotationStatus};
use super::types::{CreateQuotationInput, LineItemInput, UpdateQuotationInput};

fn fresh_db() -> (tempfile::TempDir, Db) {
    let dir = tempdir().unwrap();
    let db = Db::open(dir.path().join("test.db")).unwrap();
    db.run_migrations().unwrap();
    (dir, db)
}

fn seed_customer(db: &Db, name: &str) -> String {
    let c = customer::create(
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
    .unwrap();
    c.id
}

fn sample_create(customer_id: String) -> CreateQuotationInput {
    CreateQuotationInput {
        customer_id,
            business_profile_id: None,
        issue_date: "2026-05-12".into(),
        valid_until: "2026-06-11".into(),
        currency: "MYR".into(),
        tax_enabled: true,
        tax_rate: Some(0.06),
        lines: vec![
            LineItemInput {
                description: "Wedding video editing".into(),
                quantity: 1.0,
                unit_price: 3000.0,
            },
            LineItemInput {
                description: "Drone footage".into(),
                quantity: 2.0,
                unit_price: 500.0,
            },
        ],
        notes: Some("first draft".into()),
        terms: None,
    }
}

#[test]
fn create_assigns_number_and_computes_totals() {
    let (_d, db) = fresh_db();
    let cid = seed_customer(&db, "Acme");
    let r = service::create(&db, sample_create(cid)).unwrap();

    assert!(r.quotation.number.starts_with("QUO-"));
    assert_eq!(r.quotation.status, QuotationStatus::Draft);
    assert_eq!(r.lines.len(), 2);
    assert_eq!(r.lines[0].position, 1);
    assert_eq!(r.lines[1].position, 2);
    assert!((r.quotation.subtotal - 4000.0).abs() < 1e-6);
    assert!((r.quotation.tax_amount - 240.0).abs() < 1e-6);
    assert!((r.quotation.total - 4240.0).abs() < 1e-6);
    // snapshot is the customer JSON
    assert_eq!(r.quotation.customer_snapshot["name"], "Acme");
}

#[test]
fn create_increments_numbering_per_call() {
    let (_d, db) = fresh_db();
    let cid = seed_customer(&db, "Acme");
    let a = service::create(&db, sample_create(cid.clone())).unwrap();
    let b = service::create(&db, sample_create(cid)).unwrap();
    assert_ne!(a.quotation.number, b.quotation.number);
    let a_seq = a.quotation.number.rsplit('-').next().unwrap();
    let b_seq = b.quotation.number.rsplit('-').next().unwrap();
    assert_eq!(a_seq.parse::<i32>().unwrap() + 1, b_seq.parse::<i32>().unwrap());
}

#[test]
fn create_rejects_no_lines() {
    let (_d, db) = fresh_db();
    let cid = seed_customer(&db, "Acme");
    let mut inp = sample_create(cid);
    inp.lines.clear();
    let err = service::create(&db, inp).unwrap_err();
    assert!(matches!(err, AppError::Validation(_)));
}

#[test]
fn create_rejects_bad_date() {
    let (_d, db) = fresh_db();
    let cid = seed_customer(&db, "Acme");
    let mut inp = sample_create(cid);
    inp.issue_date = "12-05-2026".into();
    let err = service::create(&db, inp).unwrap_err();
    assert!(matches!(err, AppError::Validation(_)));
}

#[test]
fn create_rejects_negative_unit_price() {
    let (_d, db) = fresh_db();
    let cid = seed_customer(&db, "Acme");
    let mut inp = sample_create(cid);
    inp.lines[0].unit_price = -1.0;
    let err = service::create(&db, inp).unwrap_err();
    assert!(matches!(err, AppError::Validation(_)));
}

#[test]
fn create_rejects_missing_customer() {
    let (_d, db) = fresh_db();
    let err = service::create(&db, sample_create("not-a-real-id".into())).unwrap_err();
    assert!(matches!(err, AppError::NotFound { .. }));
}

#[test]
fn update_replaces_lines_and_recomputes() {
    let (_d, db) = fresh_db();
    let cid = seed_customer(&db, "Acme");
    let r = service::create(&db, sample_create(cid.clone())).unwrap();

    let updated = service::update(
        &db,
        UpdateQuotationInput {
            id: r.quotation.id.clone(),
            customer_id: cid,
            business_profile_id: None,
            issue_date: "2026-05-15".into(),
            valid_until: "2026-06-15".into(),
            currency: "MYR".into(),
            tax_enabled: false,
            tax_rate: None,
            lines: vec![LineItemInput {
                description: "Single shot".into(),
                quantity: 1.0,
                unit_price: 1500.0,
            }],
            notes: None,
            terms: Some("Net 30".into()),
        },
    )
    .unwrap();

    assert_eq!(updated.lines.len(), 1);
    assert!((updated.quotation.subtotal - 1500.0).abs() < 1e-6);
    assert!((updated.quotation.tax_amount - 0.0).abs() < 1e-6);
    assert!((updated.quotation.total - 1500.0).abs() < 1e-6);
    assert_eq!(updated.quotation.terms.as_deref(), Some("Net 30"));
    assert_eq!(updated.quotation.number, r.quotation.number); // number preserved
}

#[test]
fn update_rejects_non_draft() {
    let (_d, db) = fresh_db();
    let cid = seed_customer(&db, "Acme");
    let r = service::create(&db, sample_create(cid.clone())).unwrap();
    service::mark_sent(&db, &r.quotation.id).unwrap();

    let err = service::update(
        &db,
        UpdateQuotationInput {
            id: r.quotation.id,
            customer_id: cid,
            business_profile_id: None,
            issue_date: "2026-05-15".into(),
            valid_until: "2026-06-15".into(),
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
    .unwrap_err();
    assert!(matches!(err, AppError::Validation(_)));
}

#[test]
fn state_machine_allows_only_documented_transitions() {
    use QuotationStatus::*;
    assert!(can_transition(Draft, Sent));
    assert!(can_transition(Sent, Accepted));
    assert!(can_transition(Sent, Rejected));
    assert!(can_transition(Sent, Expired));

    assert!(!can_transition(Draft, Accepted));
    assert!(!can_transition(Draft, Rejected));
    assert!(!can_transition(Accepted, Sent));
    assert!(!can_transition(Rejected, Sent));
    assert!(!can_transition(Expired, Sent));
    assert!(!can_transition(Accepted, Rejected));
}

#[test]
fn mark_transitions_persist_and_reject_illegal() {
    let (_d, db) = fresh_db();
    let cid = seed_customer(&db, "Acme");
    let r = service::create(&db, sample_create(cid)).unwrap();

    // Draft -> Accepted not allowed
    let err = service::mark_accepted(&db, &r.quotation.id).unwrap_err();
    assert!(matches!(err, AppError::InvalidTransition { .. }));

    // Draft -> Sent OK
    let sent = service::mark_sent(&db, &r.quotation.id).unwrap();
    assert_eq!(sent.status, QuotationStatus::Sent);

    // Sent -> Sent not allowed
    let err = service::mark_sent(&db, &r.quotation.id).unwrap_err();
    assert!(matches!(err, AppError::InvalidTransition { .. }));

    // Sent -> Accepted OK
    let accepted = service::mark_accepted(&db, &r.quotation.id).unwrap();
    assert_eq!(accepted.status, QuotationStatus::Accepted);
}

#[test]
fn find_by_id_returns_lines_in_position_order() {
    let (_d, db) = fresh_db();
    let cid = seed_customer(&db, "Acme");
    let r = service::create(&db, sample_create(cid)).unwrap();
    let again = service::find_by_id(&db, &r.quotation.id).unwrap();
    assert_eq!(again.lines.len(), 2);
    assert_eq!(again.lines[0].position, 1);
    assert_eq!(again.lines[1].position, 2);
    assert_eq!(again.lines[0].description, "Wedding video editing");
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
fn cascade_delete_lines_on_quotation_delete() {
    // Not directly callable since service has no delete (mark_void is invoice only),
    // but verify the FK cascade is wired correctly via a raw delete.
    let (_d, db) = fresh_db();
    let cid = seed_customer(&db, "Acme");
    let r = service::create(&db, sample_create(cid)).unwrap();
    let qid = r.quotation.id.clone();

    db.with_conn(|c| {
        c.execute("DELETE FROM quotation WHERE id = ?1", rusqlite::params![qid])?;
        Ok(())
    })
    .unwrap();

    let remaining: i64 = db
        .with_conn(|c| {
            Ok(c.query_row(
                "SELECT COUNT(*) FROM quotation_line_item WHERE quotation_id = ?1",
                rusqlite::params![qid],
                |r| r.get(0),
            )?)
        })
        .unwrap();
    assert_eq!(remaining, 0);
}
