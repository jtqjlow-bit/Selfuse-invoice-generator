use rusqlite::{params, Connection, OptionalExtension};

use crate::error::AppResult;

use super::types::DocType;

pub fn get_last_seq(conn: &Connection, doc: DocType, year: i32) -> AppResult<i64> {
    let v: Option<i64> = conn
        .query_row(
            "SELECT last_seq FROM numbering_counter WHERE doc_type = ?1 AND year = ?2",
            params![doc.as_str(), year],
            |r| r.get(0),
        )
        .optional()?;
    Ok(v.unwrap_or(0))
}

pub fn upsert_last_seq(conn: &Connection, doc: DocType, year: i32, seq: i64) -> AppResult<()> {
    conn.execute(
        "INSERT INTO numbering_counter (doc_type, year, last_seq) VALUES (?1, ?2, ?3)
         ON CONFLICT(doc_type, year) DO UPDATE SET last_seq = excluded.last_seq",
        params![doc.as_str(), year, seq],
    )?;
    Ok(())
}
