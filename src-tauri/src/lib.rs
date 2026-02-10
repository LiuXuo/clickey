mod config;

use config::{default_config, AppConfig, Layer};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    sync::Mutex,
};
use tauri::{
    AppHandle, Emitter, EventTarget, Manager, PhysicalPosition, PhysicalSize, Position, Size,
    State, WebviewUrl, WebviewWindowBuilder,
};
use tauri::{menu::Menu, menu::MenuItem, tray::TrayIconBuilder};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Region {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
enum ClickAction {
    Left,
    Right,
    Middle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OverlayActivatePayload {
    region: Region,
    config: AppConfig,
    click_action: ClickAction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct NativeClickPayload {
    x: f64,
    y: f64,
    button: ClickAction,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct NativeKeyPayload {
    key: String,
}

#[derive(Debug, Default, Clone)]
struct ActivationHotkeyIds {
    left: Option<u32>,
    right: Option<u32>,
    middle: Option<u32>,
}

impl ActivationHotkeyIds {
    fn from_config(config: &AppConfig) -> Self {
        Self {
            left: parse_hotkey_id(&config.hotkeys.activation.left_click),
            right: parse_hotkey_id(&config.hotkeys.activation.right_click),
            middle: parse_hotkey_id(&config.hotkeys.activation.middle_click),
        }
    }

    fn action_for_id(&self, id: u32) -> Option<ClickAction> {
        if self.left == Some(id) {
            return Some(ClickAction::Left);
        }
        if self.right == Some(id) {
            return Some(ClickAction::Right);
        }
        if self.middle == Some(id) {
            return Some(ClickAction::Middle);
        }
        None
    }
}

struct AppState {
    config: Mutex<AppConfig>,
    activation_ids: Mutex<ActivationHotkeyIds>,
    activation_shortcuts: Mutex<Vec<Shortcut>>,
    overlay_shortcuts: Mutex<Vec<Shortcut>>,
    overlay_key_map: Mutex<HashMap<u32, String>>,
    overlay_active: Mutex<bool>,
}

#[tauri::command]
fn apply_config(app: AppHandle, state: State<'_, AppState>, config: AppConfig) -> Result<(), String> {
    println!("[config] apply_config called");
    let activation_ids = ActivationHotkeyIds::from_config(&config);
    {
        let mut config_guard = state.config.lock().map_err(|_| "config lock poisoned")?;
        *config_guard = config.clone();
    }
    {
        let mut ids_guard = state
            .activation_ids
            .lock()
            .map_err(|_| "activation ids lock poisoned")?;
        *ids_guard = activation_ids;
    }

    register_activation_hotkeys(&app, state.inner(), &config)?;
    let overlay_active = state
        .overlay_active
        .lock()
        .map(|guard| *guard)
        .unwrap_or(false);
    if overlay_active {
        register_overlay_hotkeys(&app, state.inner(), &config)?;
    }
    Ok(())
}

#[tauri::command]
fn native_click(app: AppHandle, payload: NativeClickPayload) -> Result<(), String> {
    // 真实鼠标调用的占位实现，后续替换为系统级鼠标操作
    println!(
        "[stub] native click: action={:?}, x={}, y={}",
        payload.button, payload.x, payload.y
    );

    if let Some(window) = app.get_webview_window("overlay") {
        let _ = window.hide();
    }
    let _ = unregister_overlay_hotkeys(&app, app.state::<AppState>().inner());
    if let Ok(mut active) = app.state::<AppState>().overlay_active.lock() {
        *active = false;
    }
    println!("[overlay] hidden after native click");

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let default_cfg = default_config();
    let _ = parse_shortcut_or_panic(
        "leftClick",
        &default_cfg.hotkeys.activation.left_click,
    );
    let _ = parse_shortcut_or_panic(
        "rightClick",
        &default_cfg.hotkeys.activation.right_click,
    );
    let _ = parse_shortcut_or_panic(
        "middleClick",
        &default_cfg.hotkeys.activation.middle_click,
    );
    let activation_ids = ActivationHotkeyIds::from_config(&default_cfg);

    let global_shortcut_plugin = tauri_plugin_global_shortcut::Builder::new()
        .with_handler(|app, shortcut, event| {
            if event.state != ShortcutState::Pressed {
                return;
            }
            println!("[shortcut] pressed id={}", shortcut.id());

            let action = {
                let ids = app
                    .state::<AppState>()
                    .activation_ids
                    .lock()
                    .map(|guard| guard.clone())
                    .unwrap_or_default();
                ids.action_for_id(shortcut.id())
            };

            if let Some(action) = action {
                println!("[shortcut] activation action={:?}", action);
                trigger_overlay(app, action);
                return;
            }

            let overlay_active = app
                .state::<AppState>()
                .overlay_active
                .lock()
                .map(|guard| *guard)
                .unwrap_or(false);
            if !overlay_active {
                println!("[shortcut] overlay inactive; ignoring");
                return;
            }

            let overlay_key = app
                .state::<AppState>()
                .overlay_key_map
                .lock()
                .ok()
                .and_then(|map| map.get(&shortcut.id()).cloned());

            if let Some(key) = overlay_key {
                println!("[shortcut] overlay key={}", key);
                let _ = app.emit_to(
                    EventTarget::webview_window("overlay"),
                    "native:key",
                    NativeKeyPayload { key },
                );
            } else {
                println!("[shortcut] no overlay key mapping for id={}", shortcut.id());
            }
        })
        .build();

    tauri::Builder::default()
        .manage(AppState {
            config: Mutex::new(default_cfg),
            activation_ids: Mutex::new(activation_ids),
            activation_shortcuts: Mutex::new(Vec::new()),
            overlay_shortcuts: Mutex::new(Vec::new()),
            overlay_key_map: Mutex::new(HashMap::new()),
            overlay_active: Mutex::new(false),
        })
        .plugin(tauri_plugin_opener::init())
        .plugin(global_shortcut_plugin)
        .invoke_handler(tauri::generate_handler![apply_config, native_click])
        .setup(|app| {
            let handle = app.handle();
            create_overlay_window(&handle)?;
            register_tray(&handle)?;
            let config = app
                .state::<AppState>()
                .config
                .lock()
                .map(|guard| guard.clone())
                .unwrap_or_else(|_| default_config());
            register_activation_hotkeys(&handle, app.state::<AppState>().inner(), &config)?;
            println!("[startup] activation hotkeys registered");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn register_tray(app: &AppHandle) -> tauri::Result<()> {
    let settings_item = MenuItem::with_id(app, "settings", "Settings...", true, None::<&str>)?;
    let settings_id = settings_item.id().clone();
    let menu = Menu::with_items(app, &[&settings_item])?;

    let mut builder = TrayIconBuilder::new().menu(&menu).on_menu_event(move |app, event| {
        if event.id == settings_id {
            show_settings(app);
        }
    });

    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }

    builder.build(app)?;
    Ok(())
}

fn show_settings(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("settings") {
        let _ = window.show();
        let _ = window.set_focus();
    }
}

fn create_overlay_window(app: &AppHandle) -> tauri::Result<()> {
    let overlay = WebviewWindowBuilder::new(app, "overlay", WebviewUrl::App("overlay".into()))
        .transparent(true)
        .decorations(false)
        .resizable(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .visible(false)
        .build()?;

    overlay.set_ignore_cursor_events(true)?;
    overlay.set_focusable(false)?;

    Ok(())
}

fn trigger_overlay(app: &AppHandle, action: ClickAction) {
    let region = compute_virtual_region(app);
    let config = app
        .state::<AppState>()
        .config
        .lock()
        .map(|guard| guard.clone())
        .unwrap_or_else(|_| default_config());
    if let Ok(mut active) = app.state::<AppState>().overlay_active.lock() {
        *active = true;
    }
    println!(
        "[overlay] show action={:?} region=({}, {}, {}, {})",
        action, region.x, region.y, region.width, region.height
    );

    if let Some(window) = app.get_webview_window("overlay") {
        // 将 Overlay 定位到“虚拟屏幕”范围内，保证多显示器一致
        let _ = window.set_position(Position::Physical(PhysicalPosition::new(
            region.x as i32,
            region.y as i32,
        )));
        let _ = window.set_size(Size::Physical(PhysicalSize::new(
            region.width.max(1.0) as u32,
            region.height.max(1.0) as u32,
        )));
        let _ = window.set_ignore_cursor_events(true);
        let _ = window.set_focusable(false);
        let _ = window.show();
    }

    let config_for_keys = config.clone();
    let payload = OverlayActivatePayload {
        region,
        config,
        click_action: action,
    };

    let _ = app.emit_to(
        EventTarget::webview_window("overlay"),
        "overlay:activate",
        payload,
    );

    let app_handle = app.clone();
    std::thread::spawn(move || {
        if let Err(err) = register_overlay_hotkeys(
            &app_handle,
            app_handle.state::<AppState>().inner(),
            &config_for_keys,
        ) {
            println!("[hotkeys] overlay register failed: {}", err);
        }
    });
}

// stub key sequence removed; we only advance on real input

fn compute_virtual_region(app: &AppHandle) -> Region {
    let monitors = app.available_monitors().unwrap_or_default();
    if monitors.is_empty() {
        // 无法获取屏幕信息时的保底区域
        return Region {
            x: 0.0,
            y: 0.0,
            width: 1920.0,
            height: 1080.0,
        };
    }

    let mut min_x = i32::MAX;
    let mut min_y = i32::MAX;
    let mut max_x = i32::MIN;
    let mut max_y = i32::MIN;

    for monitor in monitors {
        let pos = monitor.position();
        let size = monitor.size();
        min_x = min_x.min(pos.x);
        min_y = min_y.min(pos.y);
        max_x = max_x.max(pos.x + size.width as i32);
        max_y = max_y.max(pos.y + size.height as i32);
    }

    Region {
        x: min_x as f64,
        y: min_y as f64,
        width: (max_x - min_x).max(1) as f64,
        height: (max_y - min_y).max(1) as f64,
    }
}

fn collect_overlay_keys(config: &AppConfig) -> Vec<String> {
    let preset = config
        .presets
        .iter()
        .find(|candidate| candidate.id == config.active_preset_id);
    let mut keys = Vec::new();
    if let Some(preset) = preset {
        for layer in &preset.layers {
            match layer {
                Layer::Single { keys: layer_keys, .. } => {
                    keys.extend(layer_keys.iter().cloned());
                }
                Layer::Combo { stage0, stage1 } => {
                    keys.extend(stage0.keys.iter().cloned());
                    keys.extend(stage1.keys.iter().cloned());
                }
            }
        }
    }

    let mut seen = HashSet::new();
    keys.retain(|key| seen.insert(key.to_lowercase()));
    keys
}

fn register_activation_hotkeys(
    app: &AppHandle,
    state: &AppState,
    config: &AppConfig,
) -> Result<(), String> {
    // ?????????????????
    let mut shortcuts = Vec::new();
    if let Some(shortcut) = parse_shortcut(&config.hotkeys.activation.left_click) {
        shortcuts.push(shortcut);
    }
    if let Some(shortcut) = parse_shortcut(&config.hotkeys.activation.right_click) {
        shortcuts.push(shortcut);
    }
    if let Some(shortcut) = parse_shortcut(&config.hotkeys.activation.middle_click) {
        shortcuts.push(shortcut);
    }

    if shortcuts.is_empty() {
        return Err("no valid activation hotkeys to register".to_string());
    }

    let shortcut_manager = app.global_shortcut();
    if let Ok(previous) = state.activation_shortcuts.lock().map(|guard| guard.clone()) {
        if !previous.is_empty() {
            let _ = shortcut_manager.unregister_multiple(previous);
        }
    }

    shortcut_manager
        .register_multiple(shortcuts.clone())
        .map_err(|e| e.to_string())?;

    if let Ok(mut guard) = state.activation_shortcuts.lock() {
        *guard = shortcuts;
    }
    println!("[hotkeys] activation registered");

    Ok(())
}

fn register_overlay_hotkeys(
    app: &AppHandle,
    state: &AppState,
    config: &AppConfig,
) -> Result<(), String> {
    let keys = collect_overlay_keys(config);
    let shortcut_manager = app.global_shortcut();

    if let Ok(previous) = state.overlay_shortcuts.lock().map(|guard| guard.clone()) {
        if !previous.is_empty() {
            let _ = shortcut_manager.unregister_multiple(previous);
        }
    }

    if keys.is_empty() {
        if let Ok(mut map) = state.overlay_key_map.lock() {
            map.clear();
        }
        if let Ok(mut guard) = state.overlay_shortcuts.lock() {
            guard.clear();
        }
        println!("[hotkeys] overlay keys empty");
        return Ok(());
    }

    let mut key_map = HashMap::new();
    let mut shortcuts = Vec::new();
    for key in keys {
        if let Some(shortcut) = parse_shortcut(&key) {
            key_map.insert(shortcut.id(), key.clone());
            shortcuts.push(shortcut);
        }
    }

    if shortcuts.is_empty() {
        println!("[hotkeys] overlay keys had no valid shortcuts");
        return Ok(());
    }

    shortcut_manager
        .register_multiple(shortcuts.clone())
        .map_err(|e| e.to_string())?;

    if let Ok(mut guard) = state.overlay_shortcuts.lock() {
        *guard = shortcuts;
    }
    if let Ok(mut guard) = state.overlay_key_map.lock() {
        *guard = key_map;
    }
    println!("[hotkeys] overlay registered");

    Ok(())
}

fn unregister_overlay_hotkeys(app: &AppHandle, state: &AppState) -> Result<(), String> {
    let shortcut_manager = app.global_shortcut();
    if let Ok(previous) = state.overlay_shortcuts.lock().map(|guard| guard.clone()) {
        if !previous.is_empty() {
            let _ = shortcut_manager.unregister_multiple(previous);
        }
    }
    if let Ok(mut guard) = state.overlay_shortcuts.lock() {
        guard.clear();
    }
    if let Ok(mut guard) = state.overlay_key_map.lock() {
        guard.clear();
    }
    println!("[hotkeys] overlay unregistered");
    Ok(())
}
fn parse_hotkey_id(value: &str) -> Option<u32> {
    parse_shortcut(value).map(|shortcut| shortcut.id())
}

fn parse_shortcut(value: &str) -> Option<Shortcut> {
    value.parse::<Shortcut>().ok()
}

fn parse_shortcut_or_panic(label: &str, value: &str) -> Shortcut {
    // 启动期热键校验，失败直接中断
    value.parse::<Shortcut>().unwrap_or_else(|_| {
        panic!("invalid hotkey for {label}: {value}");
    })
}
