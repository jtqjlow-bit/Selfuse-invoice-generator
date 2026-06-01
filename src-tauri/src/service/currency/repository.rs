use rusqlite::{params, Connection, OptionalExtension};

use crate::error::AppResult;

use super::types::ExchangeRate;

fn map_row(r: &rusqlite::Row<'_>) -> rusqlite::Result<ExchangeRate> {
    Ok(ExchangeRate {
        base: r.get(0)?,
        target: r.get(1)?,
        rate: r.get(2)?,
        fetched_at: r.get(3)?,
    })
}

pub fn get_cached(conn: &Connection, base: &str, target: &str) -> AppResult<Option<ExchangeRate>> {
    let row = conn
        .query_row(
            "SELECT base, target, rate, fetched_at FROM exchange_rate_cache
             WHERE base = ?1 AND target = ?2",
            params![base, target],
            map_row,
        )
        .optional()?;
    Ok(row)
}

pub fn upsert(
    conn: &Connection,
    base: &str,
    target: &str,
    rate: f64,
    fetched_at: &str,
) -> AppResult<()> {
    conn.execute(
        "INSERT INTO exchange_rate_cache (base, target, rate, fetched_at)
         VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(base, target)
         DO UPDATE SET rate = excluded.rate, fetched_at = excluded.fetched_at",
        params![base, target, rate, fetched_at],
    )?;
    Ok(())
}

pub fn list_all(conn: &Connection) -> AppResult<Vec<ExchangeRate>> {
    let mut stmt = conn.prepare(
        "SELECT base, target, rate, fetched_at FROM exchange_rate_cache
         ORDER BY base, target",
    )?;
    let rows = stmt.query_map([], map_row)?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}
