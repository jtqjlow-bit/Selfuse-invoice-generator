use rusqlite::{params, Connection, OptionalExtension, Row};

use crate::error::{AppError, AppResult};

use super::types::{BankAccount, BusinessProfile, EntityType, Qr};

fn map_row(row: &Row<'_>) -> rusqlite::Result<BusinessProfile> {
    let bank_accounts_json: String = row.get("bank_accounts")?;
    let enabled_methods_json: String = row.get("enabled_payment_methods")?;
    let qrs_json: String = row.get("qrs")?;
    let entity_type_str: String = row.get("entity_type")?;

    let bank_accounts: Vec<BankAccount> = serde_json::from_str(&bank_accounts_json)
        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e)))?;
    let enabled_payment_methods: Vec<String> = serde_json::from_str(&enabled_methods_json)
        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e)))?;
    let qrs: Vec<Qr> = serde_json::from_str(&qrs_json)
        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e)))?;
    let entity_type = EntityType::from_str(&entity_type_str).ok_or_else(|| {
        rusqlite::Error::FromSqlConversionFailure(
            0,
            rusqlite::types::Type::Text,
            format!("unknown entity_type '{entity_type_str}'").into(),
        )
    })?;

    Ok(BusinessProfile {
        id: row.get("id")?,
        entity_type,
        name: row.get("name")?,
        address: row.get("address")?,
        email: row.get("email")?,
        phone: row.get("phone")?,
        ssm_no: row.get("ssm_no")?,
        nric: row.get("nric")?,
        sst_no: row.get("sst_no")?,
        logo_path: row.get("logo_path")?,
        qr_path: row.get("qr_path")?,
        bank_accounts,
        qrs,
        enabled_payment_methods,
        default_tax_rate: row.get("default_tax_rate")?,
        default_quotation_valid_days: row.get("default_quotation_valid_days")?,
        default_invoice_due_days: row.get("default_invoice_due_days")?,
        data_dir: row.get("data_dir")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
    })
}

pub fn insert(conn: &Connection, p: &BusinessProfile) -> AppResult<()> {
    let bank_accounts_json = serde_json::to_string(&p.bank_accounts)
        .map_err(|e| AppError::Internal(format!("serialize bank_accounts: {e}")))?;
    let enabled_methods_json = serde_json::to_string(&p.enabled_payment_methods)
        .map_err(|e| AppError::Internal(format!("serialize enabled_payment_methods: {e}")))?;
    let qrs_json = serde_json::to_string(&p.qrs)
        .map_err(|e| AppError::Internal(format!("serialize qrs: {e}")))?;
    conn.execute(
        "INSERT INTO business_profile (
            id, entity_type, name, address, email, phone, ssm_no, nric, sst_no,
            logo_path, qr_path, bank_accounts, qrs, enabled_payment_methods,
            default_tax_rate, default_quotation_valid_days, default_invoice_due_days,
            data_dir, created_at, updated_at
        ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,?20)",
        params![
            p.id,
            p.entity_type.as_str(),
            p.name,
            p.address,
            p.email,
            p.phone,
            p.ssm_no,
            p.nric,
            p.sst_no,
            p.logo_path,
            p.qr_path,
            bank_accounts_json,
            qrs_json,
            enabled_methods_json,
            p.default_tax_rate,
            p.default_quotation_valid_days,
            p.default_invoice_due_days,
            p.data_dir,
            p.created_at,
            p.updated_at,
        ],
    )?;
    Ok(())
}

pub fn update(conn: &Connection, p: &BusinessProfile) -> AppResult<()> {
    let bank_accounts_json = serde_json::to_string(&p.bank_accounts)
        .map_err(|e| AppError::Internal(format!("serialize bank_accounts: {e}")))?;
    let enabled_methods_json = serde_json::to_string(&p.enabled_payment_methods)
        .map_err(|e| AppError::Internal(format!("serialize enabled_payment_methods: {e}")))?;
    let qrs_json = serde_json::to_string(&p.qrs)
        .map_err(|e| AppError::Internal(format!("serialize qrs: {e}")))?;
    let affected = conn.execute(
        "UPDATE business_profile SET
            entity_type = ?2,
            name = ?3,
            address = ?4,
            email = ?5,
            phone = ?6,
            ssm_no = ?7,
            nric = ?8,
            sst_no = ?9,
            logo_path = ?10,
            qr_path = ?11,
            bank_accounts = ?12,
            qrs = ?13,
            enabled_payment_methods = ?14,
            default_tax_rate = ?15,
            default_quotation_valid_days = ?16,
            default_invoice_due_days = ?17,
            data_dir = ?18,
            updated_at = ?19
        WHERE id = ?1",
        params![
            p.id,
            p.entity_type.as_str(),
            p.name,
            p.address,
            p.email,
            p.phone,
            p.ssm_no,
            p.nric,
            p.sst_no,
            p.logo_path,
            p.qr_path,
            bank_accounts_json,
            qrs_json,
            enabled_methods_json,
            p.default_tax_rate,
            p.default_quotation_valid_days,
            p.default_invoice_due_days,
            p.data_dir,
            p.updated_at,
        ],
    )?;
    if affected == 0 {
        return Err(AppError::NotFound {
            entity: "business_profile".into(),
            id: p.id.clone(),
        });
    }
    Ok(())
}

pub fn delete(conn: &Connection, id: &str) -> AppResult<()> {
    let affected = conn.execute("DELETE FROM business_profile WHERE id = ?1", params![id])?;
    if affected == 0 {
        return Err(AppError::NotFound {
            entity: "business_profile".into(),
            id: id.into(),
        });
    }
    Ok(())
}

pub fn find_by_id(conn: &Connection, id: &str) -> AppResult<Option<BusinessProfile>> {
    Ok(conn
        .query_row(
            "SELECT * FROM business_profile WHERE id = ?1",
            params![id],
            map_row,
        )
        .optional()?)
}

pub fn list(conn: &Connection) -> AppResult<Vec<BusinessProfile>> {
    let mut s = conn.prepare("SELECT * FROM business_profile ORDER BY created_at ASC")?;
    let rows = s.query_map([], map_row)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

pub fn set_logo_path(conn: &Connection, id: &str, path: Option<&str>) -> AppResult<()> {
    let now = chrono::Utc::now().to_rfc3339();
    let affected = conn.execute(
        "UPDATE business_profile SET logo_path = ?1, updated_at = ?2 WHERE id = ?3",
        params![path, now, id],
    )?;
    if affected == 0 {
        return Err(AppError::NotFound {
            entity: "business_profile".into(),
            id: id.into(),
        });
    }
    Ok(())
}

pub fn set_qr_path(conn: &Connection, id: &str, path: Option<&str>) -> AppResult<()> {
    let now = chrono::Utc::now().to_rfc3339();
    let affected = conn.execute(
        "UPDATE business_profile SET qr_path = ?1, updated_at = ?2 WHERE id = ?3",
        params![path, now, id],
    )?;
    if affected == 0 {
        return Err(AppError::NotFound {
            entity: "business_profile".into(),
            id: id.into(),
        });
    }
    Ok(())
}
