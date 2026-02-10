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

export function createInitialState(
  config: AppConfig,
  initialRegion: Region,
): RuntimeState {
  return {
    presetId: config.activePresetId,
    layerIndex: 0,
    stage: 0,
    region: { ...initialRegion },
    done: false,
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

  const keyIndex = step.keys.indexOf(key);
  if (keyIndex < 0) {
    return { state, didAdvance: false };
  }

  const nextRegion = cropRegion(
    state.region,
    step.rows,
    step.cols,
    keyIndex + 1,
  );
  const nextState = advanceState(state, preset, layer, nextRegion);

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
  };
}

export function isComboLayer(layer: Layer): layer is ComboLayer {
  return layer.mode === "combo";
}
