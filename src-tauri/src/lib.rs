mod config;

use config::{default_config, AppConfig, Layer};
use enigo::{Enigo, MouseButton, MouseControllable};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    fs,
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    time::Duration,
};
use tauri::{menu::Menu, menu::MenuItem, tray::TrayIconBuilder};
use tauri::{
    AppHandle, Emitter, EventTarget, Manager, PhysicalPosition, PhysicalSize, Position, Size,
    State, WebviewUrl, WebviewWindowBuilder,
};
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

#[derive(Debug, Clone)]
struct NudgeRepeat {
    key: String,
    stop: Arc<AtomicBool>,
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
    overlay_click_action: Mutex<Option<ClickAction>>,
    monitor_index: Mutex<usize>,
    nudge_repeat: Mutex<Option<NudgeRepeat>>,
}

const CONFIG_FILE_NAME: &str = "config.json";
const NUDGE_REPEAT_DELAY_MS: u64 = 250;
const NUDGE_REPEAT_INTERVAL_MS: u64 = 40;

fn config_path(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_config_dir()
        .map(|dir| dir.join(CONFIG_FILE_NAME))
        .map_err(|_| "unable to resolve app config directory".to_string())
}

fn load_config(app: &AppHandle) -> (AppConfig, bool) {
    let path = match config_path(app) {
        Ok(path) => path,
        Err(err) => {
            println!("[config] failed to resolve path: {}", err);
            return (default_config(), true);
        }
    };

    match fs::read_to_string(&path) {
        Ok(contents) => match serde_json::from_str::<AppConfig>(&contents) {
            Ok(config) => match validate_config(&config) {
                Ok(()) => (config, false),
                Err(err) => {
                    println!("[config] invalid config file, using default: {}", err);
                    (default_config(), true)
                }
            },
            Err(err) => {
                println!("[config] invalid config file, using default: {}", err);
                (default_config(), true)
            }
        },
        Err(_) => (default_config(), true),
    }
}

fn persist_config(app: &AppHandle, config: &AppConfig) -> Result<(), String> {
    let path = config_path(app)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let payload = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
    fs::write(&path, payload).map_err(|e| e.to_string())
}

fn set_state_config(state: &AppState, config: AppConfig) -> Result<(), String> {
    {
        let mut config_guard = state.config.lock().map_err(|_| "config lock poisoned")?;
        *config_guard = config.clone();
    }
    {
        let mut ids_guard = state
            .activation_ids
            .lock()
            .map_err(|_| "activation ids lock poisoned")?;
        *ids_guard = ActivationHotkeyIds::from_config(&config);
    }
    Ok(())
}

fn get_state_config(state: &AppState) -> Result<AppConfig, String> {
    state
        .config
        .lock()
        .map(|guard| guard.clone())
        .map_err(|_| "config lock poisoned".to_string())
}

fn validate_keys(keys: &[String], expected_len: usize, label: &str) -> Result<(), String> {
    if keys.len() != expected_len {
        return Err(format!(
            "{} expects {} keys but got {}",
            label,
            expected_len,
            keys.len()
        ));
    }
    if keys.iter().any(|key| key.trim().is_empty()) {
        return Err(format!("{} contains empty key labels", label));
    }
    Ok(())
}

fn validate_hotkey(value: &str, label: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        return Err(format!("{} hotkey is empty", label));
    }
    parse_shortcut(value)
        .map(|_| ())
        .ok_or_else(|| format!("{} hotkey is invalid: {}", label, value))
}

