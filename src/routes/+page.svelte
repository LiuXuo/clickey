<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import { initLocale, locale, setLocale, t, type Locale } from "$lib/i18n";
  import defaultConfig from "$lib/shared/default-config.json";
  import type { AppConfig } from "$lib/core";

  const initialConfig = JSON.parse(JSON.stringify(defaultConfig)) as AppConfig;

  const fieldClass =
    "mt-2 w-full rounded-lg border border-zinc-300 bg-white px-3 py-2 text-sm text-zinc-900 shadow-sm focus:border-zinc-900 focus:outline-none focus:ring-2 focus:ring-zinc-900/20 disabled:cursor-not-allowed disabled:bg-zinc-100";
  const textAreaClass =
    "mt-2 w-full min-h-[100px] rounded-lg border border-zinc-300 bg-white px-3 py-2 text-xs leading-relaxed text-zinc-900 shadow-sm focus:border-zinc-900 focus:outline-none focus:ring-2 focus:ring-zinc-900/20 disabled:cursor-not-allowed disabled:bg-zinc-100";
  const compactSelectClass =
    "rounded-lg border border-zinc-300 bg-white px-3 py-2 text-xs text-zinc-900 shadow-sm focus:border-zinc-900 focus:outline-none focus:ring-2 focus:ring-zinc-900/20 disabled:cursor-not-allowed disabled:bg-zinc-100";

  const keyPool = [
    "q",
    "w",
    "e",
    "r",
    "t",
    "a",
    "s",
    "d",
    "f",
    "g",
    "z",
    "x",
    "c",
    "v",
    "b",
    "y",
    "u",
    "i",
    "o",
    "p",
    "h",
    "j",
    "k",
    "l",
    ";",
    "n",
    "m",
    ",",
    ".",
    "/",
  ];

  let config = $state<AppConfig>(initialConfig);
  let status = $state("");
  let error = $state("");
  let isLoading = $state(true);
  let isApplying = $state(false);
  let isResetting = $state(false);
  let isImporting = $state(false);
  let isExporting = $state(false);
  let fileInput: HTMLInputElement | null = null;

  function clearFeedback() {
    status = "";
    error = "";
  }

  function toPositiveInt(value: string, fallback: number): number {
    const parsed = Number.parseInt(value, 10);
    return Number.isFinite(parsed) && parsed > 0 ? parsed : fallback;
  }

  function clampInt(value: string, min: number, max: number, fallback: number) {
    const parsed = Number.parseInt(value, 10);
    if (!Number.isFinite(parsed)) {
      return fallback;
    }
    return Math.min(Math.max(parsed, min), max);
  }

  function parseKeys(value: string): string[] {
    return value.split(/[\s,]+/).filter(Boolean);
  }

  function formatKeys(keys: string[]): string {
    return keys.join(" ");
  }

  function fillKeys(existing: string[], count: number): string[] {
    const result = existing
      .filter((key) => key.trim().length > 0)
      .slice(0, count);
    for (let index = result.length; index < count; index += 1) {
      result.push(keyPool[index % keyPool.length]);
    }
    return result;
  }

  function getDefaultSingleLayer() {
    const candidate = initialConfig.layers.find(
      (layer) => layer.mode === "single",
    );
    if (candidate && candidate.mode === "single") {
      return {
        rows: candidate.rows,
        cols: candidate.cols,
        keys: [...candidate.keys],
      };
    }
    return { rows: 3, cols: 5, keys: keyPool.slice(0, 15) };
  }

  function getDefaultComboLayer() {
    const candidate = initialConfig.layers.find(
      (layer) => layer.mode === "combo",
    );
    if (candidate && candidate.mode === "combo") {
      return {
        stage0: {
          rows: candidate.stage0.rows,
          cols: candidate.stage0.cols,
          keys: [...candidate.stage0.keys],
        },
        stage1: {
          rows: candidate.stage1.rows,
          cols: candidate.stage1.cols,
          keys: [...candidate.stage1.keys],
        },
      };
    }
    return {
      stage0: { rows: 1, cols: 15, keys: keyPool.slice(0, 15) },
      stage1: { rows: 15, cols: 1, keys: keyPool.slice(0, 15) },
    };
  }

  function onLocaleChange(event: Event) {
    const target = event.currentTarget as HTMLSelectElement;
    const next = target.value as Locale;
    setLocale(next);
    config.app.locale = next;
    clearFeedback();
    void invoke("set_locale", { locale: next }).catch((err) => {
      error = err instanceof Error ? err.message : String(err);
    });
  }

  function switchLayerMode(index: number, mode: "single" | "combo") {
    const layer = config.layers[index];
    if (!layer || layer.mode === mode) {
      return;
    }

    if (mode === "single") {
      const defaults = getDefaultSingleLayer();
      const pooled =
        layer.mode === "combo"
          ? [...layer.stage0.keys, ...layer.stage1.keys]
          : layer.keys;
      const nextLayers = [...config.layers];
      nextLayers[index] = {
        mode: "single",
        rows: defaults.rows,
        cols: defaults.cols,
        keys: fillKeys(pooled, defaults.rows * defaults.cols),
      };
      config.layers = nextLayers;
      clearFeedback();
      return;
    }

    const defaults = getDefaultComboLayer();
    const pooled =
      layer.mode === "single"
        ? layer.keys
        : [...layer.stage0.keys, ...layer.stage1.keys];
    const nextLayers = [...config.layers];
    nextLayers[index] = {
      mode: "combo",
      stage0: {
        ...defaults.stage0,
        keys: fillKeys(pooled, defaults.stage0.rows * defaults.stage0.cols),
      },
      stage1: {
        ...defaults.stage1,
        keys: fillKeys(pooled, defaults.stage1.rows * defaults.stage1.cols),
      },
    };
    config.layers = nextLayers;
    clearFeedback();
  }

  function addSingleLayer() {
    const base = getDefaultSingleLayer();
    config.layers = [
      ...config.layers,
      {
        mode: "single",
        rows: base.rows,
        cols: base.cols,
        keys: fillKeys(base.keys, base.rows * base.cols),
      },
    ];
    clearFeedback();
  }

  function addComboLayer() {
    const base = getDefaultComboLayer();
    config.layers = [
      ...config.layers,
      {
        mode: "combo",
        stage0: {
          ...base.stage0,
          keys: fillKeys(base.stage0.keys, base.stage0.rows * base.stage0.cols),
        },
        stage1: {
          ...base.stage1,
          keys: fillKeys(base.stage1.keys, base.stage1.rows * base.stage1.cols),
        },
      },
    ];
    clearFeedback();
  }
  function moveLayer(index: number, direction: -1 | 1) {
    const nextIndex = index + direction;
    if (nextIndex < 0 || nextIndex >= config.layers.length) {
      return;
    }
    const nextLayers = [...config.layers];
    const [moved] = nextLayers.splice(index, 1);
    nextLayers.splice(nextIndex, 0, moved);
    config.layers = nextLayers;
    clearFeedback();
  }

  function removeLayer(index: number) {
    if (config.layers.length <= 1) {
      error = $t("errors.layersRequired");
      return;
    }
    if (!confirm($t("errors.removeLayerConfirm", { index: index + 1 }))) {
      return;
    }
    config.layers = config.layers.filter(
      (_, layerIndex) => layerIndex !== index,
    );
    clearFeedback();
  }

  function updateSingleLayerGrid(
    index: number,
    field: "rows" | "cols",
    event: Event,
  ) {
    const layer = config.layers[index];
    if (!layer || layer.mode !== "single") {
      return;
    }
    const target = event.currentTarget as HTMLInputElement;
    layer[field] = toPositiveInt(target.value, layer[field]);
    clearFeedback();
  }

  function updateSingleLayerKeys(index: number, event: Event) {
    const layer = config.layers[index];
    if (!layer || layer.mode !== "single") {
      return;
    }
    const target = event.currentTarget as HTMLTextAreaElement;
    layer.keys = parseKeys(target.value);
    clearFeedback();
  }

  function updateComboStageGrid(
    index: number,
    stage: 0 | 1,
    field: "rows" | "cols",
    event: Event,
  ) {
    const layer = config.layers[index];
    if (!layer || layer.mode !== "combo") {
      return;
    }
    const target = event.currentTarget as HTMLInputElement;
    const stageConfig = stage === 0 ? layer.stage0 : layer.stage1;
    stageConfig[field] = toPositiveInt(target.value, stageConfig[field]);
    clearFeedback();
  }

  function updateComboStageKeys(index: number, stage: 0 | 1, event: Event) {
    const layer = config.layers[index];
    if (!layer || layer.mode !== "combo") {
      return;
    }
    const target = event.currentTarget as HTMLTextAreaElement;
    const stageConfig = stage === 0 ? layer.stage0 : layer.stage1;
    stageConfig.keys = parseKeys(target.value);
    clearFeedback();
  }

  function validateConfig(candidate: AppConfig): string[] {
    const issues: string[] = [];

    if (!candidate.layers.length) {
      issues.push($t("errors.layersRequired"));
      return issues;
    }

    candidate.layers.forEach((layer, index) => {
      if (layer.mode === "single") {
        const expected = layer.rows * layer.cols;
        if (!layer.rows || !layer.cols) {
          issues.push(
            $t("errors.layerGridInvalidSimple", { index: index + 1 }),
          );
        }
        if (layer.keys.length !== expected) {
          issues.push(
            $t("errors.layerExpectedKeysSimple", {
              index: index + 1,
              expected,
            }),
          );
        }
      } else {
        const expected0 = layer.stage0.rows * layer.stage0.cols;
        const expected1 = layer.stage1.rows * layer.stage1.cols;
        if (!layer.stage0.rows || !layer.stage0.cols) {
          issues.push(
            $t("errors.stage0GridInvalidSimple", { index: index + 1 }),
          );
        }
        if (!layer.stage1.rows || !layer.stage1.cols) {
          issues.push(
            $t("errors.stage1GridInvalidSimple", { index: index + 1 }),
          );
        }
        if (layer.stage0.keys.length !== expected0) {
          issues.push(
            $t("errors.stage0ExpectedKeysSimple", {
              index: index + 1,
              expected: expected0,
            }),
          );
        }
        if (layer.stage1.keys.length !== expected1) {
          issues.push(
            $t("errors.stage1ExpectedKeysSimple", {
              index: index + 1,
              expected: expected1,
            }),
          );
        }
      }
    });

    if (!candidate.hotkeys.activation.trigger.trim()) {
      issues.push($t("errors.activationHotkeyEmpty"));
    }
    if (!candidate.hotkeys.controls.cancel.trim()) {
      issues.push($t("errors.cancelHotkeyEmpty"));
    }
    if (!candidate.hotkeys.controls.undo.trim()) {
      issues.push($t("errors.undoHotkeyEmpty"));
    }
    if (!candidate.hotkeys.controls.directClick.trim()) {
      issues.push($t("errors.directClickHotkeyEmpty"));
    }
    if (!candidate.hotkeys.controls.switchAction.trim()) {
      issues.push($t("errors.switchActionHotkeyEmpty"));
    }
    if (!candidate.hotkeys.controls.nextMonitor.trim()) {
      issues.push($t("errors.nextMonitorHotkeyEmpty"));
    }
    if (candidate.nudge.stepPx <= 0) {
      issues.push($t("errors.nudgeStep"));
    }
    if (candidate.overlay.lineWidthPx <= 0) {
      issues.push($t("errors.overlayLineWidth"));
    }
    if (candidate.overlay.font.sizePx <= 0) {
      issues.push($t("errors.overlayFontSize"));
    }

    return issues;
  }

  async function applyConfig() {
    error = "";
    status = "";
    const issues = validateConfig(config);
    if (issues.length) {
      error = issues[0];
      return;
    }
    isApplying = true;
    try {
      await invoke("apply_config", { config });
      status = $t("status.applied");
    } catch (err) {
      error = err instanceof Error ? err.message : String(err);
    } finally {
      isApplying = false;
    }
  }

  async function resetConfig() {
    error = "";
    status = "";
    isResetting = true;
    try {
      const reset = await invoke<AppConfig>("reset_config");
      config = reset;
      if (reset.app.locale === "zh-CN" || reset.app.locale === "en-US") {
        setLocale(reset.app.locale);
      }
      status = $t("status.reset");
    } catch (err) {
      error = err instanceof Error ? err.message : String(err);
    } finally {
      isResetting = false;
    }
  }

  async function exportOverrideJson() {
    error = "";
    status = "";
    isExporting = true;
    try {
      const json = await invoke<string>("export_override_json");
      const blob = new Blob([json], { type: "application/json;charset=utf-8" });
      const url = URL.createObjectURL(blob);
      const anchor = document.createElement("a");
      anchor.href = url;
      anchor.download = "clickey.settings.override.json";
      document.body.appendChild(anchor);
      anchor.click();
      anchor.remove();
      URL.revokeObjectURL(url);
      status = $t("status.exported");
    } catch (err) {
      error = err instanceof Error ? err.message : String(err);
    } finally {
      isExporting = false;
    }
  }

  function openImportPicker() {
    fileInput?.click();
  }

  async function onImportFileChange(event: Event) {
    const input = event.currentTarget as HTMLInputElement;
    const file = input.files?.[0];
    input.value = "";
    if (!file) {
      return;
    }

    error = "";
    status = "";
    isImporting = true;
    try {
      const json = await file.text();
      const imported = await invoke<AppConfig>("import_override_json", {
        json,
      });
      config = imported;
      if (imported.app.locale === "zh-CN" || imported.app.locale === "en-US") {
        setLocale(imported.app.locale);
      }
      status = $t("status.imported");
    } catch (err) {
      error = err instanceof Error ? err.message : String(err);
    } finally {
      isImporting = false;
    }
  }

  onMount(() => {
    initLocale();
    void (async () => {
      try {
        const loaded = await invoke<AppConfig>("get_config");
        config = loaded;
        if (loaded.app.locale === "zh-CN" || loaded.app.locale === "en-US") {
          setLocale(loaded.app.locale);
        }
      } catch (err) {
        error = err instanceof Error ? err.message : String(err);
      } finally {
        isLoading = false;
      }
    })();
  });
