use hual::cache::{L2Cache, LRUCache};
use hual::pipeline;
use hual::pipeline::{list_photos as list_photos_hual, open_db, PhotoRow};
use hual::thumbnail;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tauri::State;
use tauri_plugin_dialog::DialogExt;

#[tauri::command]
pub fn get_preview(
    path: String,
    l1: State<Mutex<LRUCache<String, Vec<u8>>>>,
) -> Result<Vec<u8>, String> {
    if let Some(bytes) = l1.lock().unwrap().get(&path) {
        log::info!("L1 hit: {path}");
        return Ok(bytes.clone());
    }
    log::info!("L1 miss: {path}");

    let bytes = thumbnail::extract_thumbnail(Path::new(&path)).map_err(|e| e.to_string())?;
    l1.lock().unwrap().put(path.clone(), bytes.clone());
    log::info!(
        "L1 populated: {path} ({} entries)",
        l1.lock().unwrap().len()
    );
    Ok(bytes)
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
