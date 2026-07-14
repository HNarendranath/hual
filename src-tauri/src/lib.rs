use hual::pipeline;
use hual::pipeline::{list_photos as list_photos_hual, open_db, PhotoRow};
use hual::thumbnail;
use std::path::Path;
use tauri::generate_handler;
use tauri_plugin_dialog::DialogExt;

#[tauri::command]
fn get_thumbnail(path: String) -> Result<Vec<u8>, String> {
    thumbnail::extract_thumbnail(Path::new(&path)).map_err(|e| e.to_string())
}

#[tauri::command]
fn list_photos(db_path: String) -> Result<Vec<PhotoRow>, String> {
    let conn = open_db(Path::new(&db_path)).map_err(|e| e.to_string())?;
    list_photos_hual(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
fn import_photos(src: String, dest: String) -> Result<(), String> {
    let src_dir = Path::new(&src);
    let dest_dir = Path::new(&dest);

    pipeline::run_import(src_dir, dest_dir);
    Ok(()) // TODO: handle errors form run_import
}

#[tauri::command]
fn pick_dir(app: tauri::AppHandle) -> Option<String> {
    app.dialog()
        .file()
        .blocking_pick_folder()
        .map(|path| path.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_photos,
            get_thumbnail,
            import_photos,
            pick_dir,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
