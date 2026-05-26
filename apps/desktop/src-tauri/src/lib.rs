use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Manager};
use walkdir::WalkDir;

#[derive(Debug, Serialize)]
struct MediaAsset {
    id: i64,
    file_path: String,
    file_name: String,
    extension: String,
    media_type: String,
    file_size: i64,
    created_at: Option<i64>,
    modified_at: Option<i64>,
    imported_at: i64,
}

#[derive(Debug, Serialize)]
struct ImportSummary {
    folder_path: String,
    imported_count: usize,
    skipped_count: usize,
    total_media_count: i64,
    assets: Vec<MediaAsset>,
}

#[derive(Debug, Serialize)]
struct LibraryOverview {
    total_media_count: i64,
    photo_count: i64,
    video_count: i64,
    assets: Vec<MediaAsset>,
}

#[tauri::command]
fn import_media_folder(app: AppHandle, folder_path: String) -> Result<ImportSummary, String> {
    let folder = PathBuf::from(&folder_path);
    if !folder.exists() {
        return Err("选择的文件夹不存在".to_string());
    }
    if !folder.is_dir() {
        return Err("请选择一个文件夹，而不是单个文件".to_string());
    }

    let db_path = database_path(&app)?;
    let conn = Connection::open(db_path).map_err(|error| error.to_string())?;
    initialize_database(&conn)?;

    let mut imported_count = 0usize;
    let mut skipped_count = 0usize;
    let imported_at = unix_timestamp_now();

    for entry in WalkDir::new(&folder).follow_links(false).into_iter().filter_map(Result::ok) {
        if !entry.file_type().is_file() {
            continue;
        }

        let path = entry.path();
        let Some(media_type) = media_type_for_path(path) else {
            continue;
        };

        let path_text = path.to_string_lossy().to_string();
        let already_exists = asset_exists(&conn, &path_text)?;
        if already_exists {
            skipped_count += 1;
            continue;
        }

        let metadata = fs::metadata(path).map_err(|error| error.to_string())?;
        let file_size = safe_i64_from_u64(metadata.len());
        let file_name = path
            .file_name()
            .map(|value| value.to_string_lossy().to_string())
            .unwrap_or_else(|| "未命名文件".to_string());
        let extension = path
            .extension()
            .map(|value| value.to_string_lossy().to_lowercase())
            .unwrap_or_default();
        let created_at = metadata.created().ok().and_then(system_time_to_unix);
        let modified_at = metadata.modified().ok().and_then(system_time_to_unix);

        conn.execute(
            "INSERT INTO media_assets (
                file_path,
                file_name,
                extension,
                media_type,
                file_size,
                created_at,
                modified_at,
                imported_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                path_text,
                file_name,
                extension,
                media_type,
                file_size,
                created_at,
                modified_at,
                imported_at
            ],
        )
        .map_err(|error| error.to_string())?;

        imported_count += 1;
    }

    let overview = load_library_overview(&conn)?;

    Ok(ImportSummary {
        folder_path,
        imported_count,
        skipped_count,
        total_media_count: overview.total_media_count,
        assets: overview.assets,
    })
}

#[tauri::command]
fn get_library_overview(app: AppHandle) -> Result<LibraryOverview, String> {
    let db_path = database_path(&app)?;
    let conn = Connection::open(db_path).map_err(|error| error.to_string())?;
    initialize_database(&conn)?;
    load_library_overview(&conn)
}

fn database_path(app: &AppHandle) -> Result<PathBuf, String> {
    let mut app_data_dir = app.path().app_data_dir().map_err(|error| error.to_string())?;
    fs::create_dir_all(&app_data_dir).map_err(|error| error.to_string())?;
    app_data_dir.push("yinghe_ai.sqlite3");
    Ok(app_data_dir)
}

