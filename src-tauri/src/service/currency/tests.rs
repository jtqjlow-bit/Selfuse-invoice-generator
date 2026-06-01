use chrono::Utc;
use tempfile::tempdir;

use crate::infra::Db;

use super::{repository, service};

fn fresh_db() -> (tempfile::TempDir, Db) {
    let dir = tempdir().unwrap();
    let db = Db::open(dir.path().join("t.db")).unwrap();
    db.run_migrations().unwrap();
    (dir, db)
}

#[test]
fn same_currency_is_one() {
    let (_dir, db) = fresh_db();
    assert_eq!(service::get_rate(&db, "usd", "USD").unwrap(), 1.0);
    assert_eq!(service::convert(&db, 100.0, "MYR", "MYR").unwrap(), 100.0);
}

#[test]
fn rejects_invalid_code() {
    let (_dir, db) = fresh_db();
    assert!(service::get_rate(&db, "US", "USD").is_err());
    assert!(service::get_rate(&db, "US1", "USD").is_err());
    assert!(service::get_rate(&db, "USDD", "USD").is_err());
}

#[test]
fn fresh_cache_hit_avoids_network() {
    let (_dir, db) = fresh_db();
    let now = Utc::now().to_rfc3339();
    db.transaction(|tx| repository::upsert(tx, "EUR", "GBP", 0.85, &now))
        .unwrap();
    // Lowercase input is normalized; a fresh cache row must satisfy the call offline.
    assert_eq!(service::get_rate(&db, "eur", "gbp").unwrap(), 0.85);
    assert!((service::convert(&db, 200.0, "EUR", "GBP").unwrap() - 170.0).abs() < 1e-9);
}

#[test]
fn list_cached_returns_rows() {
    let (_dir, db) = fresh_db();
    let now = Utc::now().to_rfc3339();
    db.transaction(|tx| {
        repository::upsert(tx, "USD", "MYR", 4.7, &now)?;
        repository::upsert(tx, "USD", "SGD", 1.35, &now)
    })
    .unwrap();
    let all = service::list_cached(&db).unwrap();
    assert_eq!(all.len(), 2);
}
