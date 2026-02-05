use tauri::{AppHandle, Manager};
use tauri_plugin_global_shortcut::GlobalShortcutExt;
use crate::AppState;
use enigo::{Enigo, Key, Keyboard, Settings, Direction};

pub struct ShortcutsManager {
    app_handle: AppHandle,
}

impl ShortcutsManager {
    pub fn new(app_handle: AppHandle) -> Self {
        Self { app_handle }
    }

    pub async fn register_shortcuts(&self) -> Result<(), String> {
        let state = self.app_handle.state::<AppState>();
        let settings = state.settings.read().await;
        let hotkeys = &settings.hotkeys;

        // Unregister all first to be safe
        let _ = self.app_handle.global_shortcut().unregister_all();

        if let Some(push) = &hotkeys.push {
             self.app_handle.global_shortcut().register(push.as_str())
                .map_err(|e| format!("Failed to register push shortcut: {}", e))?;
        }

        if let Some(paste) = &hotkeys.paste {
             self.app_handle.global_shortcut().register(paste.as_str())
                .map_err(|e| format!("Failed to register paste shortcut: {}", e))?;
        }
        
        if let Some(history) = &hotkeys.history {
             self.app_handle.global_shortcut().register(history.as_str())
                .map_err(|e| format!("Failed to register history shortcut: {}", e))?;
        }

        Ok(())
    }

    pub fn handle_shortcut_event(&self, shortcut_str: String) {
        let app_handle = self.app_handle.clone();
        
        tauri::async_runtime::spawn(async move {
            let state = app_handle.state::<AppState>();
            let settings = state.settings.read().await.clone();
            
            if Some(&shortcut_str) == settings.hotkeys.push.as_ref() {
                println!("Shortcut triggered: Push");
                let _ = state.sync_manager.manual_sync().await;
            } else if Some(&shortcut_str) == settings.hotkeys.paste.as_ref() {
                println!("Shortcut triggered: Paste");
                // TODO: Get latest clip from DB and paste it
                 if let Ok(mut enigo) = Enigo::new(&Settings::default()) {
                     // Simulate Ctrl+V
                     let _ = enigo.key(Key::Control, Direction::Press);
                     let _ = enigo.key(Key::Unicode('v'), Direction::Click);
                     let _ = enigo.key(Key::Control, Direction::Release);
                 }
            } else if Some(&shortcut_str) == settings.hotkeys.history.as_ref() {
                println!("Shortcut triggered: History");
                if let Some(window) = app_handle.get_webview_window("history") {
                    if window.is_visible().unwrap_or(false) {
                        let _ = window.hide();
                    } else {
                        // Center window and show
                        // Using tauri-plugin-positioner if available, or just center
                        let _ = window.center();
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
            }
        });
    }
}
