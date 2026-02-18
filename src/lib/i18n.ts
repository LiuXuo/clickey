import { browser } from "$app/environment";
import { derived, get, writable } from "svelte/store";

export type Locale = "zh-CN" | "en-US";

const STORAGE_KEY = "clickey.locale";
export const DEFAULT_LOCALE: Locale = "zh-CN";

const zh = {
  "app.settings": "设置",
  "app.brand": "Clickey",
  "app.loading": "加载中...",
  "app.applying": "正在应用...",
  "app.apply": "应用",
  "app.resetting": "正在恢复...",
  "app.reset": "恢复默认",
  "language.label": "语言",
  "language.zh": "简体中文",
  "language.en": "English",
  "status.applied": "已应用",
  "status.reset": "已恢复默认",
  "presets.section": "预设",
  "presets.title": "预设管理",
  "presets.subtitle": "切换 / 复制 / 重命名",
  "presets.active": "当前预设",
  "presets.name": "预设名称",
  "presets.namePlaceholder": "预设名称",
  "presets.idLabel": "ID",
  "presets.duplicateActive": "复制当前",
  "presets.removeActive": "删除当前",
  "presets.list": "预设列表",
  "presets.activeSummary": "当前概览",
  "presets.noActive": "暂无当前预设。",
  "presets.layersCount": "{count} 层",
  "presets.copyName": "{name}（副本）",
  "layers.section": "层",
  "layers.title": "层编辑",
  "layers.subtitle": "行 / 列 / 键位",
  "layers.addSingle": "添加单层",
  "layers.addCombo": "添加组合",
  "layers.layerLabel": "第 {index} 层",
  "layers.type.single": "单层",
  "layers.type.combo": "组合",
  "layers.mode": "模式",
  "layers.moveUp": "上移",
  "layers.moveDown": "下移",
  "layers.remove": "删除",
  "layers.expectedKeys": "期望键数：",
  "layers.rows": "行",
  "layers.columns": "列",
  "layers.keys": "键位",
  "layers.keysHint": "键位（空格或逗号分隔）",
  "layers.autoFit": "自动补齐",
  "layers.currentExpected": "当前：{current} / 期望：{expected}",
  "layers.stage0": "阶段 0",
  "layers.stage1": "阶段 1",
  "hotkeys.section": "热键",
  "hotkeys.title": "按键绑定",
  "hotkeys.subtitle": "全局快捷键格式",
  "hotkeys.activation": "激活",
  "hotkeys.controls": "控制",
  "hotkeys.leftClick": "左键",
  "hotkeys.rightClick": "右键",
  "hotkeys.middleClick": "中键",
  "hotkeys.cancel": "取消",
  "hotkeys.undo": "撤销",
  "hotkeys.directClick": "直接点击",
  "overlay.section": "遮罩",
  "overlay.title": "遮罩样式",
  "overlay.subtitle": "遮罩 / 线条 / 字体",
  "overlay.alpha": "透明度 (0-255)",
  "overlay.lineWidth": "线宽",
  "overlay.fontSize": "字体大小",
  "overlay.maskColor": "遮罩颜色",
  "overlay.lineColor": "线条颜色",
  "overlay.textColor": "文字颜色",
  "overlay.fontFamily": "字体",
  "footer.note":
    "表单设置已覆盖预设、层、热键和遮罩样式。点击“应用”以保存并刷新运行时行为。",
  "errors.presetRequired": "至少需要一个预设。",
  "errors.removePresetConfirm": "删除预设“{name}”？",
  "errors.removeLayerConfirm": "删除第 {index} 层？",
  "errors.presetLayerRequired": "预设至少需要一层。",
  "errors.presetsEmpty": "预设列表为空。",
  "errors.presetIdEmpty": "预设 ID 不能为空。",
  "errors.presetNameEmpty": "预设名称不能为空。",
  "errors.duplicatePresetId": "重复的预设 ID：{id}",
  "errors.presetNoLayers": "预设 {id} 没有层。",
  "errors.layerGridInvalid": "预设 {id} 的第 {index} 层网格无效。",
  "errors.layerExpectedKeys": "预设 {id} 的第 {index} 层需要 {expected} 个键位。",
  "errors.stage0GridInvalid": "预设 {id} 的第 {index} 层阶段 0 网格无效。",
  "errors.stage1GridInvalid": "预设 {id} 的第 {index} 层阶段 1 网格无效。",
  "errors.stage0ExpectedKeys":
    "预设 {id} 的第 {index} 层阶段 0 需要 {expected} 个键位。",
  "errors.stage1ExpectedKeys":
    "预设 {id} 的第 {index} 层阶段 1 需要 {expected} 个键位。",
  "errors.activePresetMissing": "当前预设不存在。",
  "errors.leftHotkeyEmpty": "左键激活热键为空。",
  "errors.rightHotkeyEmpty": "右键激活热键为空。",
  "errors.middleHotkeyEmpty": "中键激活热键为空。",
  "errors.cancelHotkeyEmpty": "取消热键为空。",
  "errors.undoHotkeyEmpty": "撤销热键为空。",
  "errors.directClickHotkeyEmpty": "直接点击热键为空。",
  "errors.overlayLineWidth": "遮罩线宽必须大于 0。",
  "errors.overlayFontSize": "遮罩字体大小必须大于 0。",
} as const;

