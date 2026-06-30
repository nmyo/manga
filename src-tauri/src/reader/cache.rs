use super::image_decode::{should_decode_image, source_extension};
use super::types::{
    ReaderCacheStatsResult, ReaderManifest, ReaderPage, DEFAULT_READER_CACHE_LIMIT_BYTES,
    MAX_READER_CACHE_LIMIT_BYTES, MIN_READER_CACHE_LIMIT_BYTES,
};
use crate::api::{ApiError, ApiErrorKind, ApiResult};
use crate::diagnostics;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use tauri::{AppHandle, Manager};

pub(crate) fn file_size_bytes(path: &Path) -> Option<u64> {
    fs::metadata(path).map(|metadata| metadata.len()).ok()
}

pub(crate) fn write_temp_reader_cache_file<F>(cache_path: &Path, write: F) -> ApiResult<()>
where
    F: FnOnce(&Path) -> ApiResult<()>,
{
    let temp_path = reader_page_temp_cache_path(cache_path);

    if temp_path.exists() {
        let _ = fs::remove_file(&temp_path);
    }

    if let Err(error) = write(&temp_path) {
        let _ = fs::remove_file(&temp_path);
        return Err(error);
    }

    persist_reader_cache_file(&temp_path, cache_path)
}

fn persist_reader_cache_file(temp_path: &Path, cache_path: &Path) -> ApiResult<()> {
    match fs::rename(temp_path, cache_path) {
        Ok(()) => Ok(()),
        Err(error) => {
            let _ = fs::remove_file(temp_path);

            if cache_path.exists() {
                Ok(())
            } else {
                Err(map_cache_error(error))
            }
        }
    }
}

pub(crate) fn reader_cache_root(app: &AppHandle) -> ApiResult<PathBuf> {
    app.path()
        .app_cache_dir()
        .map(|path| path.join("reader"))
        .map_err(|error| ApiError::new(ApiErrorKind::Cache, error.to_string()))
}

pub(crate) fn reader_page_cache_path(
    cache_root: &Path,
    manifest: &ReaderManifest,
    page: &ReaderPage,
) -> ApiResult<PathBuf> {
    let extension = if should_decode_image(manifest, page) {
        "webp"
    } else {
        source_extension(&page.source_url)
    };
    let read_dir = cache_root.join(safe_path_segment(&manifest.read_id));

    Ok(read_dir.join(format!("{:04}.{extension}", page.index + 1)))
}

fn reader_page_temp_cache_path(cache_path: &Path) -> PathBuf {
    let file_name = cache_path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("page");
    let nonce = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();

    cache_path.with_file_name(format!(
        "{}.{}.{}.tmp",
        file_name,
        std::process::id(),
        nonce
    ))
}

fn is_reader_cache_temp_path(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("tmp"))
}

pub(crate) fn cleanup_reader_cache(cache_root: &Path, cache_limit_bytes: u64) -> ApiResult<()> {
    let files = collect_cache_files(cache_root)?;
    let total_size = files.iter().map(|file| file.size).sum::<u64>();

    if total_size <= cache_limit_bytes {
        return Ok(());
    }

    let cache_trim_bytes = cache_trim_bytes(cache_limit_bytes);
    let mut files = files;
    files.sort_by_key(|file| file.modified);
    let mut current_size = total_size;

    for file in files {
        if current_size <= cache_trim_bytes {
            break;
        }

        match fs::remove_file(&file.path) {
            Ok(()) => {
                current_size = current_size.saturating_sub(file.size);
            }
            Err(error) => {
                diagnostics::warn(format!(
                    "Failed to remove reader cache file {:?}: {error}",
                    file.path
                ));
            }
        }
    }

    Ok(())
}

#[derive(Debug)]
struct CacheFile {
    path: PathBuf,
    size: u64,
    modified: SystemTime,
}

fn collect_cache_files(cache_root: &Path) -> ApiResult<Vec<CacheFile>> {
    let mut files = Vec::new();

    if !cache_root.exists() {
        return Ok(files);
    }

    collect_cache_files_in(cache_root, &mut files)?;

    Ok(files)
}

fn collect_cache_files_in(dir: &Path, files: &mut Vec<CacheFile>) -> ApiResult<()> {
    for entry in fs::read_dir(dir).map_err(map_cache_error)? {
        let entry = entry.map_err(map_cache_error)?;
        let path = entry.path();
        let metadata = entry.metadata().map_err(map_cache_error)?;

        if metadata.is_dir() {
            collect_cache_files_in(&path, files)?;
        } else if metadata.is_file() {
            if is_reader_cache_temp_path(&path) {
                continue;
            }

            files.push(CacheFile {
                path,
                size: metadata.len(),
                modified: metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH),
            });
        }
    }

    Ok(())
}

pub(crate) fn reader_cache_stats(
    cache_root: PathBuf,
    cache_limit_bytes: u64,
) -> ApiResult<ReaderCacheStatsResult> {
    let files = collect_cache_files(&cache_root)?;
    let total_bytes = files.iter().map(|file| file.size).sum::<u64>();

    Ok(ReaderCacheStatsResult {
        cache_dir: cache_root.to_string_lossy().to_string(),
        total_bytes,
        file_count: files.len() as u32,
        cache_limit_bytes,
        cache_trim_bytes: cache_trim_bytes(cache_limit_bytes),
    })
}

pub(crate) fn normalize_cache_limit(cache_limit_bytes: Option<u64>) -> u64 {
    cache_limit_bytes
        .unwrap_or(DEFAULT_READER_CACHE_LIMIT_BYTES)
        .clamp(MIN_READER_CACHE_LIMIT_BYTES, MAX_READER_CACHE_LIMIT_BYTES)
}

pub(crate) fn cache_trim_bytes(cache_limit_bytes: u64) -> u64 {
    cache_limit_bytes.saturating_mul(82) / 100
}

fn safe_path_segment(value: &str) -> String {
    let segment = value
        .chars()
        .filter(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_'))
        .collect::<String>();

    if segment.is_empty() {
        "unknown".to_string()
    } else {
        segment
    }
}

pub(crate) fn map_cache_error(error: std::io::Error) -> ApiError {
    ApiError::new(ApiErrorKind::Cache, error.to_string())
}
