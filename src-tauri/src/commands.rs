use hual::cache::L2Cache;
use hual::pipeline;
use hual::pipeline::{list_photos as list_photos_hual, open_db, PhotoRow};
use hual::thumbnail;
use std::path::{Path, PathBuf};
use tauri_plugin_dialog::DialogExt;

#[tauri::command]
pub fn get_preview(path: String) -> Result<Vec<u8>, String> {
    thumbnail::extract_thumbnail(Path::new(&path)).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_webp(cache_dir: String, src: String) -> Option<Vec<u8>> {
    let cache = L2Cache::new(PathBuf::from(&cache_dir)).ok()?;
    cache.get(&src)
}

#[tauri::command]
pub fn list_photos(db_path: String) -> Result<Vec<PhotoRow>, String> {
    let conn = open_db(Path::new(&db_path)).map_err(|e| e.to_string())?;
    list_photos_hual(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn import_photos(src: String, dest: String) -> Result<(), String> {
    let src_dir = Path::new(&src);
    let dest_dir = Path::new(&dest);

    pipeline::run_import(src_dir, dest_dir);
    Ok(()) // TODO: handle errors form run_import
}

#[tauri::command]
pub fn pick_dir(app: tauri::AppHandle) -> Option<String> {
    app.dialog()
        .file()
        .blocking_pick_folder()
        .map(|path| path.to_string())
}
