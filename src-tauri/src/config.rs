use serde::{Deserialize, Serialize};

// 与前端共享的默认配置（单一事实来源）
pub const DEFAULT_CONFIG_JSON: &str = include_str!("../../src/lib/shared/default-config.json");

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    pub app: AppSection,
    pub hotkeys: HotkeysConfig,
    pub active_preset_id: String,
    pub presets: Vec<Preset>,
    pub overlay: OverlayConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        default_config()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSection {
    pub tray: TrayConfig,
    pub settings_window: SettingsWindowConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrayConfig {
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsWindowConfig {
    pub open_from_tray: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HotkeysConfig {
    pub activation: ActivationHotkeys,
    pub controls: ControlHotkeys,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivationHotkeys {
    pub left_click: String,
    pub right_click: String,
    pub middle_click: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ControlHotkeys {
    pub cancel: String,
    pub undo: String,
    pub direct_click: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Preset {
    pub id: String,
    pub name: String,
    pub layers: Vec<Layer>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "mode", rename_all = "camelCase")]
pub enum Layer {
    Single { rows: u32, cols: u32, keys: Vec<String> },
    Combo { stage0: GridStage, stage1: GridStage },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GridStage {
    pub rows: u32,
    pub cols: u32,
    pub keys: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OverlayConfig {
    pub alpha: u8,
    pub mask_color: String,
    pub line_color: String,
    pub text_color: String,
    pub line_width_px: u32,
    pub font: OverlayFont,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OverlayFont {
    pub family: String,
    pub size_px: u32,
}

pub fn default_config() -> AppConfig {
    // 统一从默认 JSON 反序列化，保证结构一致
    let json = DEFAULT_CONFIG_JSON
        .strip_prefix('\u{FEFF}')
        .unwrap_or(DEFAULT_CONFIG_JSON);
    serde_json::from_str(json)
        .expect("default-config.json should be valid AppConfig")
}
