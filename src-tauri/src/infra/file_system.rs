//! Thin wrapper around `std::fs` for use by business modules.
//!
//! Per CLAUDE.md §12.5, business code must NOT touch `std::fs` directly. Add a
//! function here when a new file-system primitive is genuinely needed (don't
//! pre-build the whole `read_file / write_file / ensure_dir / zip_dir / unzip /
//! copy / delete` surface up front — YAGNI).
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

use walkdir::WalkDir;
use zip::{
    write::{SimpleFileOptions},
    ZipArchive, ZipWriter,
};

use crate::error::{AppError, AppResult};

pub fn ensure_dir(path: impl AsRef<Path>) -> AppResult<()> {
    std::fs::create_dir_all(path.as_ref())?;
    Ok(())
}

pub fn write_file(path: impl AsRef<Path>, content: &str) -> AppResult<()> {
    std::fs::write(path.as_ref(), content)?;
    Ok(())
}

pub fn write_bytes(path: impl AsRef<Path>, content: &[u8]) -> AppResult<()> {
    std::fs::write(path.as_ref(), content)?;
    Ok(())
}

pub fn read_file(path: impl AsRef<Path>) -> AppResult<String> {
    Ok(std::fs::read_to_string(path.as_ref())?)
}

pub fn delete_file(path: impl AsRef<Path>) -> AppResult<()> {
    match std::fs::remove_file(path.as_ref()) {
        Ok(_) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e.into()),
    }
}

pub fn path_exists(path: impl AsRef<Path>) -> bool {
    path.as_ref().exists()
}

/// Recursively delete a directory. NotFound is treated as success (idempotent).
pub fn remove_dir_all(path: impl AsRef<Path>) -> AppResult<()> {
    match std::fs::remove_dir_all(path.as_ref()) {
        Ok(_) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e.into()),
    }
}

pub fn copy_file(from: impl AsRef<Path>, to: impl AsRef<Path>) -> AppResult<()> {
    if let Some(parent) = to.as_ref().parent() {
        ensure_dir(parent)?;
    }
    std::fs::copy(from.as_ref(), to.as_ref())?;
    Ok(())
}

/// Copy a file or recursively copy a directory tree (file → file, dir → dir).
/// Missing intermediate dirs in `to` are created.
pub fn copy_path(from: impl AsRef<Path>, to: impl AsRef<Path>) -> AppResult<()> {
    let from = from.as_ref();
    let to = to.as_ref();
    if from.is_file() {
        return copy_file(from, to);
    }
    if !from.is_dir() {
        return Err(AppError::Validation(format!(
            "copy_path: source missing or not a regular file/dir: {}",
            from.display()
        )));
    }
    ensure_dir(to)?;
    for entry in WalkDir::new(from).into_iter().filter_map(|e| e.ok()) {
        let rel = entry.path().strip_prefix(from).map_err(|e| {
            AppError::Internal(format!("copy_path strip_prefix: {e}"))
        })?;
        if rel.as_os_str().is_empty() {
            continue;
        }
        let target = to.join(rel);
        if entry.path().is_dir() {
            ensure_dir(&target)?;
        } else if entry.path().is_file() {
            copy_file(entry.path(), &target)?;
        }
    }
    Ok(())
}

/// Zip the entire contents of `src_dir` into `target_zip`. Each entry path
/// inside the zip is relative to `src_dir` (so `src_dir/foo/bar.png` becomes
/// `foo/bar.png` inside the archive).
pub fn zip_dir(src_dir: impl AsRef<Path>, target_zip: impl AsRef<Path>) -> AppResult<()> {
    let src = src_dir.as_ref();
    if !src.is_dir() {
        return Err(AppError::Validation(format!(
            "zip_dir: source is not a directory: {}",
            src.display()
        )));
    }
    if let Some(parent) = target_zip.as_ref().parent() {
        ensure_dir(parent)?;
    }
    let file = File::create(target_zip.as_ref())?;
    let mut zip = ZipWriter::new(file);
    let opts = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    let mut buf = Vec::with_capacity(64 * 1024);
    for entry in WalkDir::new(src).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        let rel = path.strip_prefix(src).map_err(|e| {
            AppError::Internal(format!("zip_dir strip_prefix: {e}"))
        })?;
        if rel.as_os_str().is_empty() {
            continue;
        }
        // Zip wants forward-slash paths.
        let name = rel.to_string_lossy().replace('\\', "/");
        if path.is_dir() {
            zip.add_directory(format!("{name}/"), opts)
                .map_err(|e| AppError::Internal(format!("zip add_directory: {e}")))?;
        } else if path.is_file() {
            zip.start_file(name, opts)
                .map_err(|e| AppError::Internal(format!("zip start_file: {e}")))?;
            let mut f = File::open(path)?;
            buf.clear();
            f.read_to_end(&mut buf)?;
            zip.write_all(&buf)?;
        }
    }
    zip.finish()
        .map_err(|e| AppError::Internal(format!("zip finish: {e}")))?;
    Ok(())
}

/// Extract `src_zip` into `target_dir` (creating it if needed). Entries inside
/// the zip that try to escape `target_dir` (zip-slip) are rejected.
pub fn unzip_to_dir(src_zip: impl AsRef<Path>, target_dir: impl AsRef<Path>) -> AppResult<()> {
    let target = target_dir.as_ref();
    ensure_dir(target)?;
    let file = File::open(src_zip.as_ref())?;
    let mut archive = ZipArchive::new(file)
        .map_err(|e| AppError::Validation(format!("无法读取 zip：{e}")))?;
    for i in 0..archive.len() {
        let mut entry = archive
            .by_index(i)
            .map_err(|e| AppError::Validation(format!("zip 条目 {i} 读取失败：{e}")))?;
        let enclosed = entry
            .enclosed_name()
            .ok_or_else(|| AppError::Validation(format!("zip 条目 {i} 路径非法")))?;
        let out_path = target.join(&enclosed);
        if entry.is_dir() {
            ensure_dir(&out_path)?;
            continue;
        }
        if let Some(parent) = out_path.parent() {
            ensure_dir(parent)?;
        }
        let mut out = File::create(&out_path)?;
        std::io::copy(&mut entry, &mut out)?;
    }
    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn ensure_dir_creates_nested_directories() {
        let dir = tempdir().unwrap();
        let nested = dir.path().join("a").join("b").join("c");
        ensure_dir(&nested).unwrap();
        assert!(nested.exists());
    }

    #[test]
    fn write_then_read_roundtrip() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.txt");
        write_file(&path, "hello\nworld").unwrap();
        let s = read_file(&path).unwrap();
        assert_eq!(s, "hello\nworld");
    }

    #[test]
    fn delete_missing_is_ok() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("never_existed.txt");
        delete_file(&path).unwrap();
    }

    #[test]
    fn delete_existing_removes_it() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("doomed.txt");
        write_file(&path, "x").unwrap();
        delete_file(&path).unwrap();
        assert!(!path.exists());
    }
}
