use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::infra::Db;

use super::repository;
use super::types::{CreateCustomerInput, Customer, CustomerType, UpdateCustomerInput};

fn now() -> String {
    chrono::Utc::now().to_rfc3339()
}

fn validate_identity(type_: CustomerType, ssm_no: &Option<String>, nric: &Option<String>) -> AppResult<()> {
    match type_ {
        CustomerType::Company => {
            if ssm_no.as_ref().map(|s| s.trim().is_empty()).unwrap_or(true) {
                return Err(AppError::Validation(
                    "Company 类型必须填写 SSM 号".into(),
                ));
            }
        }
        CustomerType::Individual => {
            if nric.as_ref().map(|s| s.trim().is_empty()).unwrap_or(true) {
                return Err(AppError::Validation(
                    "Individual 类型必须填写 NRIC".into(),
                ));
            }
        }
    }
    Ok(())
}

pub fn create(db: &Db, input: CreateCustomerInput) -> AppResult<Customer> {
    if input.name.trim().is_empty() {
        return Err(AppError::Validation("name 不能为空".into()));
    }
    validate_identity(input.type_, &input.ssm_no, &input.nric)?;

    let now_s = now();
    let c = Customer {
        id: Uuid::new_v4().to_string(),
        type_: input.type_,
        name: input.name.trim().to_string(),
        contact_person: trim_opt(input.contact_person),
        email: trim_opt(input.email),
        phone: trim_opt(input.phone),
        address: trim_opt(input.address),
        ssm_no: trim_opt(input.ssm_no),
        nric: trim_opt(input.nric),
        tax_no: trim_opt(input.tax_no),
        notes: trim_opt(input.notes),
        archived: false,
        created_at: now_s.clone(),
        updated_at: now_s,
    };
    db.transaction(|tx| {
        repository::insert(tx, &c)?;
        Ok(())
    })?;
    Ok(c)
}

pub fn update(db: &Db, input: UpdateCustomerInput) -> AppResult<Customer> {
    if input.name.trim().is_empty() {
        return Err(AppError::Validation("name 不能为空".into()));
    }
    validate_identity(input.type_, &input.ssm_no, &input.nric)?;

    db.transaction(|tx| {
        let mut existing = repository::find_by_id(tx, &input.id)?.ok_or_else(|| {
            AppError::NotFound {
                entity: "customer".into(),
                id: input.id.clone(),
            }
        })?;
        existing.type_ = input.type_;
        existing.name = input.name.trim().to_string();
        existing.contact_person = trim_opt(input.contact_person);
        existing.email = trim_opt(input.email);
        existing.phone = trim_opt(input.phone);
        existing.address = trim_opt(input.address);
        existing.ssm_no = trim_opt(input.ssm_no);
        existing.nric = trim_opt(input.nric);
        existing.tax_no = trim_opt(input.tax_no);
        existing.notes = trim_opt(input.notes);
        existing.updated_at = now();
        repository::update(tx, &existing)?;
        Ok(existing)
    })
}

pub fn find_by_id(db: &Db, id: &str) -> AppResult<Customer> {
    db.with_conn(|c| {
        repository::find_by_id(c, id)?.ok_or_else(|| AppError::NotFound {
            entity: "customer".into(),
            id: id.into(),
        })
    })
}

pub fn list(db: &Db, include_archived: bool) -> AppResult<Vec<Customer>> {
    db.with_conn(|c| repository::list(c, include_archived))
}

pub fn search(db: &Db, query: &str, include_archived: bool) -> AppResult<Vec<Customer>> {
    let q = query.trim();
    if q.is_empty() {
        return list(db, include_archived);
    }
    db.with_conn(|c| repository::search(c, q, include_archived))
}

pub fn archive(db: &Db, id: &str) -> AppResult<Customer> {
    db.transaction(|tx| {
        repository::set_archived(tx, id, true, &now())?;
        repository::find_by_id(tx, id)?.ok_or_else(|| AppError::NotFound {
            entity: "customer".into(),
            id: id.into(),
        })
    })
}

pub fn unarchive(db: &Db, id: &str) -> AppResult<Customer> {
    db.transaction(|tx| {
        repository::set_archived(tx, id, false, &now())?;
        repository::find_by_id(tx, id)?.ok_or_else(|| AppError::NotFound {
            entity: "customer".into(),
            id: id.into(),
        })
    })
}

fn trim_opt(v: Option<String>) -> Option<String> {
    v.and_then(|s| {
        let t = s.trim();
        if t.is_empty() { None } else { Some(t.to_string()) }
    })
}