fn initialize_database(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS media_assets (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            file_path TEXT NOT NULL UNIQUE,
            file_name TEXT NOT NULL,
            extension TEXT NOT NULL,
            media_type TEXT NOT NULL,
            file_size INTEGER NOT NULL,
            created_at INTEGER,
            modified_at INTEGER,
            imported_at INTEGER NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_media_assets_media_type ON media_assets(media_type);
        CREATE INDEX IF NOT EXISTS idx_media_assets_imported_at ON media_assets(imported_at);",
    )
    .map_err(|error| error.to_string())
}

fn asset_exists(conn: &Connection, file_path: &str) -> Result<bool, String> {
    let existing_id: Option<i64> = conn
        .query_row(
            "SELECT id FROM media_assets WHERE file_path = ?1 LIMIT 1",
            params![file_path],
            |row| row.get(0),
        )
        .optional()
        .map_err(|error| error.to_string())?;

    Ok(existing_id.is_some())
}

fn load_library_overview(conn: &Connection) -> Result<LibraryOverview, String> {
    let total_media_count = count_by_media_type(conn, None)?;
    let photo_count = count_by_media_type(conn, Some("photo"))?;
    let video_count = count_by_media_type(conn, Some("video"))?;
    let assets = load_recent_assets(conn, 500)?;

    Ok(LibraryOverview {
        total_media_count,
        photo_count,
        video_count,
        assets,
    })
}

fn count_by_media_type(conn: &Connection, media_type: Option<&str>) -> Result<i64, String> {
    match media_type {
        Some(value) => conn
            .query_row(
                "SELECT COUNT(*) FROM media_assets WHERE media_type = ?1",
                params![value],
                |row| row.get(0),
            )
            .map_err(|error| error.to_string()),
        None => conn
            .query_row("SELECT COUNT(*) FROM media_assets", [], |row| row.get(0))
            .map_err(|error| error.to_string()),
    }
}

fn load_recent_assets(conn: &Connection, limit: i64) -> Result<Vec<MediaAsset>, String> {
    let mut statement = conn
        .prepare(
            "SELECT
                id,
                file_path,
                file_name,
                extension,
                media_type,
                file_size,
                created_at,
                modified_at,
                imported_at
            FROM media_assets
            ORDER BY imported_at DESC, id DESC
            LIMIT ?1",
        )
        .map_err(|error| error.to_string())?;

    let rows = statement
        .query_map(params![limit], |row| {
            Ok(MediaAsset {
                id: row.get(0)?,
                file_path: row.get(1)?,
                file_name: row.get(2)?,
                extension: row.get(3)?,
                media_type: row.get(4)?,
                file_size: row.get(5)?,
                created_at: row.get(6)?,
                modified_at: row.get(7)?,
                imported_at: row.get(8)?,
            })
        })
        .map_err(|error| error.to_string())?;

    let mut assets = Vec::new();
    for row in rows {
        assets.push(row.map_err(|error| error.to_string())?);
    }

    Ok(assets)
}

fn media_type_for_path(path: &Path) -> Option<&'static str> {
    let extension = path.extension()?.to_string_lossy().to_lowercase();

    match extension.as_str() {
        "jpg" | "jpeg" | "png" | "webp" | "heic" | "heif" | "tif" | "tiff" | "bmp"
        | "gif" | "dng" | "arw" | "cr2" | "cr3" | "nef" | "orf" | "rw2" | "raf"
        | "pef" => Some("photo"),
        "mp4" | "mov" | "m4v" | "avi" | "mkv" | "webm" => Some("video"),
        _ => None,
    }
}

fn safe_i64_from_u64(value: u64) -> i64 {
    if value > i64::MAX as u64 {
        i64::MAX
    } else {
        value as i64
    }
}

fn system_time_to_unix(value: SystemTime) -> Option<i64> {
    value
        .duration_since(UNIX_EPOCH)
        .ok()
        .map(|duration| duration.as_secs() as i64)
}

fn unix_timestamp_now() -> i64 {
    system_time_to_unix(SystemTime::now()).unwrap_or_default()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            import_media_folder,
            get_library_overview
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
