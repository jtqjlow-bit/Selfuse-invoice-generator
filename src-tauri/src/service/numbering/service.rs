use chrono::{Datelike, Local};

use crate::error::{AppError, AppResult};
use crate::infra::Db;

use super::repository;
use super::types::{format_number, DocType};

fn current_year() -> i32 {
    Local::now().year()
}

/// Atomically reserve and return the next number for the given doc type in the current local year.
pub fn next(db: &Db, doc: DocType) -> AppResult<String> {
    let year = current_year();
    db.transaction(|tx| {
        let last = repository::get_last_seq(tx, doc, year)?;
        let next_seq = last + 1;
        repository::upsert_last_seq(tx, doc, year, next_seq)?;
        Ok(format_number(doc.prefix(), year, next_seq))
    })
}

/// Preview the number that `next()` would produce without consuming it.
pub fn peek(db: &Db, doc: DocType) -> AppResult<String> {
    let year = current_year();
    db.with_conn(|c| {
        let last = repository::get_last_seq(c, doc, year)?;
        Ok(format_number(doc.prefix(), year, last + 1))
    })
}

/// User-initiated override: force the next assigned seq to be `seq + 1` for the given year.
/// Useful when importing existing books or correcting a mis-issue.
pub fn set_override(db: &Db, doc: DocType, year: i32, seq: i64) -> AppResult<()> {
    if seq < 0 {
        return Err(AppError::Validation("seq 不能为负数".into()));
    }
    if !(2000..=2999).contains(&year) {
        return Err(AppError::Validation(
            "year 必须在 2000 ~ 2999 之间".into(),
        ));
    }
    db.transaction(|tx| repository::upsert_last_seq(tx, doc, year, seq))
}
