mod app;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut builder = tauri::Builder::default()
        .manage(app::app_lock::AppLockSessionState::default())
        .manage(app::editor::FilePassphraseStore::default());

    #[cfg(desktop)]
    {
        builder = builder.plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            let _ = app::windows::show_main_window(app);
        }));
    }

    builder
        .setup(|app| {
            app::app_lock::setup(app.handle(), app.state())?;
            app::menu::setup(app)?;
            app::tray::setup(app)?;
            Ok(())
        })
        .on_menu_event(app::menu::handle_menu_event)
        .on_window_event(app::windows::handle_close_requested)
        .plugin(tauri_plugin_biometry::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            app::app_lock::app_lock_get_settings,
            app::app_lock::app_lock_save_settings,
            app::app_lock::app_lock_lock_session,
            app::app_lock::app_lock_unlock_session,
            app::editor::editor_pick_folder,
            app::editor::editor_pick_file,
            app::editor::editor_list_directory,
            app::editor::editor_inspect_file,
            app::editor::editor_read_file,
            app::editor::editor_write_file,
            app::editor::editor_create_file,
            app::editor::editor_create_directory,
            app::editor::editor_delete_path,
            app::editor::editor_rename_path,
            app::editor::editor_convert_to_encrypted,
            app::editor::editor_convert_to_custom_encrypted,
            app::editor::editor_convert_custom_to_default_encrypted,
            app::editor::editor_unlock_encrypted_file,
            app::editor::editor_convert_to_plain,
            app::editor::editor_get_settings,
            app::editor::editor_save_encryption_passphrase,
            app::security::security_get_settings,
            app::windows::window_show_preferences,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app_handle, event| {
            app::windows::handle_run_event(app_handle, event);
        });
}
