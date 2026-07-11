mod app;
mod commands;
mod database;
mod paths;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut builder = tauri::Builder::default()
        .manage(app::app_lock::AppLockSessionState::default())
        .manage(app::editor::FilePassphraseStore::default())
        .manage(app::code_snippets::SnippetRegistry::default());
    builder = app::clipboard_assistant::register_protocols(builder);

    #[cfg(desktop)]
    {
        builder = builder.plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            let _ = app::windows::show_main_window(app);
        }));
    }

    builder
        .setup(|app| {
            paths::ensure_profile_scaffold(app.handle())?;
            database::log_db_full_path(app.handle());
            app::app_lock::setup(app.handle(), app.state())?;
            app::code_snippets::setup(app.handle())?;
            app::clipboard_assistant::setup(app.handle())?;
            app::menu::setup(app)?;
            app::tray::setup(app)?;
            app::updates::schedule_startup_update_check(app.handle());
            Ok(())
        })
        .on_menu_event(app::menu::handle_menu_event)
        .on_window_event(app::windows::handle_close_requested)
        .plugin(tauri_plugin_biometry::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(database::plugin().build())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .invoke_handler(crate::app_invoke_handler!())
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app_handle, event| {
            app::windows::handle_run_event(app_handle, event);
        });
}
