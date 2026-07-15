use hual::cache::LRUCache;
use std::sync::Mutex;

mod commands;

const L1_CAPACITY: usize = 200;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(Mutex::new(LRUCache::<String, Vec<u8>>::new(L1_CAPACITY)))
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
            commands::list_photos,
            commands::get_preview,
            commands::import_photos,
            commands::pick_dir,
            commands::get_webp,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
