use tempfile::tempdir;

use crate::error::AppError;
use crate::infra::Db;

use super::service;
use super::types::DocType;

fn fresh_db() -> (tempfile::TempDir, Db) {
    let dir = tempdir().unwrap();
    let db = Db::open(dir.path().join("test.db")).unwrap();
    db.run_migrations().unwrap();
    (dir, db)
}

fn current_year() -> i32 {
    use chrono::{Datelike, Local};
    Local::now().year()
}

#[test]
fn next_starts_at_001() {
    let (_d, db) = fresh_db();
    let n = service::next(&db, DocType::Quotation).unwrap();
    assert_eq!(n, format!("QUO-{:04}-001", current_year()));
}

#[test]
fn next_increments_and_separates_doc_types() {
    let (_d, db) = fresh_db();
    let y = current_year();

    assert_eq!(
        service::next(&db, DocType::Quotation).unwrap(),
        format!("QUO-{y:04}-001")
    );
    assert_eq!(
        service::next(&db, DocType::Quotation).unwrap(),
        format!("QUO-{y:04}-002")
    );
    assert_eq!(
        service::next(&db, DocType::Invoice).unwrap(),
        format!("INV-{y:04}-001")
    );
    assert_eq!(
        service::next(&db, DocType::PaymentVoucher).unwrap(),
        format!("PV-{y:04}-001")
    );
    assert_eq!(
        service::next(&db, DocType::Quotation).unwrap(),
        format!("QUO-{y:04}-003")
    );
}

#[test]
fn peek_does_not_consume() {
    let (_d, db) = fresh_db();
    let y = current_year();

    assert_eq!(
        service::peek(&db, DocType::Invoice).unwrap(),
        format!("INV-{y:04}-001")
    );
    assert_eq!(
        service::peek(&db, DocType::Invoice).unwrap(),
        format!("INV-{y:04}-001"),
        "peek must not increment"
    );
    assert_eq!(
        service::next(&db, DocType::Invoice).unwrap(),
        format!("INV-{y:04}-001")
    );
    assert_eq!(
        service::peek(&db, DocType::Invoice).unwrap(),
        format!("INV-{y:04}-002")
    );
}

#[test]
fn set_override_applies_to_subsequent_next() {
    let (_d, db) = fresh_db();
    let y = current_year();

    service::next(&db, DocType::Invoice).unwrap(); // -> 001
    service::set_override(&db, DocType::Invoice, y, 99).unwrap();
    let n = service::next(&db, DocType::Invoice).unwrap();
    assert_eq!(n, format!("INV-{y:04}-100"));
}

#[test]
fn set_override_rejects_negative() {
    let (_d, db) = fresh_db();
    let y = current_year();
    let err = service::set_override(&db, DocType::Invoice, y, -1).unwrap_err();
    assert!(matches!(err, AppError::Validation(_)));
}

#[test]
fn set_override_rejects_bad_year() {
    let (_d, db) = fresh_db();
    let err = service::set_override(&db, DocType::Invoice, 1999, 0).unwrap_err();
    assert!(matches!(err, AppError::Validation(_)));
    let err = service::set_override(&db, DocType::Invoice, 3000, 0).unwrap_err();
    assert!(matches!(err, AppError::Validation(_)));
}

#[test]
fn yearly_isolation_via_set_override() {
    // We can't easily simulate a year-rollover, so we just verify counters in different
    // years are independent rows.
    let (_d, db) = fresh_db();
    let y = current_year();
    service::set_override(&db, DocType::Quotation, y - 1, 42).unwrap();
    // The current-year counter is untouched, so next() in current year still returns 001.
    let n = service::next(&db, DocType::Quotation).unwrap();
    assert_eq!(n, format!("QUO-{y:04}-001"));
    // Override on a different (past) year persists independently.
    let last_prev: i64 = db
        .with_conn(|c| {
            Ok(c.query_row(
                "SELECT last_seq FROM numbering_counter WHERE doc_type = 'Quotation' AND year = ?1",
                rusqlite::params![y - 1],
                |r| r.get(0),
            )?)
        })
        .unwrap();
    assert_eq!(last_prev, 42);
}

#[test]
fn padding_handles_three_digits_and_overflow() {
    let (_d, db) = fresh_db();
    let y = current_year();
    service::set_override(&db, DocType::Invoice, y, 999).unwrap();
    assert_eq!(
        service::next(&db, DocType::Invoice).unwrap(),
        format!("INV-{y:04}-1000"),
        "seq > 999 should overflow the padding"
    );
}
