import type {
  AppConfig,
  ComboLayer,
  ComboStage,
  CurrentStep,
  EngineOutput,
  GridStage,
  Layer,
  Point,
  Preset,
  Region,
  RuntimeState,
  RuntimeSnapshot,
} from "./types";

export function cropRegion(
  current: Region,
  rows: number,
  cols: number,
  keyIndex: number,
): Region {
  const row = Math.ceil(keyIndex / cols);
  const col = ((keyIndex - 1) % cols) + 1;
  const cellWidth = current.width / cols;
  const cellHeight = current.height / rows;

  return {
    x: current.x + (col - 1) * cellWidth,
    y: current.y + (row - 1) * cellHeight,
    width: cellWidth,
    height: cellHeight,
  };
}

export function regionCenter(region: Region): Point {
  return {
    x: Math.round(region.x + region.width / 2),
    y: Math.round(region.y + region.height / 2),
  };
}

const KEY_ALIASES: Record<string, string> = {
  escape: "esc",
  esc: "esc",
  backspace: "backspace",
  space: "space",
  spacebar: "space",
  " ": "space",
  arrowleft: "left",
  arrowright: "right",
  arrowup: "up",
  arrowdown: "down",
};

function normalizeKey(value: string): string {
  const lower = value.toLowerCase();
  return KEY_ALIASES[lower] ?? lower;
}

function normalizeControlKeys(config: AppConfig) {
  return {
    cancel: normalizeKey(config.hotkeys.controls.cancel),
    undo: normalizeKey(config.hotkeys.controls.undo),
    directClick: normalizeKey(config.hotkeys.controls.directClick),
  };
}

function getNudgeDelta(key: string): Point | null {
  const normalized = normalizeKey(key);
  switch (normalized) {
    case "left":
      return { x: -5, y: 0 };
    case "right":
      return { x: 5, y: 0 };
    case "up":
      return { x: 0, y: -5 };
    case "down":
      return { x: 0, y: 5 };
    default:
      return null;
  }
}

function clamp(value: number, min: number, max: number): number {
  const safeMax = Math.max(min, max);
  return Math.min(Math.max(value, min), safeMax);
}

function nudgeRegion(region: Region, base: Region, delta: Point): Region {
  const nextX = clamp(
    region.x + delta.x,
    base.x,
    base.x + base.width - region.width,
  );
  const nextY = clamp(
    region.y + delta.y,
    base.y,
    base.y + base.height - region.height,
  );

  return { ...region, x: nextX, y: nextY };
}

function regionsEqual(a: Region, b: Region): boolean {
  return (
    a.x === b.x &&
    a.y === b.y &&
    a.width === b.width &&
    a.height === b.height
  );
}

function pushHistory(state: RuntimeState): RuntimeSnapshot[] {
  return [
    ...state.history,
    {
      layerIndex: state.layerIndex,
      stage: state.stage,
      region: { ...state.region },
    },
  ];
}

export function createInitialState(
  config: AppConfig,
  initialRegion: Region,
): RuntimeState {
  return {
    presetId: config.activePresetId,
    layerIndex: 0,
    stage: 0,
    region: { ...initialRegion },
    baseRegion: { ...initialRegion },
    done: false,
    history: [],
  };
}

export function applyKey(
  config: AppConfig,
  state: RuntimeState,
  key: string,
): EngineOutput {
  if (state.done) {
    return { state, didAdvance: false };
  }

  const normalizedKey = normalizeKey(key);
  const controlKeys = normalizeControlKeys(config);

  if (normalizedKey === controlKeys.cancel) {
    return { state: { ...state, done: true }, didAdvance: true };
  }

  if (normalizedKey === controlKeys.undo) {
    if (state.history.length === 0) {
      return { state: { ...state, done: true }, didAdvance: true };
    }

    const previous = state.history[state.history.length - 1];
    const nextState = {
      ...state,
      region: { ...previous.region },
      layerIndex: previous.layerIndex,
      stage: previous.stage,
      done: false,
      history: state.history.slice(0, -1),
    };

    return { state: nextState, didAdvance: true };
  }

  if (normalizedKey === controlKeys.directClick) {
    return {
      state: { ...state, done: true },
      clickPoint: regionCenter(state.region),
      didAdvance: true,
    };
  }

  const nudge = getNudgeDelta(normalizedKey);
  if (nudge) {
    const step = getCurrentStep(config, state);
    if (!step || step.mode !== "single") {
      return { state, didAdvance: false };
    }

    const nextRegion = nudgeRegion(state.region, state.baseRegion, nudge);
    if (regionsEqual(nextRegion, state.region)) {
      return { state, didAdvance: false };
    }

    return { state: { ...state, region: nextRegion }, didAdvance: true };
  }

  const preset = getPreset(config, state.presetId);
  if (!preset) {
    return { state, didAdvance: false };
  }

  const layer = preset.layers[state.layerIndex];
  if (!layer) {
    return { state: { ...state, done: true }, didAdvance: false };
  }

  const step = getStepForLayer(layer, state.stage);
  if (!step) {
    return { state, didAdvance: false };
  }

  const keyIndex = step.keys.findIndex(
    (candidate) => normalizeKey(candidate) === normalizedKey,
  );
  if (keyIndex < 0) {
    return { state, didAdvance: false };
  }

  const nextRegion = cropRegion(
    state.region,
    step.rows,
    step.cols,
    keyIndex + 1,
  );
  const history = pushHistory(state);
  const nextState = advanceState(state, preset, layer, nextRegion, history);

  if (nextState.done) {
    return {
      state: nextState,
      clickPoint: regionCenter(nextRegion),
      didAdvance: true,
    };
  }

  return { state: nextState, didAdvance: true };
}

export function getCurrentStep(
  config: AppConfig,
  state: RuntimeState,
): CurrentStep | null {
  const preset = getPreset(config, state.presetId);
  if (!preset) {
    return null;
  }

  const layer = preset.layers[state.layerIndex];
  if (!layer) {
    return null;
  }

  const step = getStepForLayer(layer, state.stage);
  if (!step) {
    return null;
  }

  return {
    mode: layer.mode,
    stage: state.stage,
    layerIndex: state.layerIndex,
    rows: step.rows,
    cols: step.cols,
    keys: step.keys,
  };
}

function getPreset(config: AppConfig, presetId: string): Preset | undefined {
  return config.presets.find((candidate) => candidate.id === presetId);
}

function getStepForLayer(layer: Layer, stage: ComboStage): GridStage | null {
  if (layer.mode === "single") {
    return {
      rows: layer.rows,
      cols: layer.cols,
      keys: layer.keys,
    };
  }

  return stage === 0 ? layer.stage0 : layer.stage1;
}

function advanceState(
  state: RuntimeState,
  preset: Preset,
  layer: Layer,
  nextRegion: Region,
  history: RuntimeSnapshot[],
): RuntimeState {
  let nextLayerIndex = state.layerIndex;
  let nextStage: ComboStage = 0;

  if (layer.mode === "combo") {
    if (state.stage === 0) {
      nextStage = 1;
    } else {
      nextLayerIndex += 1;
      nextStage = 0;
    }
  } else {
    nextLayerIndex += 1;
    nextStage = 0;
  }

  const done = nextLayerIndex >= preset.layers.length;

  return {
    ...state,
    region: nextRegion,
    layerIndex: nextLayerIndex,
    stage: nextStage,
    done,
    history,
  };
}

export function isComboLayer(layer: Layer): layer is ComboLayer {
  return layer.mode === "combo";
}
