use std::path::{Path, PathBuf};
use std::sync::Mutex;

use rusqlite::{Connection, OpenFlags};

use crate::error::{AppError, AppResult};

const MIGRATIONS: &[(i32, &str, &str)] = &[
    (1, "initial", include_str!("../../migrations/0001_initial.sql")),
    (
        2,
        "company_settings",
        include_str!("../../migrations/0002_company_settings.sql"),
    ),
    (
        3,
        "customer",
        include_str!("../../migrations/0003_customer.sql"),
    ),
    (
        4,
        "numbering_counter",
        include_str!("../../migrations/0004_numbering_counter.sql"),
    ),
    (
        5,
        "quotation",
        include_str!("../../migrations/0005_quotation.sql"),
    ),
    (
        6,
        "invoice",
        include_str!("../../migrations/0006_invoice.sql"),
    ),
    (
        7,
        "payment_voucher",
        include_str!("../../migrations/0007_payment_voucher.sql"),
    ),
    (
        8,
        "pdf_template",
        include_str!("../../migrations/0008_pdf_template.sql"),
    ),
    (
        9,
        "payment_voucher_nullable_invoice",
        include_str!("../../migrations/0009_payment_voucher_nullable_invoice.sql"),
    ),
    (
        10,
        "company_settings_entity_type",
        include_str!("../../migrations/0010_company_settings_entity_type.sql"),
    ),
    (
        11,
        "business_profile",
        include_str!("../../migrations/0011_business_profile.sql"),
    ),
    (
        12,
        "multi_qr_invoice_picker",
        include_str!("../../migrations/0012_multi_qr_invoice_picker.sql"),
    ),
    (
        13,
        "exchange_rate_cache",
        include_str!("../../migrations/0013_exchange_rate_cache.sql"),
    ),
];

pub struct Db {
    conn: Mutex<Connection>,
    #[allow(dead_code)] // surfaced for diagnostics / future backup paths
    path: PathBuf,
}

impl Db {
    pub fn open(path: impl AsRef<Path>) -> AppResult<Self> {
        let path = path.as_ref().to_path_buf();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open_with_flags(
            &path,
            OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE,
        )?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;
        Ok(Self {
            conn: Mutex::new(conn),
            path,
        })
    }

    pub fn run_migrations(&self) -> AppResult<()> {
        let mut conn = self
            .conn
            .lock()
            .map_err(|_| AppError::Internal("db mutex poisoned".into()))?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS _migrations (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                applied_at TEXT NOT NULL
            )",
            [],
        )?;

        for (id, name, sql) in MIGRATIONS {
            let already_applied: bool = conn.query_row(
                "SELECT EXISTS(SELECT 1 FROM _migrations WHERE id = ?1)",
                [id],
                |row| row.get(0),
            )?;
            if already_applied {
                continue;
            }
            let tx = conn.transaction()?;
            tx.execute_batch(sql)?;
            tx.execute(
                "INSERT INTO _migrations (id, name, applied_at) VALUES (?1, ?2, ?3)",
                rusqlite::params![id, name, chrono::Utc::now().to_rfc3339()],
            )?;
            tx.commit()?;
        }
        Ok(())
    }

    pub fn with_conn<F, R>(&self, f: F) -> AppResult<R>
    where
        F: FnOnce(&Connection) -> AppResult<R>,
    {
        let conn = self
            .conn
            .lock()
            .map_err(|_| AppError::Internal("db mutex poisoned".into()))?;
        f(&conn)
    }

    pub fn transaction<F, R>(&self, f: F) -> AppResult<R>
    where
        F: FnOnce(&rusqlite::Transaction<'_>) -> AppResult<R>,
    {
        let mut conn = self
            .conn
            .lock()
            .map_err(|_| AppError::Internal("db mutex poisoned".into()))?;
        let tx = conn.transaction()?;
        let result = f(&tx)?;
        tx.commit()?;
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn opens_and_creates_parent_dir() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("nested").join("test.db");
        let db = Db::open(&path).unwrap();
        db.run_migrations().unwrap();
        assert!(path.exists());
    }

    #[test]
    fn migrations_idempotent() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.db");
        let db = Db::open(&path).unwrap();
        db.run_migrations().unwrap();
        db.run_migrations().unwrap(); // re-run, should not re-apply
        let count = db
            .with_conn(|c| {
                Ok(c.query_row("SELECT COUNT(*) FROM _migrations", [], |r| r.get::<_, i64>(0))?)
            })
            .unwrap();
        assert_eq!(count as usize, MIGRATIONS.len());
    }

    #[test]
    fn transaction_rolls_back_on_error() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.db");
        let db = Db::open(&path).unwrap();
        db.run_migrations().unwrap();
        db.with_conn(|c| {
            c.execute("CREATE TABLE t (v INTEGER)", [])?;
            Ok(())
        })
        .unwrap();

        let result: AppResult<()> = db.transaction(|tx| {
            tx.execute("INSERT INTO t (v) VALUES (1)", [])?;
            Err(AppError::Internal("boom".into()))
        });
        assert!(result.is_err());

        let count: i64 = db
            .with_conn(|c| Ok(c.query_row("SELECT COUNT(*) FROM t", [], |r| r.get(0))?))
            .unwrap();
        assert_eq!(count, 0);
    }
}
