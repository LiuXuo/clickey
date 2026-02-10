<script lang="ts">
  import { onMount } from "svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import type {
    OverlayActivatePayload,
    NativeKeyPayload,
    ClickAction,
  } from "$lib/ipc/types";
  import { applyKey, createInitialState, getCurrentStep } from "$lib/core";
  import type { AppConfig, Region, RuntimeState } from "$lib/core";

  let config = $state<AppConfig | null>(null);
  let runtime = $state<RuntimeState | null>(null);
  let baseRegion = $state<Region | null>(null);
  let clickAction = $state<ClickAction | null>(null);
  let canvas: HTMLCanvasElement | null = null;
  const currentWindow = getCurrentWindow();

  function draw() {
    if (!canvas) {
      return;
    }

    const ctx = canvas.getContext("2d");
    if (!ctx) {
      return;
    }

    const width = window.innerWidth;
    const height = window.innerHeight;
    const scale = window.devicePixelRatio || 1;
    canvas.width = Math.floor(width * scale);
    canvas.height = Math.floor(height * scale);
    ctx.setTransform(scale, 0, 0, scale, 0, 0);

    ctx.clearRect(0, 0, width, height);

    if (!config || !runtime || !baseRegion) {
      return;
    }

    const step = getCurrentStep(config, runtime);
    if (!step) {
      return;
    }

    const offsetX = -baseRegion.x / scale;
    const offsetY = -baseRegion.y / scale;
    const regionX = runtime.region.x / scale + offsetX;
    const regionY = runtime.region.y / scale + offsetY;
    const regionW = runtime.region.width / scale;
    const regionH = runtime.region.height / scale;

    // Mask the whole screen so grid/text read clearly
    ctx.fillStyle = `${config.overlay.maskColor}${Math.round(
      config.overlay.alpha,
    )
      .toString(16)
      .padStart(2, "0")}`;
    ctx.fillRect(0, 0, width, height);

    ctx.strokeStyle = config.overlay.lineColor;
    ctx.lineWidth = Math.max(1, config.overlay.lineWidthPx);

    ctx.strokeRect(regionX, regionY, regionW, regionH);

    const cellW = regionW / step.cols;
    const cellH = regionH / step.rows;

    for (let col = 1; col < step.cols; col += 1) {
      const x = regionX + col * cellW;
      ctx.beginPath();
      ctx.moveTo(x, regionY);
      ctx.lineTo(x, regionY + regionH);
      ctx.stroke();
    }

    for (let row = 1; row < step.rows; row += 1) {
      const y = regionY + row * cellH;
      ctx.beginPath();
      ctx.moveTo(regionX, y);
      ctx.lineTo(regionX + regionW, y);
      ctx.stroke();
    }

    ctx.fillStyle = config.overlay.textColor;
    ctx.font = `600 ${config.overlay.font.sizePx}px ${config.overlay.font.family}`;
    ctx.textAlign = "center";
    ctx.textBaseline = "middle";
    ctx.shadowColor = "rgba(0, 0, 0, 0.55)";
    ctx.shadowBlur = 4;

    step.keys.forEach((key, index) => {
      const row = Math.floor(index / step.cols);
      const col = index % step.cols;
      const x = regionX + (col + 0.5) * cellW;
      const y = regionY + (row + 0.5) * cellH;
      ctx.fillText(key, x, y);
    });
  }

  async function handleKey(key: string) {
    if (!config || !runtime) {
      return;
    }

    const result = applyKey(config, runtime, key);
    runtime = result.state;
    draw();

    if (result.clickPoint && clickAction) {
      await invoke("native_click", {
        x: result.clickPoint.x,
        y: result.clickPoint.y,
        button: clickAction,
      });
      await currentWindow.hide();
    }
  }

  onMount(() => {
    let unlistenActivate: (() => void) | undefined;
    let unlistenKey: (() => void) | undefined;

    void (async () => {
      unlistenActivate = await listen<OverlayActivatePayload>(
        "overlay:activate",
        (event) => {
          config = event.payload.config;
          baseRegion = event.payload.region;
          runtime = createInitialState(
            event.payload.config,
            event.payload.region,
          );
          clickAction = event.payload.clickAction;
          draw();
        },
      );

      unlistenKey = await listen<NativeKeyPayload>("native:key", (event) => {
        void handleKey(event.payload.key);
      });
    })();

    const handleResize = () => draw();
    window.addEventListener("resize", handleResize);

    return () => {
      unlistenActivate?.();
      unlistenKey?.();
      window.removeEventListener("resize", handleResize);
    };
  });
</script>

<main>
  <canvas bind:this={canvas}></canvas>
</main>

<style>
  :global(body) {
    margin: 0;
    background: transparent;
    overflow: hidden;
  }

  main {
    position: fixed;
    inset: 0;
    pointer-events: none;
  }

  canvas {
    width: 100vw;
    height: 100vh;
    display: block;
  }
</style>
