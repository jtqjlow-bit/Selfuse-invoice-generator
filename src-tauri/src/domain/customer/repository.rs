use rusqlite::{params, Connection, OptionalExtension, Row};

use crate::error::{AppError, AppResult};

use super::types::{Customer, CustomerType};

fn map_row(row: &Row<'_>) -> rusqlite::Result<Customer> {
    let type_str: String = row.get("type_")?;
    let type_ = CustomerType::from_str(&type_str).ok_or_else(|| {
        rusqlite::Error::FromSqlConversionFailure(
            0,
            rusqlite::types::Type::Text,
            format!("unknown customer type {type_str}").into(),
        )
    })?;
    let archived_int: i64 = row.get("archived")?;
    Ok(Customer {
        id: row.get("id")?,
        type_,
        name: row.get("name")?,
        contact_person: row.get("contact_person")?,
        email: row.get("email")?,
        phone: row.get("phone")?,
        address: row.get("address")?,
        ssm_no: row.get("ssm_no")?,
        nric: row.get("nric")?,
        tax_no: row.get("tax_no")?,
        notes: row.get("notes")?,
        archived: archived_int != 0,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
    })
}

pub fn insert(conn: &Connection, c: &Customer) -> AppResult<()> {
    conn.execute(
        "INSERT INTO customer (
            id, type_, name, contact_person, email, phone, address,
            ssm_no, nric, tax_no, notes, archived, created_at, updated_at
        ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14)",
        params![
            c.id,
            c.type_.as_str(),
            c.name,
            c.contact_person,
            c.email,
            c.phone,
            c.address,
            c.ssm_no,
            c.nric,
            c.tax_no,
            c.notes,
            c.archived as i64,
            c.created_at,
            c.updated_at,
        ],
    )?;
    Ok(())
}

pub fn update(conn: &Connection, c: &Customer) -> AppResult<()> {
    let affected = conn.execute(
        "UPDATE customer SET
            type_ = ?2, name = ?3, contact_person = ?4, email = ?5, phone = ?6,
            address = ?7, ssm_no = ?8, nric = ?9, tax_no = ?10, notes = ?11,
            updated_at = ?12
        WHERE id = ?1",
        params![
            c.id,
            c.type_.as_str(),
            c.name,
            c.contact_person,
            c.email,
            c.phone,
            c.address,
            c.ssm_no,
            c.nric,
            c.tax_no,
            c.notes,
            c.updated_at,
        ],
    )?;
    if affected == 0 {
        return Err(AppError::NotFound {
            entity: "customer".into(),
            id: c.id.clone(),
        });
    }
    Ok(())
}

pub fn set_archived(conn: &Connection, id: &str, archived: bool, now: &str) -> AppResult<()> {
    let affected = conn.execute(
        "UPDATE customer SET archived = ?1, updated_at = ?2 WHERE id = ?3",
        params![archived as i64, now, id],
    )?;
    if affected == 0 {
        return Err(AppError::NotFound {
            entity: "customer".into(),
            id: id.into(),
        });
    }
    Ok(())
}

pub fn find_by_id(conn: &Connection, id: &str) -> AppResult<Option<Customer>> {
    Ok(conn
        .query_row("SELECT * FROM customer WHERE id = ?1", params![id], map_row)
        .optional()?)
}

pub fn list(conn: &Connection, include_archived: bool) -> AppResult<Vec<Customer>> {
    let sql = if include_archived {
        "SELECT * FROM customer ORDER BY archived ASC, name COLLATE NOCASE ASC"
    } else {
        "SELECT * FROM customer WHERE archived = 0 ORDER BY name COLLATE NOCASE ASC"
    };
    let mut s = conn.prepare(sql)?;
    let rows = s.query_map([], map_row)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

pub fn search(conn: &Connection, query: &str, include_archived: bool) -> AppResult<Vec<Customer>> {
    let pat = format!("%{}%", query.trim());
    let sql = if include_archived {
        "SELECT * FROM customer
         WHERE name LIKE ?1 OR IFNULL(contact_person,'') LIKE ?1
            OR IFNULL(email,'') LIKE ?1 OR IFNULL(phone,'') LIKE ?1
            OR IFNULL(ssm_no,'') LIKE ?1 OR IFNULL(nric,'') LIKE ?1
         ORDER BY archived ASC, name COLLATE NOCASE ASC"
    } else {
        "SELECT * FROM customer
         WHERE archived = 0 AND (
            name LIKE ?1 OR IFNULL(contact_person,'') LIKE ?1
            OR IFNULL(email,'') LIKE ?1 OR IFNULL(phone,'') LIKE ?1
            OR IFNULL(ssm_no,'') LIKE ?1 OR IFNULL(nric,'') LIKE ?1
         )
         ORDER BY name COLLATE NOCASE ASC"
    };
    let mut s = conn.prepare(sql)?;
    let rows = s.query_map(params![pat], map_row)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}
