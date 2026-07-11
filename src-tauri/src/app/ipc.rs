//! 按域聚合 IPC commands，避免 lib.rs 膨胀

#[macro_export]
macro_rules! app_invoke_handler {
    () => {
        tauri::generate_handler![
            // database
            $crate::commands::database::database_resolve_path,
            $crate::commands::database::database_reset_dev,
            // crypto
            $crate::app::crypto::commands::crypto_encrypt_text,
            $crate::app::crypto::commands::crypto_decrypt_text,
            // code_snippets
            $crate::app::code_snippets::code_snippets_sync,
            $crate::app::code_snippets::code_snippets_set_expansion_paused,
            $crate::app::code_snippets::code_snippets_get_permissions,
            $crate::app::code_snippets::code_snippets_open_accessibility_settings,
            $crate::app::code_snippets::code_snippets_get_settings,
            $crate::app::code_snippets::code_snippets_save_settings,
            // palette（code_snippets 子能力）
            $crate::app::palette::code_snippets_list_palette_items,
            $crate::app::palette::code_snippets_insert_palette_item,
            $crate::app::palette::code_snippets_copy_palette_item,
            $crate::app::palette::code_snippets_hide_palette,
            // app_lock
            $crate::app::app_lock::app_lock_get_settings,
            $crate::app::app_lock::app_lock_save_settings,
            $crate::app::app_lock::app_lock_lock_session,
            $crate::app::app_lock::app_lock_unlock_session,
            // editor
            $crate::app::editor::editor_pick_folder,
            $crate::app::editor::editor_pick_file,
            $crate::app::editor::editor_list_directory,
            $crate::app::editor::editor_inspect_file,
            $crate::app::editor::editor_read_file,
            $crate::app::editor::editor_write_file,
            $crate::app::editor::editor_create_file,
            $crate::app::editor::editor_create_directory,
            $crate::app::editor::editor_delete_path,
            $crate::app::editor::editor_rename_path,
            $crate::app::editor::editor_convert_to_encrypted,
            $crate::app::editor::editor_convert_to_custom_encrypted,
            $crate::app::editor::editor_convert_custom_to_default_encrypted,
            $crate::app::editor::editor_unlock_encrypted_file,
            $crate::app::editor::editor_convert_to_plain,
            $crate::app::editor::editor_get_settings,
            $crate::app::editor::editor_save_encryption_passphrase,
            // security（app_lock 子能力）
            $crate::app::security::security_get_settings,
            // windows
            $crate::app::windows::window_show_preferences,
        ]
    };
}
