import { describe, expect, it } from "vitest";
import type { AppConfig, Region, RuntimeState } from "../types";
import { applyKey, createInitialState } from "../engine";

function makeConfig(): AppConfig {
  return {
    app: {
      locale: "zh-CN",
      tray: { enabled: true },
      settingsWindow: { openFromTray: true },
    },
    hotkeys: {
      activation: {
        trigger: "Ctrl+;",
      },
      controls: {
        cancel: "Esc",
        undo: "Backspace",
        directClick: "Space",
        switchAction: "Enter",
        nextMonitor: "Tab",
      },
    },
    nudge: {
      stepPx: 5,
    },
    mouse: {
      smoothMove: true,
      moveDurationMs: 120,
      moveStepMs: 8,
      pressDurationMs: 24,
      landingRadiusPx: 1,
      durationRandomness: 0.24,
      stepRandomness: 0.22,
      distanceBoostPx: 1800,
      durationDistanceBoost: 0.28,
      stepDistanceBoost: 0.42,
      curveAlongRatio: 0.08,
      curveSpreadRatio: 0.12,
      jitterRatio: 0.01,
      adaptiveStrideBasePx: 7,
      adaptiveStrideDistanceRatio: 0.026,
      adaptiveStrideMaxPx: 42,
      extraStepsMax: 6,
      maxSteps: 220,
      maxStepSleepMs: 24,
    },
    layers: [
      {
        mode: "combo",
        stage0: { rows: 1, cols: 2, keys: ["a", "b"] },
        stage1: { rows: 2, cols: 1, keys: ["c", "d"] },
      },
      {
        mode: "single",
        rows: 1,
        cols: 2,
        keys: ["e", "f"],
      },
    ],
    overlay: {
      alpha: 120,
      maskColor: "#000000",
      lineColor: "#ffffff",
      textColor: "#ffffff",
      lineWidthPx: 1,
      showGrid: true,
      showDiagonals: true,
      font: { family: "Segoe UI", sizePx: 12 },
    },
  };
}

describe("engine inputs", () => {
  it("cancels on Escape", () => {
    const config = makeConfig();
    const state = createInitialState(config, {
      x: 0,
      y: 0,
      width: 100,
      height: 100,
    });

    const result = applyKey(config, state, "Escape");

    expect(result.state.done).toBe(true);
    expect(result.clickPoint).toBeUndefined();
  });

  it("direct-clicks on Space", () => {
    const config = makeConfig();
    const state = createInitialState(config, {
      x: 0,
      y: 0,
      width: 100,
      height: 100,
    });

    const result = applyKey(config, state, "Space");

    expect(result.state.done).toBe(true);
    expect(result.clickPoint).toEqual({ x: 50, y: 50 });
  });

  it("undoes the last step", () => {
    const config = makeConfig();
    const region: Region = { x: 0, y: 0, width: 100, height: 100 };
    const initial = createInitialState(config, region);

    const advanced = applyKey(config, initial, "a");
    expect(advanced.state.stage).toBe(1);
    expect(advanced.state.history).toHaveLength(1);

    const undone = applyKey(config, advanced.state, "Backspace");
    expect(undone.state.stage).toBe(0);
    expect(undone.state.layerIndex).toBe(0);
    expect(undone.state.region).toEqual(region);
    expect(undone.state.history).toHaveLength(0);
  });

  it("nudges region only in single step", () => {
    const config = makeConfig();
    const initial = createInitialState(config, {
      x: 0,
      y: 0,
      width: 100,
      height: 100,
    });

    const singleState: RuntimeState = {
      ...initial,
      layerIndex: 1,
      stage: 0,
      region: { x: 10, y: 10, width: 20, height: 20 },
    };

    const nudged = applyKey(config, singleState, "Right");
    expect(nudged.state.region).toEqual({
      x: 15,
      y: 10,
      width: 20,
      height: 20,
    });

    const clamped = applyKey(
      config,
      { ...singleState, region: { x: 0, y: 0, width: 20, height: 20 } },
      "Left",
    );
    expect(clamped.state.region.x).toBe(0);
  });

  it("uses configured nudge step size", () => {
    const config = makeConfig();
    config.nudge.stepPx = 12;
    const initial = createInitialState(config, {
      x: 0,
      y: 0,
      width: 100,
      height: 100,
    });

    const singleState: RuntimeState = {
      ...initial,
      layerIndex: 1,
      stage: 0,
      region: { x: 10, y: 10, width: 20, height: 20 },
    };

    const nudged = applyKey(config, singleState, "Right");
    expect(nudged.state.region).toEqual({
      x: 22,
      y: 10,
      width: 20,
      height: 20,
    });
  });
});