fn validate_config(config: &AppConfig) -> Result<(), String> {
    if config.presets.is_empty() {
        return Err("presets must not be empty".to_string());
    }

    validate_hotkey(&config.hotkeys.activation.left_click, "leftClick")?;
    validate_hotkey(&config.hotkeys.activation.right_click, "rightClick")?;
    validate_hotkey(&config.hotkeys.activation.middle_click, "middleClick")?;

    if config.hotkeys.controls.cancel.trim().is_empty() {
        return Err("cancel hotkey is empty".to_string());
    }
    if config.hotkeys.controls.undo.trim().is_empty() {
        return Err("undo hotkey is empty".to_string());
    }
    if config.hotkeys.controls.direct_click.trim().is_empty() {
        return Err("directClick hotkey is empty".to_string());
    }

    if config.nudge.step_px == 0 {
        return Err("nudge stepPx must be > 0".to_string());
    }

    if config.overlay.line_width_px == 0 {
        return Err("overlay lineWidthPx must be > 0".to_string());
    }
    if config.overlay.font.size_px == 0 {
        return Err("overlay font sizePx must be > 0".to_string());
    }

    let mut preset_ids = HashSet::new();
    for preset in &config.presets {
        if preset.id.trim().is_empty() {
            return Err("preset id must not be empty".to_string());
        }
        if !preset_ids.insert(preset.id.as_str()) {
            return Err(format!("duplicate preset id: {}", preset.id));
        }
        if preset.layers.is_empty() {
            return Err(format!("preset {} has no layers", preset.id));
        }

        for (layer_index, layer) in preset.layers.iter().enumerate() {
            match layer {
                Layer::Single { rows, cols, keys } => {
                    if *rows == 0 || *cols == 0 {
                        return Err(format!(
                            "preset {} layer {} has invalid grid size",
                            preset.id, layer_index
                        ));
                    }
                    let expected_len = (*rows as usize) * (*cols as usize);
                    validate_keys(
                        keys,
                        expected_len,
                        &format!("preset {} layer {}", preset.id, layer_index),
                    )?;
                }
                Layer::Combo { stage0, stage1 } => {
                    if stage0.rows == 0 || stage0.cols == 0 {
                        return Err(format!(
                            "preset {} layer {} stage0 has invalid grid size",
                            preset.id, layer_index
                        ));
                    }
                    if stage1.rows == 0 || stage1.cols == 0 {
                        return Err(format!(
                            "preset {} layer {} stage1 has invalid grid size",
                            preset.id, layer_index
                        ));
                    }
                    let expected0 = (stage0.rows as usize) * (stage0.cols as usize);
                    validate_keys(
                        &stage0.keys,
                        expected0,
                        &format!("preset {} layer {} stage0", preset.id, layer_index),
                    )?;
                    let expected1 = (stage1.rows as usize) * (stage1.cols as usize);
                    validate_keys(
                        &stage1.keys,
                        expected1,
                        &format!("preset {} layer {} stage1", preset.id, layer_index),
                    )?;
                }
            }
        }
    }

    if !preset_ids.contains(config.active_preset_id.as_str()) {
        return Err(format!(
            "activePresetId {} does not match any preset",
            config.active_preset_id
        ));
    }

    Ok(())
}

#[tauri::command]
fn apply_config(
    app: AppHandle,
    state: State<'_, AppState>,
    config: AppConfig,
) -> Result<(), String> {
    println!("[config] apply_config called");
    validate_config(&config)?;
    set_state_config(state.inner(), config.clone())?;
    register_activation_hotkeys(&app, state.inner(), &config)?;
    let overlay_active = state
        .overlay_active
        .lock()
        .map(|guard| *guard)
        .unwrap_or(false);
    if overlay_active {
        register_overlay_hotkeys(&app, state.inner(), &config)?;
    }
    persist_config(&app, &config)?;
    Ok(())
}

#[tauri::command]
fn get_config(state: State<'_, AppState>) -> Result<AppConfig, String> {
    get_state_config(state.inner())
}

#[tauri::command]
fn reset_config(app: AppHandle, state: State<'_, AppState>) -> Result<AppConfig, String> {
    let config = default_config();
    set_state_config(state.inner(), config.clone())?;
    register_activation_hotkeys(&app, state.inner(), &config)?;
    let overlay_active = state
        .overlay_active
        .lock()
        .map(|guard| *guard)
        .unwrap_or(false);
    if overlay_active {
        register_overlay_hotkeys(&app, state.inner(), &config)?;
    }
    persist_config(&app, &config)?;
    Ok(config)
}

#[tauri::command]
fn native_click(app: AppHandle, payload: NativeClickPayload) -> Result<(), String> {
    println!(
        "[native] click action={:?} x={} y={}",
        payload.button, payload.x, payload.y
    );
    perform_click(&payload)?;
    hide_overlay(&app, app.state::<AppState>().inner());

    Ok(())
}

#[tauri::command]
fn close_overlay(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    hide_overlay(&app, state.inner());
    Ok(())
}