type TranslationKey = keyof typeof zh;
export type TranslationParams = Record<string, string | number>;

const en: Record<TranslationKey, string> = {
  "app.settings": "Settings",
  "app.brand": "Clickey",
  "app.loading": "Loading...",
  "app.applying": "Applying...",
  "app.apply": "Apply",
  "app.resetting": "Resetting...",
  "app.reset": "Reset to default",
  "language.label": "Language",
  "language.zh": "简体中文",
  "language.en": "English",
  "status.applied": "Applied",
  "status.reset": "Reset to default",
  "presets.section": "Presets",
  "presets.title": "Preset Management",
  "presets.subtitle": "Switch, duplicate, rename",
  "presets.active": "Active Preset",
  "presets.name": "Preset Name",
  "presets.namePlaceholder": "Preset name",
  "presets.idLabel": "ID",
  "presets.duplicateActive": "Duplicate active",
  "presets.removeActive": "Remove active",
  "presets.list": "Preset List",
  "presets.activeSummary": "Active Summary",
  "presets.noActive": "No active preset.",
  "presets.layersCount": "{count} layers",
  "presets.copyName": "{name} (copy)",
  "layers.section": "Layers",
  "layers.title": "Layer Editor",
  "layers.subtitle": "Rows, columns, keys",
  "layers.addSingle": "Add single",
  "layers.addCombo": "Add combo",
  "layers.layerLabel": "Layer {index}",
  "layers.type.single": "Single",
  "layers.type.combo": "Combo",
  "layers.mode": "Mode",
  "layers.moveUp": "Up",
  "layers.moveDown": "Down",
  "layers.remove": "Remove",
  "layers.expectedKeys": "Expected keys:",
  "layers.rows": "Rows",
  "layers.columns": "Columns",
  "layers.keys": "Keys",
  "layers.keysHint": "Keys (space or comma separated)",
  "layers.autoFit": "Auto-fit",
  "layers.currentExpected": "Current: {current} / Expected: {expected}",
  "layers.stage0": "Stage 0",
  "layers.stage1": "Stage 1",
  "hotkeys.section": "Hotkeys",
  "hotkeys.title": "Key Bindings",
  "hotkeys.subtitle": "Global shortcut syntax",
  "hotkeys.activation": "Activation",
  "hotkeys.controls": "Controls",
  "hotkeys.leftClick": "Left Click",
  "hotkeys.rightClick": "Right Click",
  "hotkeys.middleClick": "Middle Click",
  "hotkeys.cancel": "Cancel",
  "hotkeys.undo": "Undo",
  "hotkeys.directClick": "Direct Click",
  "overlay.section": "Overlay",
  "overlay.title": "Overlay Styling",
  "overlay.subtitle": "Mask, lines, typography",
  "overlay.alpha": "Alpha (0-255)",
  "overlay.lineWidth": "Line Width",
  "overlay.fontSize": "Font Size",
  "overlay.maskColor": "Mask Color",
  "overlay.lineColor": "Line Color",
  "overlay.textColor": "Text Color",
  "overlay.fontFamily": "Font Family",
  "footer.note":
    "Form-based settings are now available for presets, layers, hotkeys, and overlay styles. Apply to persist and refresh runtime behavior.",
  "errors.presetRequired": "At least one preset is required.",
  "errors.removePresetConfirm": "Remove preset \"{name}\"?",
  "errors.removeLayerConfirm": "Remove layer {index}?",
  "errors.presetLayerRequired": "Preset must have at least one layer.",
  "errors.presetsEmpty": "Presets list is empty.",
  "errors.presetIdEmpty": "Preset id cannot be empty.",
  "errors.presetNameEmpty": "Preset name cannot be empty.",
  "errors.duplicatePresetId": "Duplicate preset id: {id}",
  "errors.presetNoLayers": "Preset {id} has no layers.",
  "errors.layerGridInvalid": "Preset {id} layer {index} grid is invalid.",
  "errors.layerExpectedKeys":
    "Preset {id} layer {index} expects {expected} keys.",
  "errors.stage0GridInvalid":
    "Preset {id} layer {index} stage0 grid invalid.",
  "errors.stage1GridInvalid":
    "Preset {id} layer {index} stage1 grid invalid.",
  "errors.stage0ExpectedKeys":
    "Preset {id} layer {index} stage0 expects {expected} keys.",
  "errors.stage1ExpectedKeys":
    "Preset {id} layer {index} stage1 expects {expected} keys.",
  "errors.activePresetMissing": "Active preset is missing.",
  "errors.leftHotkeyEmpty": "Left activation hotkey is empty.",
  "errors.rightHotkeyEmpty": "Right activation hotkey is empty.",
  "errors.middleHotkeyEmpty": "Middle activation hotkey is empty.",
  "errors.cancelHotkeyEmpty": "Cancel hotkey is empty.",
  "errors.undoHotkeyEmpty": "Undo hotkey is empty.",
  "errors.directClickHotkeyEmpty": "Direct click hotkey is empty.",
  "errors.overlayLineWidth": "Overlay lineWidthPx must be > 0.",
  "errors.overlayFontSize": "Overlay font sizePx must be > 0.",
};

const translations: Record<Locale, Record<TranslationKey, string>> = {
  "zh-CN": zh,
  "en-US": en,
};

const formatter = (template: string, params?: TranslationParams) => {
  if (!params) {
    return template;
  }
  return template.replace(/\{(\w+)\}/g, (match, key) => {
    const value = params[key];
    if (value === undefined || value === null) {
      return match;
    }
    return String(value);
  });
};

export const locale = writable<Locale>(DEFAULT_LOCALE);

export const t = derived(locale, ($locale) => {
  return (key: TranslationKey, params?: TranslationParams) => {
    const template =
      translations[$locale]?.[key] ??
      translations[DEFAULT_LOCALE][key] ??
      key;
    return formatter(template, params);
  };
});

export const initLocale = () => {
  if (!browser) {
    return;
  }
  const stored = localStorage.getItem(STORAGE_KEY);
  if (stored === "zh-CN" || stored === "en-US") {
    locale.set(stored);
  }
  document.documentElement.lang = get(locale);
};

export const setLocale = (next: Locale) => {
  locale.set(next);
  if (!browser) {
    return;
  }
  localStorage.setItem(STORAGE_KEY, next);
  document.documentElement.lang = next;
};
