use exif::{Field, Reader, Tag, Value};
use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;
use std::fs::{self, File};
use std::io::BufReader;
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
    taken_at: Option<String>,
    camera_make: Option<String>,
    camera_model: Option<String>,
    lens_model: Option<String>,
    gps_latitude: Option<f64>,
    gps_longitude: Option<f64>,
}

#[derive(Debug, Default)]
struct ExifMetadata {
    taken_at: Option<String>,
    camera_make: Option<String>,
    camera_model: Option<String>,
    lens_model: Option<String>,
    gps_latitude: Option<f64>,
    gps_longitude: Option<f64>,
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

    for entry in WalkDir::new(&folder)
        .follow_links(false)
        .into_iter()
        .filter_map(Result::ok)
    {
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
        let exif_metadata = read_exif_metadata(path, media_type);

        conn.execute(
            "INSERT INTO media_assets (
                file_path,
                file_name,
                extension,
                media_type,
                file_size,
                created_at,
                modified_at,
                imported_at,
                taken_at,
                camera_make,
                camera_model,
                lens_model,
                gps_latitude,
                gps_longitude
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
            params![
                path_text,
                file_name,
                extension,
                media_type,
                file_size,
                created_at,
                modified_at,
                imported_at,
                exif_metadata.taken_at,
                exif_metadata.camera_make,
                exif_metadata.camera_model,
                exif_metadata.lens_model,
                exif_metadata.gps_latitude,
                exif_metadata.gps_longitude
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
    let mut app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?;
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
            imported_at INTEGER NOT NULL,
            taken_at TEXT,
            camera_make TEXT,
            camera_model TEXT,
            lens_model TEXT,
            gps_latitude REAL,
            gps_longitude REAL
        );",
    )
    .map_err(|error| error.to_string())?;

    ensure_media_asset_column(conn, "taken_at", "taken_at TEXT")?;
    ensure_media_asset_column(conn, "camera_make", "camera_make TEXT")?;
    ensure_media_asset_column(conn, "camera_model", "camera_model TEXT")?;
    ensure_media_asset_column(conn, "lens_model", "lens_model TEXT")?;
    ensure_media_asset_column(conn, "gps_latitude", "gps_latitude REAL")?;
    ensure_media_asset_column(conn, "gps_longitude", "gps_longitude REAL")?;

    conn.execute_batch(
        "CREATE INDEX IF NOT EXISTS idx_media_assets_media_type ON media_assets(media_type);
        CREATE INDEX IF NOT EXISTS idx_media_assets_imported_at ON media_assets(imported_at);
        CREATE INDEX IF NOT EXISTS idx_media_assets_taken_at ON media_assets(taken_at);
        CREATE INDEX IF NOT EXISTS idx_media_assets_camera_model ON media_assets(camera_model);",
    )
    .map_err(|error| error.to_string())
}

fn ensure_media_asset_column(
    conn: &Connection,
    column_name: &str,
    column_definition: &str,
) -> Result<(), String> {
    if media_asset_column_exists(conn, column_name)? {
        return Ok(());
    }

    conn.execute(
        &format!("ALTER TABLE media_assets ADD COLUMN {column_definition}"),
        [],
    )
    .map(|_| ())
    .map_err(|error| error.to_string())
}

fn media_asset_column_exists(conn: &Connection, column_name: &str) -> Result<bool, String> {
    let mut statement = conn
        .prepare("PRAGMA table_info(media_assets)")
        .map_err(|error| error.to_string())?;
    let rows = statement
        .query_map([], |row| row.get::<_, String>(1))
        .map_err(|error| error.to_string())?;

    for row in rows {
        if row.map_err(|error| error.to_string())? == column_name {
            return Ok(true);
        }
    }

    Ok(false)
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
                imported_at,
                taken_at,
                camera_make,
                camera_model,
                lens_model,
                gps_latitude,
                gps_longitude
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
                taken_at: row.get(9)?,
                camera_make: row.get(10)?,
                camera_model: row.get(11)?,
                lens_model: row.get(12)?,
                gps_latitude: row.get(13)?,
                gps_longitude: row.get(14)?,
            })
        })
        .map_err(|error| error.to_string())?;

    let mut assets = Vec::new();
    for row in rows {
        assets.push(row.map_err(|error| error.to_string())?);
    }

    Ok(assets)
}

fn read_exif_metadata(path: &Path, media_type: &str) -> ExifMetadata {
    if media_type != "photo" {
        return ExifMetadata::default();
    }

    let Ok(file) = File::open(path) else {
        return ExifMetadata::default();
    };
    let mut reader = BufReader::new(file);
    let Ok(exif) = Reader::new().read_from_container(&mut reader) else {
        return ExifMetadata::default();
    };

    ExifMetadata {
        taken_at: exif_text(&exif, Tag::DateTimeOriginal)
            .or_else(|| exif_text(&exif, Tag::DateTime))
            .and_then(|value| normalize_exif_datetime(&value)),
        camera_make: exif_text(&exif, Tag::Make),
        camera_model: exif_text(&exif, Tag::Model),
        lens_model: exif_text(&exif, Tag::LensModel),
        gps_latitude: gps_coordinate(&exif, Tag::GPSLatitude, Tag::GPSLatitudeRef),
        gps_longitude: gps_coordinate(&exif, Tag::GPSLongitude, Tag::GPSLongitudeRef),
    }
}

fn find_exif_field(exif: &exif::Exif, tag: Tag) -> Option<&Field> {
    exif.fields().find(|field| field.tag == tag)
}

