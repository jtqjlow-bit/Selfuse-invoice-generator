use std::path::Path;

use tempfile::tempdir;

use crate::domain::customer;
use crate::infra::{file_system, Db};

use super::service;

fn fresh_db(dir: &Path) -> Db {
    let db = Db::open(dir.join("t.db")).unwrap();
    db.run_migrations().unwrap();
    db
}

#[test]
fn imports_valid_rows_and_reports_bad_ones() {
    let dir = tempdir().unwrap();
    let db = fresh_db(dir.path());
    let csv = "type,name,contact_person,email,phone,address,ssm_no,nric,tax_no,notes\n\
               Company,Acme Sdn Bhd,Ali,a@x.com,012,,SSM123,,SST9,hello\n\
               Individual,John,,,,,,NRIC123,,\n\
               Banana,BadType,,,,,,,,\n\
               Company,,,,,,,,,\n";
    let path = dir.path().join("customers.csv");
    file_system::write_file(&path, csv).unwrap();

    let report = service::import_customers_from_csv(&db, path.to_str().unwrap()).unwrap();
    assert_eq!(report.total, 4);
    assert_eq!(report.imported, 2);
    assert_eq!(report.failed, 2);
    assert_eq!(report.errors.len(), 2);
    // bad type is line 4, empty name is line 5 (header = line 1).
    assert_eq!(report.errors[0].line, 4);
    assert_eq!(report.errors[1].line, 5);

    let all = customer::list(&db, true).unwrap();
    assert_eq!(all.len(), 2);
}

#[test]
fn missing_required_identity_is_reported_not_fatal() {
    let dir = tempdir().unwrap();
    let db = fresh_db(dir.path());
    // Company row without ssm_no must be rejected by domain validation, but
    // the whole import should still succeed and report the failure.
    let csv = "type,name,contact_person,email,phone,address,ssm_no,nric,tax_no,notes\n\
               Company,No SSM,,,,,,,,\n";
    let path = dir.path().join("c.csv");
    file_system::write_file(&path, csv).unwrap();

    let report = service::import_customers_from_csv(&db, path.to_str().unwrap()).unwrap();
    assert_eq!(report.total, 1);
    assert_eq!(report.imported, 0);
    assert_eq!(report.failed, 1);
    assert_eq!(customer::list(&db, true).unwrap().len(), 0);
}

#[test]
fn exports_xlsx_file() {
    let dir = tempdir().unwrap();
    let db = fresh_db(dir.path());
    customer::create(
        &db,
        customer::CreateCustomerInput {
            type_: customer::CustomerType::Individual,
            name: "Jane".into(),
            contact_person: None,
            email: None,
            phone: None,
            address: None,
            ssm_no: None,
            nric: Some("NRIC1".into()),
            tax_no: None,
            notes: None,
        },
    )
    .unwrap();

    let out = dir.path().join("export.xlsx");
    service::export_all_to_excel(&db, out.to_str().unwrap()).unwrap();
    assert!(out.exists());
    let bytes = std::fs::read(&out).unwrap();
    assert!(bytes.len() > 100);
    // .xlsx is a zip archive; it starts with the "PK" magic bytes.
    assert_eq!(&bytes[0..2], b"PK");
}
