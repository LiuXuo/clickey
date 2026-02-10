export type KeyCode = string;

export interface Region {
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface Point {
  x: number;
  y: number;
}

export interface GridStage {
  rows: number;
  cols: number;
  keys: KeyCode[];
}

export interface SingleLayer {
  mode: "single";
  rows: number;
  cols: number;
  keys: KeyCode[];
}

export interface ComboLayer {
  mode: "combo";
  stage0: GridStage;
  stage1: GridStage;
}

export type Layer = SingleLayer | ComboLayer;

export interface Preset {
  id: string;
  name: string;
  layers: Layer[];
}

export interface AppConfig {
  app: {
    tray: {
      enabled: boolean;
    };
    settingsWindow: {
      openFromTray: boolean;
    };
  };
  hotkeys: {
    activation: {
      leftClick: string;
      rightClick: string;
      middleClick: string;
    };
    controls: {
      cancel: string;
      undo: string;
      directClick: string;
    };
  };
  activePresetId: string;
  presets: Preset[];
  overlay: {
    alpha: number;
    maskColor: string;
    lineColor: string;
    textColor: string;
    lineWidthPx: number;
    font: {
      family: string;
      sizePx: number;
    };
  };
}

export type ComboStage = 0 | 1;

export interface RuntimeState {
  presetId: string;
  layerIndex: number;
  stage: ComboStage;
  region: Region;
  done: boolean;
}

export interface EngineOutput {
  state: RuntimeState;
  clickPoint?: Point;
  didAdvance: boolean;
}

export interface CurrentStep {
  mode: "single" | "combo";
  stage: ComboStage;
  layerIndex: number;
  rows: number;
  cols: number;
  keys: KeyCode[];
}
