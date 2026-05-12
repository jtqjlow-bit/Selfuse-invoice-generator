//! Export the app's data folder to a zip; restore from a zip.
//!
//! Backup contents:
//!   - `invoice.db` (after a WAL checkpoint so the main file is up to date)
//!   - `assets/`   (logos + QRs)
//!   - `templates/` (custom Tera templates uploaded by the user)
//!
//! Restore can't replace `invoice.db` while the live SQLite connection holds a
//! handle to it (Windows file lock). So we extract to a staging dir + drop a
//! marker file; on next launch, lib.rs's setup hook calls `apply_pending_restore`
//! BEFORE opening the DB.
use std::path::Path;

use rusqlite::Connection;

use crate::error::{AppError, AppResult};
use crate::infra::{file_system, Db};

const STAGING_DIRNAME: &str = ".restore_staging";
const MARKER_FILENAME: &str = ".pending_restore";

/// Names inside the app_data_dir that we back up + restore.
const BACKUP_ITEMS: &[&str] = &["invoice.db", "assets", "templates"];

pub fn export_zip(db: &Db, app_data_dir: &Path, target_zip: &Path) -> AppResult<()> {
    // Flush WAL into the main DB so the copied invoice.db is current.
    db.with_conn(|c| {
        c.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")?;
        Ok(())
    })?;

    // Stage what we want zipped into a temp dir. Cheapest way to get a stable
    // snapshot to feed into zip_dir without including .restore_staging itself
    // or .pdf_tmp.
    let stage = app_data_dir.join(".export_staging");
    file_system::remove_dir_all(&stage)?;
    file_system::ensure_dir(&stage)?;

    for name in BACKUP_ITEMS {
        let src = app_data_dir.join(name);
        if !file_system::path_exists(&src) {
            continue;
        }
        let dst = stage.join(name);
        file_system::copy_path(&src, &dst)?;
    }

    file_system::zip_dir(&stage, target_zip)?;
    file_system::remove_dir_all(&stage)?;
    Ok(())
}

/// Validate the zip + extract to staging + write the marker. The actual swap
/// happens on next launch via `apply_pending_restore`.
pub fn restore_zip(app_data_dir: &Path, zip_path: &Path) -> AppResult<()> {
    let staging = app_data_dir.join(STAGING_DIRNAME);
    file_system::remove_dir_all(&staging)?;
    file_system::unzip_to_dir(zip_path, &staging)?;

    // Must contain invoice.db, and it must be a valid SQLite file.
    let probe_db = staging.join("invoice.db");
    if !file_system::path_exists(&probe_db) {
        file_system::remove_dir_all(&staging)?;
        return Err(AppError::Validation(
            "备份文件无效：压缩包内缺少 invoice.db".into(),
        ));
    }
    let conn = Connection::open(&probe_db)
        .map_err(|e| AppError::Validation(format!("备份文件不是有效的 SQLite：{e}")))?;
    conn.query_row("SELECT count(*) FROM sqlite_master", [], |row| {
        let _: i64 = row.get(0)?;
        Ok(())
    })
    .map_err(|e| AppError::Validation(format!("无法读取备份 DB：{e}")))?;
    drop(conn);

    let marker = app_data_dir.join(MARKER_FILENAME);
    file_system::write_file(&marker, "pending")?;
    Ok(())
}

/// Called by lib.rs before opening the DB. If a pending-restore marker exists,
/// swap files into place + delete the marker; otherwise no-op.
pub fn apply_pending_restore(app_data_dir: &Path) -> AppResult<()> {
    let marker = app_data_dir.join(MARKER_FILENAME);
    if !file_system::path_exists(&marker) {
        return Ok(());
    }
    let staging = app_data_dir.join(STAGING_DIRNAME);
    if !file_system::path_exists(&staging) {
        file_system::delete_file(&marker)?;
        return Ok(());
    }

    // Wipe + replace the backed-up subtree. Also clean WAL/SHM so SQLite
    // doesn't replay stale WAL against the new DB file.
    for sibling in ["invoice.db-wal", "invoice.db-shm"] {
        let p = app_data_dir.join(sibling);
        if file_system::path_exists(&p) {
            file_system::delete_file(&p)?;
        }
    }
    for name in BACKUP_ITEMS {
        let live = app_data_dir.join(name);
        let staged = staging.join(name);
        if live.is_dir() {
            file_system::remove_dir_all(&live)?;
        } else if live.is_file() {
            file_system::delete_file(&live)?;
        }
        if file_system::path_exists(&staged) {
            file_system::copy_path(&staged, &live)?;
        }
    }
    file_system::remove_dir_all(&staging)?;
    file_system::delete_file(&marker)?;
    Ok(())
}