</script>

<main class="min-h-screen px-6 py-10">
  <input
    bind:this={fileInput}
    type="file"
    accept="application/json,.json"
    class="hidden"
    onchange={onImportFileChange}
  />

  <div class="mx-auto flex w-full max-w-5xl flex-col gap-8">
    <header class="flex flex-wrap items-end justify-between gap-4">
      <div>
        <p class="text-xs uppercase tracking-[0.35em] text-zinc-500">
          {$t("app.settings")}
        </p>
        <h1 class="text-3xl font-semibold tracking-tight text-zinc-900">
          {$t("app.brand")}
        </h1>
      </div>
      <div class="flex flex-wrap items-center gap-3">
        <div class="flex items-center gap-2">
          <label
            class="text-[11px] font-semibold uppercase tracking-[0.2em] text-zinc-500"
            for="locale-select"
          >
            {$t("language.label")}
          </label>
          <select
            id="locale-select"
            class={compactSelectClass}
            value={$locale}
            onchange={onLocaleChange}
            disabled={isLoading}
          >
            <option value="zh-CN">{$t("language.zh")}</option>
            <option value="en-US">{$t("language.en")}</option>
          </select>
        </div>

        <button
          type="button"
          class="inline-flex items-center justify-center rounded-lg border border-zinc-300 bg-white px-4 py-2 text-sm font-semibold text-zinc-700 shadow-sm transition hover:border-zinc-400 hover:text-zinc-900 disabled:cursor-not-allowed disabled:opacity-60"
          onclick={openImportPicker}
          disabled={isLoading || isImporting || isApplying || isResetting}
        >
          {isImporting ? $t("app.importing") : $t("app.import")}
        </button>

        <button
          type="button"
          class="inline-flex items-center justify-center rounded-lg border border-zinc-300 bg-white px-4 py-2 text-sm font-semibold text-zinc-700 shadow-sm transition hover:border-zinc-400 hover:text-zinc-900 disabled:cursor-not-allowed disabled:opacity-60"
          onclick={exportOverrideJson}
          disabled={isLoading || isExporting || isApplying || isResetting}
        >
          {isExporting ? $t("app.exporting") : $t("app.export")}
        </button>

        <button
          type="button"
          class="inline-flex items-center justify-center rounded-lg bg-zinc-900 px-4 py-2 text-sm font-semibold text-white shadow-sm transition hover:bg-zinc-800 disabled:cursor-not-allowed disabled:bg-zinc-400"
          onclick={applyConfig}
          disabled={isApplying || isLoading}
        >
          {isLoading
            ? $t("app.loading")
            : isApplying
              ? $t("app.applying")
              : $t("app.apply")}
        </button>

        <button
          type="button"
          class="inline-flex items-center justify-center rounded-lg border border-zinc-300 bg-white px-4 py-2 text-sm font-semibold text-zinc-700 shadow-sm transition hover:border-zinc-400 hover:text-zinc-900 disabled:cursor-not-allowed disabled:opacity-60"
          onclick={resetConfig}
          disabled={isResetting || isLoading}
        >
          {isResetting ? $t("app.resetting") : $t("app.reset")}
        </button>
      </div>
    </header>

    {#if status || error}
      <div class="flex flex-wrap items-center gap-3 text-sm">
        {#if status}
          <span class="font-semibold text-zinc-800">{status}</span>
        {/if}
        {#if error}
          <span class="font-semibold text-zinc-500">{error}</span>
        {/if}
      </div>
    {/if}

    <section
      class="rounded-2xl border border-zinc-200 bg-white/90 p-6 shadow-sm backdrop-blur"
    >
      <div class="flex items-center justify-between gap-4">
        <div>
          <p class="text-xs uppercase tracking-[0.28em] text-zinc-500">
            {$t("nudge.section")}
          </p>
          <h2 class="text-lg font-semibold text-zinc-900">
            {$t("nudge.title")}
          </h2>
        </div>
        <p class="text-xs text-zinc-500">{$t("nudge.subtitle")}</p>
      </div>

      <div class="mt-6 grid gap-6 md:grid-cols-2">
        <div>
          <label class="text-sm font-medium text-zinc-700" for="nudge-step"
            >{$t("nudge.step")}</label
          >
          <input
            id="nudge-step"
            type="number"
            min="1"
            class={fieldClass}
            value={config.nudge.stepPx}
            oninput={(event) => {
              const target = event.currentTarget as HTMLInputElement;
              config.nudge.stepPx = toPositiveInt(
                target.value,
                config.nudge.stepPx,
              );
              clearFeedback();
            }}
            disabled={isLoading}
          />
        </div>
      </div>
    </section>

    <section
      class="rounded-2xl border border-zinc-200 bg-white/90 p-6 shadow-sm backdrop-blur"
    >
      <div class="flex items-center justify-between gap-4">
        <div>
          <p class="text-xs uppercase tracking-[0.28em] text-zinc-500">
            {$t("layers.section")}
          </p>
          <h2 class="text-lg font-semibold text-zinc-900">
            {$t("layers.title")}
          </h2>
        </div>
        <div class="flex flex-wrap items-center gap-2">
          <button
            type="button"
            class="inline-flex items-center justify-center rounded-lg border border-zinc-300 bg-white px-3 py-2 text-xs font-semibold text-zinc-700 shadow-sm transition hover:border-zinc-400 hover:text-zinc-900 disabled:cursor-not-allowed disabled:opacity-60"
            onclick={addSingleLayer}
            disabled={isLoading}>{$t("layers.addSingle")}</button
          >
          <button
            type="button"
            class="inline-flex items-center justify-center rounded-lg border border-zinc-300 bg-white px-3 py-2 text-xs font-semibold text-zinc-700 shadow-sm transition hover:border-zinc-400 hover:text-zinc-900 disabled:cursor-not-allowed disabled:opacity-60"
            onclick={addComboLayer}
            disabled={isLoading}>{$t("layers.addCombo")}</button
          >
        </div>
      </div>
      <p class="mt-2 text-xs text-zinc-500">{$t("layers.subtitle")}</p>

      <div class="mt-6 space-y-4">
        {#each config.layers as layer, index (index)}
          <div class="rounded-xl border border-zinc-200 p-4">
            <div class="flex flex-wrap items-center justify-between gap-4">
              <div>
                <p class="text-xs uppercase tracking-[0.24em] text-zinc-500">
                  {$t("layers.layerLabel", { index: index + 1 })}
                </p>
                <p class="text-sm font-semibold text-zinc-900">
                  {layer.mode === "single"
                    ? $t("layers.type.single")
                    : $t("layers.type.combo")}
                </p>
              </div>
              <div class="min-w-[140px]">
                <label
                  class="text-[11px] font-semibold uppercase tracking-[0.2em] text-zinc-500"
                  for={`layer-${index}-mode`}>{$t("layers.mode")}</label
                >
                <select
                  id={`layer-${index}-mode`}
                  class={fieldClass}
                  value={layer.mode}
                  onchange={(event) =>
                    switchLayerMode(
                      index,
                      (event.currentTarget as HTMLSelectElement).value as
                        | "single"
                        | "combo",
                    )}
                  disabled={isLoading}
                >
                  <option value="single">{$t("layers.type.single")}</option>
                  <option value="combo">{$t("layers.type.combo")}</option>
                </select>
              </div>
              <div class="flex flex-wrap items-center gap-2">
                <button
                  type="button"
                  class="inline-flex items-center justify-center rounded-lg border border-zinc-300 bg-white px-2.5 py-1 text-[11px] font-semibold text-zinc-700 shadow-sm transition hover:border-zinc-400 hover:text-zinc-900 disabled:cursor-not-allowed disabled:opacity-60"
                  onclick={() => moveLayer(index, -1)}
                  disabled={isLoading || index === 0}
                  >{$t("layers.moveUp")}</button
                >
                <button
                  type="button"
                  class="inline-flex items-center justify-center rounded-lg border border-zinc-300 bg-white px-2.5 py-1 text-[11px] font-semibold text-zinc-700 shadow-sm transition hover:border-zinc-400 hover:text-zinc-900 disabled:cursor-not-allowed disabled:opacity-60"
                  onclick={() => moveLayer(index, 1)}
                  disabled={isLoading || index === config.layers.length - 1}
                  >{$t("layers.moveDown")}</button
                >
                <button
                  type="button"
                  class="inline-flex items-center justify-center rounded-lg border border-zinc-300 bg-white px-2.5 py-1 text-[11px] font-semibold text-zinc-700 shadow-sm transition hover:border-zinc-400 hover:text-zinc-900 disabled:cursor-not-allowed disabled:opacity-60"
                  onclick={() => removeLayer(index)}
                  disabled={isLoading || config.layers.length <= 1}
                  >{$t("layers.remove")}</button
                >
              </div>
            </div>

            {#if layer.mode === "single"}
              <div class="mt-4 grid gap-4 md:grid-cols-2">
                <div>
                  <label
                    class="text-sm font-medium text-zinc-700"
                    for={`layer-${index}-rows`}>{$t("layers.rows")}</label
                  >
                  <input
                    id={`layer-${index}-rows`}
                    type="number"
                    min="1"
                    class={fieldClass}
                    value={layer.rows}
                    oninput={(event) =>
                      updateSingleLayerGrid(index, "rows", event)}
                    disabled={isLoading}
                  />
                </div>
                <div>
                  <label
                    class="text-sm font-medium text-zinc-700"
                    for={`layer-${index}-cols`}>{$t("layers.columns")}</label
                  >
                  <input
                    id={`layer-${index}-cols`}
                    type="number"
                    min="1"
                    class={fieldClass}
                    value={layer.cols}
                    oninput={(event) =>
                      updateSingleLayerGrid(index, "cols", event)}
                    disabled={isLoading}
                  />
                </div>
              </div>
              <label
                class="mt-3 block text-sm font-medium text-zinc-700"
                for={`layer-${index}-keys`}>{$t("layers.keysHint")}</label
              >
              <textarea
                id={`layer-${index}-keys`}
                class={textAreaClass}
                value={formatKeys(layer.keys)}
                oninput={(event) => updateSingleLayerKeys(index, event)}
                disabled={isLoading}
              ></textarea>
            {:else}
              <div class="mt-4 grid gap-4 md:grid-cols-2">
                <div class="rounded-lg border border-zinc-200 p-4">
                  <p class="text-xs uppercase tracking-[0.24em] text-zinc-500">
                    {$t("layers.stage0")}
                  </p>
                  <div class="mt-3 grid gap-3 md:grid-cols-2">
                    <div>
                      <label
                        class="text-sm font-medium text-zinc-700"
                        for={`layer-${index}-stage0-rows`}
                        >{$t("layers.rows")}</label
                      >
                      <input
                        id={`layer-${index}-stage0-rows`}
                        type="number"
                        min="1"
                        class={fieldClass}
                        value={layer.stage0.rows}
                        oninput={(event) =>
                          updateComboStageGrid(index, 0, "rows", event)}
                        disabled={isLoading}
                      />
                    </div>
                    <div>
                      <label
                        class="text-sm font-medium text-zinc-700"
                        for={`layer-${index}-stage0-cols`}
                        >{$t("layers.columns")}</label
                      >
                      <input
                        id={`layer-${index}-stage0-cols`}
                        type="number"
                        min="1"
                        class={fieldClass}
                        value={layer.stage0.cols}
                        oninput={(event) =>
                          updateComboStageGrid(index, 0, "cols", event)}
                        disabled={isLoading}
                      />
                    </div>
                  </div>
                  <label
                    class="mt-3 block text-sm font-medium text-zinc-700"
                    for={`layer-${index}-stage0-keys`}
                    >{$t("layers.keys")}</label
                  >
                  <textarea
                    id={`layer-${index}-stage0-keys`}
                    class={textAreaClass}
                    value={formatKeys(layer.stage0.keys)}
                    oninput={(event) => updateComboStageKeys(index, 0, event)}
                    disabled={isLoading}
                  ></textarea>
                </div>

                <div class="rounded-lg border border-zinc-200 p-4">
                  <p class="text-xs uppercase tracking-[0.24em] text-zinc-500">
                    {$t("layers.stage1")}
                  </p>
                  <div class="mt-3 grid gap-3 md:grid-cols-2">
                    <div>
                      <label
                        class="text-sm font-medium text-zinc-700"
                        for={`layer-${index}-stage1-rows`}
                        >{$t("layers.rows")}</label
                      >
                      <input
                        id={`layer-${index}-stage1-rows`}
                        type="number"
                        min="1"
                        class={fieldClass}
                        value={layer.stage1.rows}
                        oninput={(event) =>
                          updateComboStageGrid(index, 1, "rows", event)}
                        disabled={isLoading}
                      />
                    </div>
                    <div>
                      <label
                        class="text-sm font-medium text-zinc-700"
                        for={`layer-${index}-stage1-cols`}
                        >{$t("layers.columns")}</label
                      >
                      <input
                        id={`layer-${index}-stage1-cols`}
                        type="number"
                        min="1"
                        class={fieldClass}
                        value={layer.stage1.cols}
                        oninput={(event) =>
                          updateComboStageGrid(index, 1, "cols", event)}
                        disabled={isLoading}
                      />
                    </div>
                  </div>
                  <label
                    class="mt-3 block text-sm font-medium text-zinc-700"
                    for={`layer-${index}-stage1-keys`}
                    >{$t("layers.keys")}</label
                  >
                  <textarea
                    id={`layer-${index}-stage1-keys`}
                    class={textAreaClass}
                    value={formatKeys(layer.stage1.keys)}
                    oninput={(event) => updateComboStageKeys(index, 1, event)}
                    disabled={isLoading}
                  ></textarea>
                </div>
              </div>
            {/if}
          </div>
        {/each}
      </div>
    </section>

    <section
      class="rounded-2xl border border-zinc-200 bg-white/90 p-6 shadow-sm backdrop-blur"
    >
      <div class="flex items-center justify-between gap-4">
        <div>
          <p class="text-xs uppercase tracking-[0.28em] text-zinc-500">
            {$t("hotkeys.section")}
          </p>
          <h2 class="text-lg font-semibold text-zinc-900">
            {$t("hotkeys.title")}
          </h2>
        </div>
      </div>

      <div class="mt-6 grid gap-6 md:grid-cols-2">
        <div>
          <label class="text-sm font-medium text-zinc-700" for="hotkey-trigger"
            >{$t("hotkeys.trigger")}</label
          >
          <input
            id="hotkey-trigger"
            class={fieldClass}
            bind:value={config.hotkeys.activation.trigger}
            oninput={clearFeedback}
            disabled={isLoading}
          />
          <label
            class="mt-3 block text-sm font-medium text-zinc-700"
            for="hotkey-switch-action">{$t("hotkeys.switchAction")}</label
          >
          <input
            id="hotkey-switch-action"
            class={fieldClass}
            bind:value={config.hotkeys.controls.switchAction}
            oninput={clearFeedback}
            disabled={isLoading}
          />
        </div>
        <div>
          <label class="text-sm font-medium text-zinc-700" for="hotkey-cancel"
            >{$t("hotkeys.cancel")}</label
          >
          <input
            id="hotkey-cancel"
            class={fieldClass}
            bind:value={config.hotkeys.controls.cancel}
            oninput={clearFeedback}
            disabled={isLoading}
          />
          <label
            class="mt-3 block text-sm font-medium text-zinc-700"
            for="hotkey-undo">{$t("hotkeys.undo")}</label
          >
          <input
            id="hotkey-undo"
            class={fieldClass}
            bind:value={config.hotkeys.controls.undo}
            oninput={clearFeedback}
            disabled={isLoading}
          />
          <label
            class="mt-3 block text-sm font-medium text-zinc-700"
            for="hotkey-direct">{$t("hotkeys.directClick")}</label
          >
          <input
            id="hotkey-direct"
            class={fieldClass}
            bind:value={config.hotkeys.controls.directClick}
            oninput={clearFeedback}
            disabled={isLoading}
          />
          <label
            class="mt-3 block text-sm font-medium text-zinc-700"
            for="hotkey-next-monitor">{$t("hotkeys.nextMonitor")}</label
          >
          <input
            id="hotkey-next-monitor"
            class={fieldClass}
            bind:value={config.hotkeys.controls.nextMonitor}
            oninput={clearFeedback}
            disabled={isLoading}
          />
        </div>
      </div>
    </section>

    <section
      class="rounded-2xl border border-zinc-200 bg-white/90 p-6 shadow-sm backdrop-blur"
    >
      <div class="flex items-center justify-between gap-4">
        <div>
          <p class="text-xs uppercase tracking-[0.28em] text-zinc-500">
            {$t("overlay.section")}
          </p>
          <h2 class="text-lg font-semibold text-zinc-900">
            {$t("overlay.title")}
          </h2>
        </div>
      </div>

      <div class="mt-6 grid gap-6 md:grid-cols-2">
        <div>
          <label class="text-sm font-medium text-zinc-700" for="overlay-alpha"
            >{$t("overlay.alpha")}</label
          >
          <input
            id="overlay-alpha"
            type="number"
            min="0"
            max="255"
            class={fieldClass}
            value={config.overlay.alpha}
            oninput={(event) => {
              const target = event.currentTarget as HTMLInputElement;
              config.overlay.alpha = clampInt(
                target.value,
                0,
                255,
                config.overlay.alpha,
              );
              clearFeedback();
            }}
            disabled={isLoading}
          />
        </div>
        <div>
          <label class="text-sm font-medium text-zinc-700" for="overlay-line"
            >{$t("overlay.lineWidth")}</label
          >
          <input
            id="overlay-line"
            type="number"
            min="1"
            class={fieldClass}
            value={config.overlay.lineWidthPx}
            oninput={(event) => {
              const target = event.currentTarget as HTMLInputElement;
              config.overlay.lineWidthPx = toPositiveInt(
                target.value,
                config.overlay.lineWidthPx,
              );
              clearFeedback();
            }}
            disabled={isLoading}
          />
        </div>
        <div>
          <label class="text-sm font-medium text-zinc-700" for="overlay-font"
            >{$t("overlay.fontSize")}</label
          >
          <input
            id="overlay-font"
            type="number"
            min="1"
            class={fieldClass}
            value={config.overlay.font.sizePx}
            oninput={(event) => {
              const target = event.currentTarget as HTMLInputElement;
              config.overlay.font.sizePx = toPositiveInt(
                target.value,
                config.overlay.font.sizePx,
              );
              clearFeedback();
            }}
            disabled={isLoading}
          />
        </div>
        <div>
          <label
            class="text-sm font-medium text-zinc-700"
            for="overlay-show-grid">{$t("overlay.showGrid")}</label
          >
          <input
            id="overlay-show-grid"
            type="checkbox"
            class="mt-3 h-4 w-4 rounded border-zinc-300 text-zinc-900 focus:ring-zinc-900/30"
            bind:checked={config.overlay.showGrid}
            onchange={clearFeedback}
            disabled={isLoading}
          />
        </div>
        <div>
          <label
            class="text-sm font-medium text-zinc-700"
            for="overlay-show-diagonals">{$t("overlay.showDiagonals")}</label
          >
          <input
            id="overlay-show-diagonals"
            type="checkbox"
            class="mt-3 h-4 w-4 rounded border-zinc-300 text-zinc-900 focus:ring-zinc-900/30"
            bind:checked={config.overlay.showDiagonals}
            onchange={clearFeedback}
            disabled={isLoading}
          />
        </div>
        <div>
          <label
            class="text-sm font-medium text-zinc-700"
            for="overlay-mask-color">{$t("overlay.maskColor")}</label
          >
          <input
            id="overlay-mask-color"
            class={fieldClass}
            bind:value={config.overlay.maskColor}
            oninput={clearFeedback}
            disabled={isLoading}
          />
        </div>
        <div>
          <label
            class="text-sm font-medium text-zinc-700"
            for="overlay-line-color">{$t("overlay.lineColor")}</label
          >
          <input
            id="overlay-line-color"
            class={fieldClass}
            bind:value={config.overlay.lineColor}
            oninput={clearFeedback}
            disabled={isLoading}
          />
        </div>
        <div>
          <label
            class="text-sm font-medium text-zinc-700"
            for="overlay-text-color">{$t("overlay.textColor")}</label
          >
          <input
            id="overlay-text-color"
            class={fieldClass}
            bind:value={config.overlay.textColor}
            oninput={clearFeedback}
            disabled={isLoading}
          />
        </div>
        <div>
          <label
            class="text-sm font-medium text-zinc-700"
            for="overlay-font-family">{$t("overlay.fontFamily")}</label
          >
          <input
            id="overlay-font-family"
            class={fieldClass}
            bind:value={config.overlay.font.family}
            oninput={clearFeedback}
            disabled={isLoading}
          />
        </div>
      </div>
    </section>

    <p class="text-xs text-zinc-500">{$t("footer.note")}</p>
  </div>
</main>
