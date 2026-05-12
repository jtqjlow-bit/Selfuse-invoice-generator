use tempfile::tempdir;

use crate::error::AppError;
use crate::infra::Db;

use super::service;
use super::types::{CreateCustomerInput, CustomerType, UpdateCustomerInput};

fn fresh_db() -> (tempfile::TempDir, Db) {
    let dir = tempdir().unwrap();
    let db = Db::open(dir.path().join("test.db")).unwrap();
    db.run_migrations().unwrap();
    (dir, db)
}

fn company(name: &str) -> CreateCustomerInput {
    CreateCustomerInput {
        type_: CustomerType::Company,
        name: name.into(),
        contact_person: Some("Jane".into()),
        email: Some(format!("{name}@example.com")),
        phone: None,
        address: None,
        ssm_no: Some("SSM-001".into()),
        nric: None,
        tax_no: None,
        notes: None,
    }
}

fn individual(name: &str) -> CreateCustomerInput {
    CreateCustomerInput {
        type_: CustomerType::Individual,
        name: name.into(),
        contact_person: None,
        email: None,
        phone: None,
        address: None,
        ssm_no: None,
        nric: Some("999999-99-9999".into()),
        tax_no: None,
        notes: None,
    }
}

#[test]
fn create_company_and_find() {
    let (_d, db) = fresh_db();
    let c = service::create(&db, company("Acme")).unwrap();
    assert_eq!(c.name, "Acme");
    assert_eq!(c.type_, CustomerType::Company);
    assert!(!c.archived);
    assert!(!c.id.is_empty());

    let again = service::find_by_id(&db, &c.id).unwrap();
    assert_eq!(again.id, c.id);
    assert_eq!(again.email.as_deref(), Some("Acme@example.com"));
}

#[test]
fn create_individual_requires_nric() {
    let (_d, db) = fresh_db();
    let mut inp = individual("John");
    inp.nric = Some("   ".into()); // whitespace = empty
    let err = service::create(&db, inp).unwrap_err();
    assert!(matches!(err, AppError::Validation(_)));
}

#[test]
fn create_company_requires_ssm() {
    let (_d, db) = fresh_db();
    let mut inp = company("NoSSM");
    inp.ssm_no = None;
    let err = service::create(&db, inp).unwrap_err();
    assert!(matches!(err, AppError::Validation(_)));
}

#[test]
fn create_requires_name() {
    let (_d, db) = fresh_db();
    let mut inp = company("Acme");
    inp.name = "   ".into();
    let err = service::create(&db, inp).unwrap_err();
    assert!(matches!(err, AppError::Validation(_)));
}

#[test]
fn update_replaces_fields() {
    let (_d, db) = fresh_db();
    let c = service::create(&db, company("Acme")).unwrap();
    let updated = service::update(
        &db,
        UpdateCustomerInput {
            id: c.id.clone(),
            type_: CustomerType::Company,
            name: "Acme Studios".into(),
            contact_person: Some("Bob".into()),
            email: c.email.clone(),
            phone: Some("0123456789".into()),
            address: None,
            ssm_no: Some("SSM-002".into()),
            nric: None,
            tax_no: None,
            notes: None,
        },
    )
    .unwrap();
    assert_eq!(updated.name, "Acme Studios");
    assert_eq!(updated.ssm_no.as_deref(), Some("SSM-002"));
    assert_eq!(updated.phone.as_deref(), Some("0123456789"));
    assert_eq!(updated.contact_person.as_deref(), Some("Bob"));
    assert_eq!(updated.created_at, c.created_at);
    assert_ne!(updated.updated_at, c.updated_at);
}

#[test]
fn list_excludes_archived_by_default() {
    let (_d, db) = fresh_db();
    let a = service::create(&db, company("Alpha")).unwrap();
    let _b = service::create(&db, company("Bravo")).unwrap();
    service::archive(&db, &a.id).unwrap();

    let active = service::list(&db, false).unwrap();
    assert_eq!(active.len(), 1);
    assert_eq!(active[0].name, "Bravo");

    let all = service::list(&db, true).unwrap();
    assert_eq!(all.len(), 2);
}

#[test]
fn archive_then_unarchive() {
    let (_d, db) = fresh_db();
    let c = service::create(&db, company("Acme")).unwrap();
    let archived = service::archive(&db, &c.id).unwrap();
    assert!(archived.archived);
    let unarchived = service::unarchive(&db, &c.id).unwrap();
    assert!(!unarchived.archived);
}

#[test]
fn search_finds_by_name_email_ssm() {
    let (_d, db) = fresh_db();
    let mut a = company("Acme Studios");
    a.email = Some("hello@acme.my".into());
    a.ssm_no = Some("SSM-AAA-001".into());
    service::create(&db, a).unwrap();

    let mut b = company("Beta Films");
    b.ssm_no = Some("SSM-BBB-001".into());
    service::create(&db, b).unwrap();

    assert_eq!(service::search(&db, "acme", false).unwrap().len(), 1);
    assert_eq!(service::search(&db, "AAA", false).unwrap().len(), 1);
    assert_eq!(service::search(&db, "hello@", false).unwrap().len(), 1);
    assert_eq!(service::search(&db, "ssm", false).unwrap().len(), 2);
    assert_eq!(service::search(&db, "", false).unwrap().len(), 2); // empty falls back to list
    assert_eq!(service::search(&db, "nomatch", false).unwrap().len(), 0);
}

#[test]
fn find_missing_returns_not_found() {
    let (_d, db) = fresh_db();
    let err = service::find_by_id(&db, "does-not-exist").unwrap_err();
    assert!(matches!(err, AppError::NotFound { .. }));
}
