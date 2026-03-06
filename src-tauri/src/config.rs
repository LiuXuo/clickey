use serde::{Deserialize, Serialize};

// 与前端共享的默认配置（单一事实来源）
pub const DEFAULT_CONFIG_JSON: &str = include_str!("../../src/lib/shared/default-config.json");

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    pub app: AppSection,
    pub hotkeys: HotkeysConfig,
    #[serde(default)]
    pub nudge: NudgeConfig,
    #[serde(default)]
    pub mouse: MouseConfig,
    pub layers: Vec<Layer>,
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
    #[serde(default = "default_locale")]
    pub locale: String,
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
    pub trigger: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ControlHotkeys {
    pub cancel: String,
    pub undo: String,
    pub direct_click: String,
    pub switch_action: String,
    pub next_monitor: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NudgeConfig {
    pub step_px: u32,
}

impl Default for NudgeConfig {
    fn default() -> Self {
        Self { step_px: 5 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MouseConfig {
    #[serde(default = "default_mouse_smooth_move")]
    pub smooth_move: bool,
    #[serde(default = "default_mouse_move_duration_ms")]
    pub move_duration_ms: u32,
    #[serde(default = "default_mouse_move_step_ms")]
    pub move_step_ms: u32,
    #[serde(default = "default_mouse_press_duration_ms")]
    pub press_duration_ms: u32,
    #[serde(default = "default_mouse_landing_radius_px")]
    pub landing_radius_px: u32,
    #[serde(default = "default_mouse_duration_randomness")]
    pub duration_randomness: f64,
    #[serde(default = "default_mouse_step_randomness")]
    pub step_randomness: f64,
    #[serde(default = "default_mouse_distance_boost_px")]
    pub distance_boost_px: f64,
    #[serde(default = "default_mouse_duration_distance_boost")]
    pub duration_distance_boost: f64,
    #[serde(default = "default_mouse_step_distance_boost")]
    pub step_distance_boost: f64,
    #[serde(default = "default_mouse_curve_along_ratio")]
    pub curve_along_ratio: f64,
    #[serde(default = "default_mouse_curve_spread_ratio")]
    pub curve_spread_ratio: f64,
    #[serde(default = "default_mouse_jitter_ratio")]
    pub jitter_ratio: f64,
    #[serde(default = "default_mouse_adaptive_stride_base_px")]
    pub adaptive_stride_base_px: f64,
    #[serde(default = "default_mouse_adaptive_stride_distance_ratio")]
    pub adaptive_stride_distance_ratio: f64,
    #[serde(default = "default_mouse_adaptive_stride_max_px")]
    pub adaptive_stride_max_px: f64,
    #[serde(default = "default_mouse_extra_steps_max")]
    pub extra_steps_max: u32,
    #[serde(default = "default_mouse_max_steps")]
    pub max_steps: u32,
    #[serde(default = "default_mouse_max_step_sleep_ms")]
    pub max_step_sleep_ms: u32,
}

impl Default for MouseConfig {
    fn default() -> Self {
        Self {
            smooth_move: default_mouse_smooth_move(),
            move_duration_ms: default_mouse_move_duration_ms(),
            move_step_ms: default_mouse_move_step_ms(),
            press_duration_ms: default_mouse_press_duration_ms(),
            landing_radius_px: default_mouse_landing_radius_px(),
            duration_randomness: default_mouse_duration_randomness(),
            step_randomness: default_mouse_step_randomness(),
            distance_boost_px: default_mouse_distance_boost_px(),
            duration_distance_boost: default_mouse_duration_distance_boost(),
            step_distance_boost: default_mouse_step_distance_boost(),
            curve_along_ratio: default_mouse_curve_along_ratio(),
            curve_spread_ratio: default_mouse_curve_spread_ratio(),
            jitter_ratio: default_mouse_jitter_ratio(),
            adaptive_stride_base_px: default_mouse_adaptive_stride_base_px(),
            adaptive_stride_distance_ratio: default_mouse_adaptive_stride_distance_ratio(),
            adaptive_stride_max_px: default_mouse_adaptive_stride_max_px(),
            extra_steps_max: default_mouse_extra_steps_max(),
            max_steps: default_mouse_max_steps(),
            max_step_sleep_ms: default_mouse_max_step_sleep_ms(),
        }
    }
}

fn default_mouse_smooth_move() -> bool {
    true
}

fn default_mouse_move_duration_ms() -> u32 {
    120
}

fn default_mouse_move_step_ms() -> u32 {
    8
}

fn default_mouse_press_duration_ms() -> u32 {
    24
}

fn default_mouse_landing_radius_px() -> u32 {
    1
}

fn default_mouse_duration_randomness() -> f64 {
    0.24
}

fn default_mouse_step_randomness() -> f64 {
    0.22
}

fn default_mouse_distance_boost_px() -> f64 {
    1800.0
}

fn default_mouse_duration_distance_boost() -> f64 {
    0.28
}

fn default_mouse_step_distance_boost() -> f64 {
    0.42
}

fn default_mouse_curve_along_ratio() -> f64 {
    0.08
}

fn default_mouse_curve_spread_ratio() -> f64 {
    0.12
}

fn default_mouse_jitter_ratio() -> f64 {
    0.01
}

fn default_mouse_adaptive_stride_base_px() -> f64 {
    7.0
}

fn default_mouse_adaptive_stride_distance_ratio() -> f64 {
    0.026
}

fn default_mouse_adaptive_stride_max_px() -> f64 {
    42.0
}

fn default_mouse_extra_steps_max() -> u32 {
    6
}

fn default_mouse_max_steps() -> u32 {
    220
}

fn default_mouse_max_step_sleep_ms() -> u32 {
    24
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "mode", rename_all = "camelCase")]
pub enum Layer {
    Single {
        rows: u32,
        cols: u32,
        keys: Vec<String>,
    },
    Combo {
        stage0: GridStage,
        stage1: GridStage,
    },
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
    #[serde(default = "default_overlay_show_grid")]
    pub show_grid: bool,
    #[serde(default = "default_overlay_show_diagonals")]
    pub show_diagonals: bool,
    pub font: OverlayFont,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OverlayFont {
    pub family: String,
    pub size_px: u32,
}

fn default_overlay_show_grid() -> bool {
    true
}

fn default_overlay_show_diagonals() -> bool {
    true
}

fn default_locale() -> String {
    "zh-CN".to_string()
}

pub fn default_config() -> AppConfig {
    // 统一从默认 JSON 反序列化，保证结构一致
    let json = DEFAULT_CONFIG_JSON
        .strip_prefix('\u{FEFF}')
        .unwrap_or(DEFAULT_CONFIG_JSON);
    serde_json::from_str(json).expect("default-config.json should be valid AppConfig")
}