#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let default_cfg = default_config();
    let _ = parse_shortcut_or_panic("leftClick", &default_cfg.hotkeys.activation.left_click);
    let _ = parse_shortcut_or_panic("rightClick", &default_cfg.hotkeys.activation.right_click);
    let _ = parse_shortcut_or_panic("middleClick", &default_cfg.hotkeys.activation.middle_click);
    let activation_ids = ActivationHotkeyIds::from_config(&default_cfg);

    let global_shortcut_plugin = tauri_plugin_global_shortcut::Builder::new()
        .with_handler(|app, shortcut, event| {
            let state = app.state::<AppState>();

            if event.state == ShortcutState::Released {
                let overlay_active = state
                    .overlay_active
                    .lock()
                    .map(|guard| *guard)
                    .unwrap_or(false);
                if !overlay_active {
                    return;
                }

                let overlay_key = state
                    .overlay_key_map
                    .lock()
                    .ok()
                    .and_then(|map| map.get(&shortcut.id()).cloned());

                if let Some(key) = overlay_key {
                    if is_nudge_key(&key) {
                        stop_nudge_repeat(state.inner());
                    }
                }
                return;
            }

            if event.state != ShortcutState::Pressed {
                return;
            }
            println!("[shortcut] pressed id={}", shortcut.id());

            let action = {
                let ids = state
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

            let overlay_active = state
                .overlay_active
                .lock()
                .map(|guard| *guard)
                .unwrap_or(false);
            if !overlay_active {
                println!("[shortcut] overlay inactive; ignoring");
                return;
            }

            let overlay_key = state
                .overlay_key_map
                .lock()
                .ok()
                .and_then(|map| map.get(&shortcut.id()).cloned());

            if let Some(key) = overlay_key {
                println!("[shortcut] overlay key={}", key);
                if is_next_monitor_key(&key) {
                    switch_monitor(app);
                    return;
                }
                if is_nudge_key(&key) && is_nudge_repeat_active(state.inner(), &key) {
                    return;
                }
                let _ = app.emit_to(
                    EventTarget::webview_window("overlay"),
                    "native:key",
                    NativeKeyPayload { key: key.clone() },
                );
                if is_nudge_key(&key) {
                    start_nudge_repeat(app.clone(), state.inner(), key);
                }
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
            overlay_click_action: Mutex::new(None),
            monitor_index: Mutex::new(0),
            nudge_repeat: Mutex::new(None),
        })
        .plugin(tauri_plugin_opener::init())
        .plugin(global_shortcut_plugin)
        .invoke_handler(tauri::generate_handler![
            apply_config,
            get_config,
            reset_config,
            native_click,
            close_overlay
        ])
        .setup(|app| {
            let handle = app.handle();
            create_overlay_window(&handle)?;
            register_tray(&handle)?;
            let state = app.state::<AppState>();
            let (mut config, mut should_persist) = load_config(&handle);
            if let Err(err) = set_state_config(state.inner(), config.clone()) {
                println!("[config] failed to set state config: {}", err);
                config = default_config();
                let _ = set_state_config(state.inner(), config.clone());
                should_persist = true;
            }

            if let Err(err) =
                register_activation_hotkeys(&handle, app.state::<AppState>().inner(), &config)
            {
                println!("[hotkeys] activation register failed: {}", err);
                let fallback = default_config();
                let _ = set_state_config(state.inner(), fallback.clone());
                register_activation_hotkeys(&handle, app.state::<AppState>().inner(), &fallback)?;
                config = fallback;
                should_persist = true;
            }
            if should_persist {
                let _ = persist_config(&handle, &config);
            }
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

    let mut builder = TrayIconBuilder::new()
        .menu(&menu)
        .on_menu_event(move |app, event| {
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

fn show_overlay_window(app: &AppHandle, region: &Region) {
    if let Some(window) = app.get_webview_window("overlay") {
        let target_pos = PhysicalPosition::new(region.x as i32, region.y as i32);
        let target_size = PhysicalSize::new(
            region.width.max(1.0) as u32,
            region.height.max(1.0) as u32,
        );

        let _ = window.set_position(Position::Physical(target_pos));
        let _ = window.set_size(Size::Physical(target_size));
        let _ = window.set_ignore_cursor_events(true);
        let _ = window.set_focusable(false);
        let _ = window.show();

        // Align client area to the target region to avoid DWM offset on Windows.
        if let (Ok(outer_pos), Ok(inner_pos)) = (window.outer_position(), window.inner_position()) {
            let delta_x = inner_pos.x - outer_pos.x;
            let delta_y = inner_pos.y - outer_pos.y;
            if delta_x != 0 || delta_y != 0 {
                let adjusted = PhysicalPosition::new(
                    target_pos.x - delta_x,
                    target_pos.y - delta_y,
                );
                let _ = window.set_position(Position::Physical(adjusted));
                println!(
                    "[overlay] adjusted position by ({}, {})",
                    delta_x, delta_y
                );
            }
        }

        if let (Ok(outer_size), Ok(inner_size)) = (window.outer_size(), window.inner_size()) {
            let border_w = outer_size.width as i32 - inner_size.width as i32;
            let border_h = outer_size.height as i32 - inner_size.height as i32;
            if border_w != 0 || border_h != 0 {
                let adjusted_size = PhysicalSize::new(
                    (target_size.width as i32 + border_w).max(1) as u32,
                    (target_size.height as i32 + border_h).max(1) as u32,
                );
                let _ = window.set_size(Size::Physical(adjusted_size));
                println!(
                    "[overlay] adjusted size by ({}, {})",
                    border_w, border_h
                );
            }
        }
    }
}

fn available_monitors(app: &AppHandle) -> Vec<tauri::Monitor> {
    app.available_monitors().unwrap_or_default()
}

fn monitor_region(monitor: &tauri::Monitor) -> Region {
    let pos = monitor.position();
    let size = monitor.size();
    Region {
        x: pos.x as f64,
        y: pos.y as f64,
        width: size.width.max(1) as f64,
        height: size.height.max(1) as f64,
    }
}

fn primary_monitor_index(
    app: &AppHandle,
    monitors: &[tauri::Monitor],
) -> usize {
    if let Ok(Some(primary)) = app.primary_monitor() {
        let primary_pos = primary.position();
        let primary_size = primary.size();
        if let Some((index, _)) = monitors.iter().enumerate().find(|(_, monitor)| {
            let pos = monitor.position();
            let size = monitor.size();
            pos.x == primary_pos.x
                && pos.y == primary_pos.y
                && size.width == primary_size.width
                && size.height == primary_size.height
        }) {
            return index;
        }
    }
    0
}

fn set_start_monitor_index(
    app: &AppHandle,
    state: &AppState,
    monitors: &[tauri::Monitor],
) -> usize {
    if monitors.is_empty() {
        return 0;
    }
    let index = primary_monitor_index(app, monitors);
    if let Ok(mut guard) = state.monitor_index.lock() {
        *guard = index;
    }
    index
}

fn next_monitor_region(app: &AppHandle, state: &AppState) -> Region {
    let monitors = available_monitors(app);
    if monitors.is_empty() {
        return compute_virtual_region(app);
    }

    let next_index = if let Ok(mut guard) = state.monitor_index.lock() {
        let next = (*guard + 1) % monitors.len();
        *guard = next;
        next
    } else {
        0
    };

    monitor_region(&monitors[next_index])
}

fn switch_monitor(app: &AppHandle) {
    let state = app.state::<AppState>();
    let region = next_monitor_region(app, state.inner());
    let config = state
        .config
        .lock()
        .map(|guard| guard.clone())
        .unwrap_or_else(|_| default_config());
    let action = state
        .overlay_click_action
        .lock()
        .ok()
        .and_then(|guard| guard.clone())
        .unwrap_or(ClickAction::Left);

    println!(
        "[overlay] switch monitor region=({}, {}, {}, {})",
        region.x, region.y, region.width, region.height
    );

    show_overlay_window(app, &region);

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
}

fn trigger_overlay(app: &AppHandle, action: ClickAction) {
    let state = app.state::<AppState>();
    let config = state
        .config
        .lock()
        .map(|guard| guard.clone())
        .unwrap_or_else(|_| default_config());
    let monitors = available_monitors(app);
    let index = set_start_monitor_index(app, state.inner(), &monitors);
    let region = if monitors.is_empty() {
        compute_virtual_region(app)
    } else {
        monitor_region(&monitors[index])
    };

    if let Ok(mut active) = state.overlay_active.lock() {
        *active = true;
    }
    if let Ok(mut stored_action) = state.overlay_click_action.lock() {
        *stored_action = Some(action.clone());
    }
    println!(
        "[overlay] show action={:?} region=({}, {}, {}, {})",
        action, region.x, region.y, region.width, region.height
    );

    show_overlay_window(app, &region);

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

fn hide_overlay(app: &AppHandle, state: &AppState) {
    if let Some(window) = app.get_webview_window("overlay") {
        let _ = window.hide();
    }
    let _ = unregister_overlay_hotkeys(app, state);
    stop_nudge_repeat(state);
    if let Ok(mut active) = state.overlay_active.lock() {
        *active = false;
    }
    if let Ok(mut action) = state.overlay_click_action.lock() {
        *action = None;
    }
    println!("[overlay] hidden");
}

fn perform_click(payload: &NativeClickPayload) -> Result<(), String> {
    let mut enigo = Enigo::new();
    let x = payload.x.round() as i32;
    let y = payload.y.round() as i32;
    enigo.mouse_move_to(x, y);

    let button = match payload.button {
        ClickAction::Left => MouseButton::Left,
        ClickAction::Right => MouseButton::Right,
        ClickAction::Middle => MouseButton::Middle,
    };
    enigo.mouse_click(button);
    Ok(())
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
                Layer::Single {
                    keys: layer_keys, ..
                } => {
                    keys.extend(layer_keys.iter().cloned());
                }
                Layer::Combo { stage0, stage1 } => {
                    keys.extend(stage0.keys.iter().cloned());
                    keys.extend(stage1.keys.iter().cloned());
                }
            }
        }
    }

    keys.push(config.hotkeys.controls.cancel.clone());
    keys.push(config.hotkeys.controls.undo.clone());
    keys.push(config.hotkeys.controls.direct_click.clone());
    keys.extend([
        "Tab",
        "ArrowLeft",
        "ArrowRight",
        "ArrowUp",
        "ArrowDown",
    ].into_iter().map(String::from));

    keys.retain(|key| !key.trim().is_empty());

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
        if let Some(shortcut) = resolve_shortcut(&key) {
            key_map.insert(shortcut.id(), key.clone());
            shortcuts.push(shortcut);
        } else {
            println!("[hotkeys] invalid overlay key: {}", key);
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

fn is_next_monitor_key(value: &str) -> bool {
    value.eq_ignore_ascii_case("tab")
}

fn is_nudge_key(value: &str) -> bool {
    matches!(
        value.to_ascii_lowercase().as_str(),
        "left"
            | "arrowleft"
            | "right"
            | "arrowright"
            | "up"
            | "arrowup"
            | "down"
            | "arrowdown"
    )
}

fn is_nudge_repeat_active(state: &AppState, key: &str) -> bool {
    if let Ok(guard) = state.nudge_repeat.lock() {
        if let Some(active) = guard.as_ref() {
            return active.key.eq_ignore_ascii_case(key);
        }
    }
    false
}

fn stop_nudge_repeat(state: &AppState) {
    if let Ok(mut guard) = state.nudge_repeat.lock() {
        if let Some(active) = guard.take() {
            active.stop.store(true, Ordering::SeqCst);
        }
    }
}

fn start_nudge_repeat(app: AppHandle, state: &AppState, key: String) {
    let stop = Arc::new(AtomicBool::new(false));
    {
        let mut guard = match state.nudge_repeat.lock() {
            Ok(guard) => guard,
            Err(_) => return,
        };

        if let Some(active) = guard.as_ref() {
            if active.key.eq_ignore_ascii_case(&key) {
                return;
            }
            active.stop.store(true, Ordering::SeqCst);
        }

        *guard = Some(NudgeRepeat {
            key: key.clone(),
            stop: Arc::clone(&stop),
        });
    }

    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(NUDGE_REPEAT_DELAY_MS));
        loop {
            if stop.load(Ordering::SeqCst) {
                break;
            }

            let overlay_active = app
                .state::<AppState>()
                .overlay_active
                .lock()
                .map(|guard| *guard)
                .unwrap_or(false);
            if !overlay_active {
                break;
            }

            let _ = app.emit_to(
                EventTarget::webview_window("overlay"),
                "native:key",
                NativeKeyPayload { key: key.clone() },
            );

            std::thread::sleep(Duration::from_millis(NUDGE_REPEAT_INTERVAL_MS));
        }
    });
}

fn resolve_shortcut(value: &str) -> Option<Shortcut> {
    if let Some(shortcut) = parse_shortcut(value) {
        return Some(shortcut);
    }

    let fallbacks = [
        ("Esc", "Escape"),
        ("Escape", "Esc"),
        ("Space", "Spacebar"),
        ("Spacebar", "Space"),
        ("Left", "ArrowLeft"),
        ("ArrowLeft", "Left"),
        ("Right", "ArrowRight"),
        ("ArrowRight", "Right"),
        ("Up", "ArrowUp"),
        ("ArrowUp", "Up"),
        ("Down", "ArrowDown"),
        ("ArrowDown", "Down"),
    ];

    for (from, to) in fallbacks {
        if let Some(candidate) = replace_token(value, from, to) {
            if let Some(shortcut) = parse_shortcut(&candidate) {
                return Some(shortcut);
            }
        }
    }

    None
}

fn replace_token(value: &str, from: &str, to: &str) -> Option<String> {
    let mut replaced = false;
    let parts: Vec<String> = value
        .split('+')
        .map(|part| {
            let trimmed = part.trim();
            if trimmed.eq_ignore_ascii_case(from) {
                replaced = true;
                to.to_string()
            } else {
                trimmed.to_string()
            }
        })
        .collect();

    if replaced {
        Some(parts.join("+"))
    } else {
        None
    }
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


