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

export interface AppConfig {
  app: {
    locale: "zh-CN" | "en-US";
    tray: {
      enabled: boolean;
    };
    settingsWindow: {
      openFromTray: boolean;
    };
  };
  hotkeys: {
    activation: {
      trigger: string;
    };
    controls: {
      cancel: string;
      undo: string;
      directClick: string;
      switchAction: string;
      nextMonitor: string;
    };
  };
  nudge: {
    stepPx: number;
  };
  mouse: {
    smoothMove: boolean;
    moveDurationMs: number;
    moveStepMs: number;
    pressDurationMs: number;
    landingRadiusPx: number;
    durationRandomness: number;
    stepRandomness: number;
    distanceBoostPx: number;
    durationDistanceBoost: number;
    stepDistanceBoost: number;
    curveAlongRatio: number;
    curveSpreadRatio: number;
    jitterRatio: number;
    adaptiveStrideBasePx: number;
    adaptiveStrideDistanceRatio: number;
    adaptiveStrideMaxPx: number;
    extraStepsMax: number;
    maxSteps: number;
    maxStepSleepMs: number;
  };
  layers: Layer[];
  overlay: {
    alpha: number;
    maskColor: string;
    lineColor: string;
    textColor: string;
    lineWidthPx: number;
    showGrid: boolean;
    showDiagonals: boolean;
    font: {
      family: string;
      sizePx: number;
    };
  };
}

export type ComboStage = 0 | 1;

export interface RuntimeState {
  layerIndex: number;
  stage: ComboStage;
  region: Region;
  baseRegion: Region;
  done: boolean;
  history: RuntimeSnapshot[];
}

export interface EngineOutput {
  state: RuntimeState;
  clickPoint?: Point;
  didAdvance: boolean;
}

export interface RuntimeSnapshot {
  layerIndex: number;
  stage: ComboStage;
  region: Region;
}

export interface CurrentStep {
  mode: "single" | "combo";
  stage: ComboStage;
  layerIndex: number;
  rows: number;
  cols: number;
  keys: KeyCode[];
}
