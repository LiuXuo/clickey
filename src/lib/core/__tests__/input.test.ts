import { describe, expect, it } from "vitest";
import type { AppConfig, Region, RuntimeState } from "../types";
import { applyKey, createInitialState } from "../engine";

function makeConfig(): AppConfig {
  return {
    app: {
      tray: { enabled: true },
      settingsWindow: { openFromTray: true },
    },
    hotkeys: {
      activation: {
        leftClick: "Ctrl+;",
        rightClick: "Ctrl+Shift+;",
        middleClick: "Ctrl+Shift+Alt+;",
      },
      controls: {
        cancel: "Esc",
        undo: "Backspace",
        directClick: "Space",
      },
    },
    activePresetId: "test",
    presets: [
      {
        id: "test",
        name: "test",
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
    expect(nudged.state.region).toEqual({ x: 15, y: 10, width: 20, height: 20 });

    const clamped = applyKey(
      config,
      { ...singleState, region: { x: 0, y: 0, width: 20, height: 20 } },
      "Left",
    );
    expect(clamped.state.region.x).toBe(0);
  });
});
