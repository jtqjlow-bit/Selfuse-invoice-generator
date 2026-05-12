use tempfile::tempdir;

use crate::infra::Db;

use super::service;
use super::types::{
    BankAccount, CreateBusinessProfileInput, EntityType, UpdateBusinessProfileInput,
};

fn fresh_db() -> (tempfile::TempDir, Db) {
    let dir = tempdir().unwrap();
    let db = Db::open(dir.path().join("test.db")).unwrap();
    db.run_migrations().unwrap();
    (dir, db)
}

fn make_company_input() -> CreateBusinessProfileInput {
    CreateBusinessProfileInput {
        entity_type: EntityType::Company,
        name: "Acme Studios".into(),
        address: Some("Kuala Lumpur".into()),
        email: Some("hello@acme.my".into()),
        phone: None,
        ssm_no: Some("SSM-001".into()),
        nric: None,
        sst_no: None,
        bank_accounts: vec![BankAccount {
            id: String::new(),
            bank_name: "Maybank".into(),
            account_number: "1234567890".into(),
            account_holder: "Acme Studios SDN BHD".into(),
        }],
        enabled_payment_methods: vec!["FPX".into(), "DuitNow".into()],
        default_tax_rate: Some(0.06),
        default_quotation_valid_days: 30,
        default_invoice_due_days: 14,
        data_dir: "C:\\Users\\me\\InvoiceData".into(),
    }
}

fn make_individual_input() -> CreateBusinessProfileInput {
    CreateBusinessProfileInput {
        entity_type: EntityType::Individual,
        name: "Jane Doe".into(),
        ssm_no: None,
        nric: Some("900101-01-1234".into()),
        ..make_company_input()
    }
}

#[test]
fn fresh_db_has_no_profiles() {
    let (_d, db) = fresh_db();
    assert!(service::list(&db).unwrap().is_empty());
}

#[test]
fn create_then_list_round_trips() {
    let (_d, db) = fresh_db();
    let p = service::create(&db, make_company_input()).unwrap();
    assert_eq!(p.entity_type, EntityType::Company);
    assert_eq!(p.name, "Acme Studios");
    assert_eq!(p.bank_accounts.len(), 1);

    let list = service::list(&db).unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].id, p.id);
}

#[test]
fn supports_multiple_profiles() {
    let (_d, db) = fresh_db();
    let a = service::create(&db, make_company_input()).unwrap();
    let b = service::create(&db, make_individual_input()).unwrap();
    let list = service::list(&db).unwrap();
    assert_eq!(list.len(), 2);
    assert!(list.iter().any(|x| x.id == a.id));
    assert!(list.iter().any(|x| x.id == b.id));
}

#[test]
fn update_changes_fields() {
    let (_d, db) = fresh_db();
    let p = service::create(&db, make_company_input()).unwrap();
    let updated = service::update(
        &db,
        UpdateBusinessProfileInput {
            id: p.id.clone(),
            entity_type: EntityType::Company,
            name: "Acme V2".into(),
            address: None,
            email: None,
            phone: None,
            ssm_no: Some("SSM-002".into()),
            nric: None,
            sst_no: None,
            bank_accounts: vec![],
            enabled_payment_methods: vec![],
            default_tax_rate: None,
            default_quotation_valid_days: 30,
            default_invoice_due_days: 14,
            data_dir: "".into(),
        },
    )
    .unwrap();
    assert_eq!(updated.name, "Acme V2");
    assert_eq!(updated.ssm_no.as_deref(), Some("SSM-002"));
}

#[test]
fn delete_removes_profile() {
    let (_d, db) = fresh_db();
    let p = service::create(&db, make_company_input()).unwrap();
    service::delete(&db, &p.id).unwrap();
    assert!(service::list(&db).unwrap().is_empty());
}

#[test]
fn rejects_empty_name() {
    let (_d, db) = fresh_db();
    let mut input = make_company_input();
    input.name = "  ".into();
    let err = service::create(&db, input).unwrap_err();
    match err {
        crate::error::AppError::Validation(msg) => assert!(msg.contains("公司名")),
        other => panic!("expected Validation, got {other:?}"),
    }
}

#[test]
fn company_requires_ssm() {
    let (_d, db) = fresh_db();
    let mut input = make_company_input();
    input.ssm_no = None;
    let err = service::create(&db, input).unwrap_err();
    match err {
        crate::error::AppError::Validation(msg) => assert!(msg.contains("SSM")),
        other => panic!("expected Validation, got {other:?}"),
    }
}

#[test]
fn individual_requires_nric() {
    let (_d, db) = fresh_db();
    let mut input = make_individual_input();
    input.nric = None;
    let err = service::create(&db, input).unwrap_err();
    match err {
        crate::error::AppError::Validation(msg) => assert!(msg.contains("NRIC")),
        other => panic!("expected Validation, got {other:?}"),
    }
}

#[test]
fn switching_to_individual_clears_logo() {
    let (_d, db) = fresh_db();
    let p = service::create(&db, make_company_input()).unwrap();
    // Simulate already having a logo by directly setting path via repo.
    // (Easier than going through set_logo with base64 in a test.)
    crate::infra::Db::with_conn(&db, |c| {
        c.execute(
            "UPDATE business_profile SET logo_path = ?1 WHERE id = ?2",
            rusqlite::params!["dummy.png", p.id],
        )
        .unwrap();
        Ok(())
    })
    .unwrap();

    let updated = service::update(
        &db,
        UpdateBusinessProfileInput {
            id: p.id.clone(),
            entity_type: EntityType::Individual,
            name: "Self".into(),
            address: None,
            email: None,
            phone: None,
            ssm_no: None,
            nric: Some("123".into()),
            sst_no: None,
            bank_accounts: vec![],
            enabled_payment_methods: vec![],
            default_tax_rate: None,
            default_quotation_valid_days: 30,
            default_invoice_due_days: 14,
            data_dir: "".into(),
        },
    )
    .unwrap();
    assert!(updated.logo_path.is_none());
}