fn exif_text(exif: &exif::Exif, tag: Tag) -> Option<String> {
    let field = find_exif_field(exif, tag)?;

    match &field.value {
        Value::Ascii(values) => values.iter().find_map(|value| {
            let text = String::from_utf8_lossy(value);
            clean_exif_text(text.as_ref())
        }),
        _ => clean_exif_text(&field.display_value().with_unit(exif).to_string()),
    }
}

fn clean_exif_text(raw: &str) -> Option<String> {
    let value = raw
        .replace('\0', "")
        .trim()
        .trim_matches('"')
        .trim_matches('\'')
        .trim()
        .to_string();

    if value.is_empty() {
        None
    } else {
        Some(value)
    }
}

fn normalize_exif_datetime(raw: &str) -> Option<String> {
    let value = clean_exif_text(raw)?;

    if let Some(prefix) = value.get(0..19) {
        let bytes = prefix.as_bytes();
        if bytes[4] == b':'
            && bytes[7] == b':'
            && bytes[10] == b' '
            && bytes[13] == b':'
            && bytes[16] == b':'
        {
            return Some(format!(
                "{}-{}-{} {}:{}:{}",
                &prefix[0..4],
                &prefix[5..7],
                &prefix[8..10],
                &prefix[11..13],
                &prefix[14..16],
                &prefix[17..19]
            ));
        }
    }

    Some(value)
}

fn gps_coordinate(exif: &exif::Exif, coordinate_tag: Tag, reference_tag: Tag) -> Option<f64> {
    let coordinate_field = find_exif_field(exif, coordinate_tag)?;
    let values = match &coordinate_field.value {
        Value::Rational(values) => values,
        _ => return None,
    };

    if values.len() < 3 {
        return None;
    }

    let degrees = rational_to_f64(&values[0])?;
    let minutes = rational_to_f64(&values[1])?;
    let seconds = rational_to_f64(&values[2])?;
    let mut decimal = degrees + minutes / 60.0 + seconds / 3600.0;

    let reference = exif_text(exif, reference_tag)
        .unwrap_or_default()
        .to_uppercase();
    if matches!(reference.as_str(), "S" | "W") {
        decimal = -decimal;
    }

    Some((decimal * 1_000_000.0).round() / 1_000_000.0)
}

fn rational_to_f64(value: &exif::Rational) -> Option<f64> {
    if value.denom == 0 {
        return None;
    }

    Some(value.num as f64 / value.denom as f64)
}

fn media_type_for_path(path: &Path) -> Option<&'static str> {
    let extension = path.extension()?.to_string_lossy().to_lowercase();

    match extension.as_str() {
        "jpg" | "jpeg" | "png" | "webp" | "heic" | "heif" | "tif" | "tiff" | "bmp" | "gif"
        | "dng" | "arw" | "cr2" | "cr3" | "nef" | "orf" | "rw2" | "raf" | "pef" => Some("photo"),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_exif_datetime() {
        assert_eq!(
            normalize_exif_datetime("2024:05:02 13:04:05"),
            Some("2024-05-02 13:04:05".to_string())
        );
    }

    #[test]
    fn keeps_nonstandard_exif_datetime_text() {
        assert_eq!(
            normalize_exif_datetime("2024-05-02T13:04:05"),
            Some("2024-05-02T13:04:05".to_string())
        );
    }

    #[test]
    fn drops_empty_exif_text() {
        assert_eq!(clean_exif_text(" \0 "), None);
    }

    #[test]
    fn imports_required_photo_video_and_raw_formats() {
        for extension in [
            "jpg", "jpeg", "png", "webp", "gif", "bmp", "heic", "dng", "arw", "cr2", "cr3", "nef",
            "mp4", "mov", "webm",
        ] {
            let file_name = format!("sample.{extension}");
            assert!(
                media_type_for_path(Path::new(&file_name)).is_some(),
                "{extension} should be importable"
            );
        }
    }

    #[test]
    fn migrates_existing_stage_three_database() {
        let conn = Connection::open_in_memory().expect("open in-memory database");
        conn.execute_batch(
            "CREATE TABLE media_assets (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                file_path TEXT NOT NULL UNIQUE,
                file_name TEXT NOT NULL,
                extension TEXT NOT NULL,
                media_type TEXT NOT NULL,
                file_size INTEGER NOT NULL,
                created_at INTEGER,
                modified_at INTEGER,
                imported_at INTEGER NOT NULL
            );",
        )
        .expect("create old media_assets table");

        initialize_database(&conn).expect("migrate media_assets table");
        assert!(media_asset_column_exists(&conn, "taken_at").expect("check taken_at column"));
        assert!(
            media_asset_column_exists(&conn, "camera_model").expect("check camera_model column")
        );
        assert!(
            media_asset_column_exists(&conn, "gps_longitude").expect("check gps_longitude column")
        );

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
                "/photos/example.jpg",
                "example.jpg",
                "jpg",
                "photo",
                123_i64,
                Option::<i64>::None,
                Option::<i64>::None,
                1_i64
            ],
        )
        .expect("insert old asset row");

        let overview = load_library_overview(&conn).expect("load migrated overview");
        assert_eq!(overview.total_media_count, 1);
        assert_eq!(overview.photo_count, 1);
        assert_eq!(overview.video_count, 0);
        assert_eq!(overview.assets[0].file_name, "example.jpg");
        assert_eq!(overview.assets[0].taken_at, None);
    }
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
