<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import { initLocale, locale, setLocale, t, type Locale } from "$lib/i18n";
  import defaultConfig from "$lib/shared/default-config.json";
  import type { AppConfig, Preset } from "$lib/core";

  const initialConfig = JSON.parse(
    JSON.stringify(defaultConfig),
  ) as AppConfig;

  const fieldClass =
    "mt-2 w-full rounded-lg border border-zinc-300 bg-white px-3 py-2 text-sm text-zinc-900 shadow-sm focus:border-zinc-900 focus:outline-none focus:ring-2 focus:ring-zinc-900/20 disabled:cursor-not-allowed disabled:bg-zinc-100";
  const textAreaClass =
    "mt-2 w-full min-h-[110px] rounded-lg border border-zinc-300 bg-white px-3 py-2 text-xs leading-relaxed text-zinc-900 shadow-sm focus:border-zinc-900 focus:outline-none focus:ring-2 focus:ring-zinc-900/20 disabled:cursor-not-allowed disabled:bg-zinc-100";
  const inlineFieldClass =
    "w-full rounded-lg border border-zinc-300 bg-white px-3 py-2 text-sm text-zinc-900 shadow-sm focus:border-zinc-900 focus:outline-none focus:ring-2 focus:ring-zinc-900/20 disabled:cursor-not-allowed disabled:bg-zinc-100";
  const colorInputClass =
    "h-10 w-12 rounded-md border border-zinc-300 bg-white p-0.5 shadow-sm";
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
  let isApplying = $state(false);
  let isLoading = $state(true);
  let isResetting = $state(false);

  const activePreset = $derived(
    config.presets.find((preset) => preset.id === config.activePresetId),
  );
  const activePresetIndex = $derived(
    config.presets.findIndex((preset) => preset.id === config.activePresetId),
  );

  function clearFeedback() {
    status = "";
    error = "";
  }

  function onLocaleChange(event: Event) {
    const target = event.currentTarget as HTMLSelectElement;
    setLocale(target.value as Locale);
  }

  function onPresetChange(event: Event) {
    const target = event.currentTarget as HTMLSelectElement;
    config.activePresetId = target.value;
    clearFeedback();
  }

  function updateActivePresetName(event: Event) {
    if (activePresetIndex < 0) {
      return;
    }
    const target = event.currentTarget as HTMLInputElement;
    config.presets[activePresetIndex].name = target.value;
    clearFeedback();
  }

  function clonePreset(preset: Preset): Preset {
    return JSON.parse(JSON.stringify(preset)) as Preset;
  }

  function createUniquePresetId(base: string): string {
    let candidate = base;
    let index = 2;
    const ids = new Set(config.presets.map((preset) => preset.id));
    while (ids.has(candidate)) {
      candidate = `${base}-${index}`;
      index += 1;
    }
    return candidate;
  }

  function duplicateActivePreset() {
    if (!activePreset) {
      return;
    }

    const copy = clonePreset(activePreset);
    const baseId = `${activePreset.id}-copy`;
    copy.id = createUniquePresetId(baseId);
    copy.name = $t("presets.copyName", { name: activePreset.name });
    config.presets = [...config.presets, copy];
    config.activePresetId = copy.id;
    clearFeedback();
  }

  function removeActivePreset() {
    if (!activePreset) {
      return;
    }

    if (config.presets.length <= 1) {
      error = $t("errors.presetRequired");
      return;
    }

    if (
      !confirm(
        $t("errors.removePresetConfirm", { name: activePreset.name }),
      )
    ) {
      return;
    }

    const nextPresets = config.presets.filter(
      (preset) => preset.id !== activePreset.id,
    );
    config.presets = nextPresets;
    config.activePresetId = nextPresets[0]?.id ?? "";
    clearFeedback();
  }

  function switchLayerMode(index: number, mode: "single" | "combo") {
    if (activePresetIndex < 0) {
      return;
    }
    const preset = config.presets[activePresetIndex];
    const layer = preset.layers[index];
    if (!layer || layer.mode === mode) {
      return;
    }

    if (mode === "single") {
      const defaults = getDefaultSingleLayer();
      const pooled =
        layer.mode === "combo"
          ? [...layer.stage0.keys, ...layer.stage1.keys]
          : layer.keys;
      const nextLayer = {
        mode: "single" as const,
        rows: defaults.rows,
        cols: defaults.cols,
        keys: fillKeys(pooled, defaults.rows * defaults.cols),
      };
      const nextLayers = [...preset.layers];
      nextLayers[index] = nextLayer;
      preset.layers = nextLayers;
      clearFeedback();
      return;
    }

    const defaults = getDefaultComboLayer();
    const pooled =
      layer.mode === "single"
        ? layer.keys
        : [...layer.stage0.keys, ...layer.stage1.keys];
    const nextLayer = {
      mode: "combo" as const,
      stage0: {
        ...defaults.stage0,
        keys: fillKeys(
          pooled,
          defaults.stage0.rows * defaults.stage0.cols,
        ),
      },
      stage1: {
        ...defaults.stage1,
        keys: fillKeys(
          pooled,
          defaults.stage1.rows * defaults.stage1.cols,
        ),
      },
    };
    const nextLayers = [...preset.layers];
    nextLayers[index] = nextLayer;
    preset.layers = nextLayers;
    clearFeedback();
  }

  function toPositiveInt(value: string, fallback: number): number {
    const parsed = Number.parseInt(value, 10);
    if (Number.isFinite(parsed) && parsed > 0) {
      return parsed;
    }
    return fallback;
  }

  function clampInt(value: string, min: number, max: number, fallback: number) {
    const parsed = Number.parseInt(value, 10);
    if (!Number.isFinite(parsed)) {
      return fallback;
    }
    return Math.min(Math.max(parsed, min), max);
  }

  function formatKeys(keys: string[]): string {
    return keys.join(" ");
  }

  function parseKeys(value: string): string[] {
    return value.split(/[\s,]+/).filter(Boolean);
  }

  function fillKeys(existing: string[], count: number): string[] {
    const trimmed = existing.filter((key) => key.trim().length > 0);
    const result = trimmed.slice(0, count);
    for (let index = result.length; index < count; index += 1) {
      result.push(keyPool[index % keyPool.length]);
    }
    return result;
  }

  function getDefaultSingleLayer() {
    const preset = initialConfig.presets[0];
    const candidate = preset?.layers.find((layer) => layer.mode === "single");
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
    const preset = initialConfig.presets[0];
    const candidate = preset?.layers.find((layer) => layer.mode === "combo");
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

  function addSingleLayer() {
    if (activePresetIndex < 0) {
      return;
    }
    const preset = config.presets[activePresetIndex];
    const base = getDefaultSingleLayer();
    const keys = fillKeys(base.keys, base.rows * base.cols);
    preset.layers = [
      ...preset.layers,
      { mode: "single", rows: base.rows, cols: base.cols, keys },
    ];
    clearFeedback();
  }

  function addComboLayer() {
    if (activePresetIndex < 0) {
      return;
    }
    const preset = config.presets[activePresetIndex];
    const base = getDefaultComboLayer();
    const stage0Keys = fillKeys(base.stage0.keys, base.stage0.rows * base.stage0.cols);
    const stage1Keys = fillKeys(base.stage1.keys, base.stage1.rows * base.stage1.cols);
    preset.layers = [
      ...preset.layers,
      {
        mode: "combo",
        stage0: { ...base.stage0, keys: stage0Keys },
        stage1: { ...base.stage1, keys: stage1Keys },
      },
    ];
    clearFeedback();
  }

  function moveLayer(index: number, direction: -1 | 1) {
    if (activePresetIndex < 0) {
      return;
    }
    const preset = config.presets[activePresetIndex];
    const nextIndex = index + direction;
    if (nextIndex < 0 || nextIndex >= preset.layers.length) {
      return;
    }
    const nextLayers = [...preset.layers];
    const [moved] = nextLayers.splice(index, 1);
    nextLayers.splice(nextIndex, 0, moved);
    preset.layers = nextLayers;
    clearFeedback();
  }

  function removeLayer(index: number) {
    if (activePresetIndex < 0) {
      return;
    }
    const preset = config.presets[activePresetIndex];
    if (preset.layers.length <= 1) {
      error = $t("errors.presetLayerRequired");
      return;
    }
    if (!confirm($t("errors.removeLayerConfirm", { index: index + 1 }))) {
      return;
    }
    preset.layers = preset.layers.filter((_, layerIndex) => layerIndex !== index);
    clearFeedback();
  }

  function autoFitSingleLayer(index: number) {
    if (activePresetIndex < 0) {
      return;
    }
    const preset = config.presets[activePresetIndex];
    const layer = preset.layers[index];
    if (!layer || layer.mode !== "single") {
      return;
    }
    layer.keys = fillKeys(layer.keys, layer.rows * layer.cols);
    clearFeedback();
  }

  function autoFitComboStage(index: number, stage: 0 | 1) {
    if (activePresetIndex < 0) {
      return;
    }
    const preset = config.presets[activePresetIndex];
    const layer = preset.layers[index];
    if (!layer || layer.mode !== "combo") {
      return;
    }
    const stageConfig = stage === 0 ? layer.stage0 : layer.stage1;
    stageConfig.keys = fillKeys(
      stageConfig.keys,
      stageConfig.rows * stageConfig.cols,
    );
    clearFeedback();
  }
  function updateSingleLayerGrid(
    index: number,
    field: "rows" | "cols",
    event: Event,
  ) {
    if (activePresetIndex < 0) {
      return;
    }
    const preset = config.presets[activePresetIndex];
    const layer = preset.layers[index];
    if (!layer || layer.mode !== "single") {
      return;
    }
    const target = event.currentTarget as HTMLInputElement;
    layer[field] = toPositiveInt(target.value, layer[field]);
    clearFeedback();
  }

  function updateSingleLayerKeys(index: number, event: Event) {
    if (activePresetIndex < 0) {
      return;
    }
    const preset = config.presets[activePresetIndex];
    const layer = preset.layers[index];
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
    if (activePresetIndex < 0) {
      return;
    }
    const preset = config.presets[activePresetIndex];
    const layer = preset.layers[index];
    if (!layer || layer.mode !== "combo") {
      return;
    }
    const target = event.currentTarget as HTMLInputElement;
    const stageConfig = stage === 0 ? layer.stage0 : layer.stage1;
    stageConfig[field] = toPositiveInt(target.value, stageConfig[field]);
    clearFeedback();
  }

  function updateComboStageKeys(
    index: number,
    stage: 0 | 1,
    event: Event,
  ) {
    if (activePresetIndex < 0) {
      return;
    }
    const preset = config.presets[activePresetIndex];
    const layer = preset.layers[index];
    if (!layer || layer.mode !== "combo") {
      return;
    }
    const target = event.currentTarget as HTMLTextAreaElement;
    const stageConfig = stage === 0 ? layer.stage0 : layer.stage1;
    stageConfig.keys = parseKeys(target.value);
    clearFeedback();
  }

  function updateOverlayAlpha(event: Event) {
    const target = event.currentTarget as HTMLInputElement;
    config.overlay.alpha = clampInt(
      target.value,
      0,
      255,
      config.overlay.alpha,
    );
    clearFeedback();
  }

  function updateOverlayLineWidth(event: Event) {
    const target = event.currentTarget as HTMLInputElement;
    config.overlay.lineWidthPx = toPositiveInt(
      target.value,
      config.overlay.lineWidthPx,
    );
    clearFeedback();
  }

  function updateOverlayFontSize(event: Event) {
    const target = event.currentTarget as HTMLInputElement;
    config.overlay.font.sizePx = toPositiveInt(
      target.value,
      config.overlay.font.sizePx,
    );
    clearFeedback();
  }

  function updateNudgeStep(event: Event) {
    const target = event.currentTarget as HTMLInputElement;
    config.nudge.stepPx = toPositiveInt(target.value, config.nudge.stepPx);
    clearFeedback();
  }

  function validateConfig(candidate: AppConfig): string[] {
    const issues: string[] = [];

    if (!candidate.presets.length) {
      issues.push($t("errors.presetsEmpty"));
      return issues;
    }

    const presetIds = new Set<string>();
    for (const preset of candidate.presets) {
      if (!preset.id.trim()) {
        issues.push($t("errors.presetIdEmpty"));
      }
      if (!preset.name.trim()) {
        issues.push($t("errors.presetNameEmpty"));
      }
      if (presetIds.has(preset.id)) {
        issues.push($t("errors.duplicatePresetId", { id: preset.id }));
      }
      presetIds.add(preset.id);
      if (!preset.layers.length) {
        issues.push($t("errors.presetNoLayers", { id: preset.id }));
      }

      preset.layers.forEach((layer, index) => {
        if (layer.mode === "single") {
          const expected = layer.rows * layer.cols;
          if (!layer.rows || !layer.cols) {
            issues.push(
              $t("errors.layerGridInvalid", { id: preset.id, index }),
            );
          }
          if (layer.keys.length !== expected) {
            issues.push(
              $t("errors.layerExpectedKeys", {
                id: preset.id,
                index,
                expected,
              }),
            );
          }
        } else {
          const expected0 = layer.stage0.rows * layer.stage0.cols;
          const expected1 = layer.stage1.rows * layer.stage1.cols;
          if (!layer.stage0.rows || !layer.stage0.cols) {
            issues.push(
              $t("errors.stage0GridInvalid", { id: preset.id, index }),
            );
          }
          if (!layer.stage1.rows || !layer.stage1.cols) {
            issues.push(
              $t("errors.stage1GridInvalid", { id: preset.id, index }),
            );
          }
          if (layer.stage0.keys.length !== expected0) {
            issues.push(
              $t("errors.stage0ExpectedKeys", {
                id: preset.id,
                index,
                expected: expected0,
              }),
            );
          }
          if (layer.stage1.keys.length !== expected1) {
            issues.push(
              $t("errors.stage1ExpectedKeys", {
                id: preset.id,
                index,
                expected: expected1,
              }),
            );
          }
        }
      });
    }

    if (!presetIds.has(candidate.activePresetId)) {
      issues.push($t("errors.activePresetMissing"));
    }

    if (!candidate.hotkeys.activation.leftClick.trim()) {
      issues.push($t("errors.leftHotkeyEmpty"));
    }
    if (!candidate.hotkeys.activation.rightClick.trim()) {
      issues.push($t("errors.rightHotkeyEmpty"));
    }
    if (!candidate.hotkeys.activation.middleClick.trim()) {
      issues.push($t("errors.middleHotkeyEmpty"));
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
      status = $t("status.reset");
    } catch (err) {
      error = err instanceof Error ? err.message : String(err);
    } finally {
      isResetting = false;
    }
  }

  onMount(() => {
    initLocale();
    void (async () => {
      try {
        const loaded = await invoke<AppConfig>("get_config");
        config = loaded;
      } catch (err) {
        error = err instanceof Error ? err.message : String(err);
      } finally {
        isLoading = false;
      }
    })();
  });
</script>

<main class="min-h-screen px-6 py-10">
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
            on:change={onLocaleChange}
            disabled={isLoading}
          >
            <option value="zh-CN">{$t("language.zh")}</option>
            <option value="en-US">{$t("language.en")}</option>
          </select>
        </div>
        <button
          type="button"
          class="inline-flex items-center justify-center rounded-lg bg-zinc-900 px-4 py-2 text-sm font-semibold text-white shadow-sm transition hover:bg-zinc-800 disabled:cursor-not-allowed disabled:bg-zinc-400"
          on:click={applyConfig}
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
          on:click={resetConfig}
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
            {$t("presets.section")}
          </p>
          <h2 class="text-lg font-semibold text-zinc-900">
            {$t("presets.title")}
          </h2>
        </div>
        <p class="text-xs text-zinc-500">{$t("presets.subtitle")}</p>
      </div>

      <div class="mt-6 grid gap-6 md:grid-cols-[1.1fr_0.9fr]">
        <div class="space-y-4">
          <div>
            <label class="text-sm font-medium text-zinc-700" for="preset-select">
              {$t("presets.active")}
            </label>
            <select
              id="preset-select"
              class={fieldClass}
              on:change={onPresetChange}
              value={config.activePresetId}
              disabled={isLoading}
            >
              {#each config.presets as preset}
                <option value={preset.id}>{preset.name}</option>
              {/each}
            </select>
          </div>

          <div>
            <label class="text-sm font-medium text-zinc-700" for="preset-name">
              {$t("presets.name")}
            </label>
            <input
              id="preset-name"
              class={fieldClass}
              value={activePreset?.name ?? ""}
              on:input={updateActivePresetName}
              disabled={isLoading || !activePreset}
              placeholder={$t("presets.namePlaceholder")}
            />
            <p class="mt-2 text-xs text-zinc-500">
              {$t("presets.idLabel")}: {activePreset ? activePreset.id : "-"}
            </p>
          </div>

          <div class="flex flex-wrap items-center gap-3">
            <button
              type="button"
              class="inline-flex items-center justify-center rounded-lg border border-zinc-300 bg-white px-3 py-2 text-xs font-semibold text-zinc-700 shadow-sm transition hover:border-zinc-400 hover:text-zinc-900 disabled:cursor-not-allowed disabled:opacity-60"
              on:click={duplicateActivePreset}
              disabled={isLoading || !activePreset}
            >
              {$t("presets.duplicateActive")}
            </button>
            <button
              type="button"
              class="inline-flex items-center justify-center rounded-lg border border-zinc-300 bg-white px-3 py-2 text-xs font-semibold text-zinc-700 shadow-sm transition hover:border-zinc-400 hover:text-zinc-900 disabled:cursor-not-allowed disabled:opacity-60"
              on:click={removeActivePreset}
              disabled={isLoading || config.presets.length <= 1}
            >
              {$t("presets.removeActive")}
            </button>
          </div>
        </div>

        <div class="space-y-4">
          <div class="rounded-xl border border-zinc-200 bg-white p-4">
            <p class="text-xs uppercase tracking-[0.28em] text-zinc-500">
              {$t("presets.list")}
            </p>
            <div class="mt-3 divide-y divide-zinc-100 text-sm">
              {#each config.presets as preset}
                <div
                  class={`flex items-center justify-between gap-3 py-2 ${preset.id === config.activePresetId ? "text-zinc-900" : "text-zinc-600"}`}
                >
                  <div>
                    <p class="font-medium">{preset.name}</p>
                    <p class="text-xs text-zinc-400">{preset.id}</p>
                  </div>
                  <span class="text-xs text-zinc-500">
                    {$t("presets.layersCount", { count: preset.layers.length })}
                  </span>
                </div>
              {/each}
            </div>
          </div>

          <div class="rounded-xl border border-dashed border-zinc-200 p-4">
            <p class="text-xs uppercase tracking-[0.28em] text-zinc-500">
              {$t("presets.activeSummary")}
            </p>
            {#if activePreset}
              <div class="mt-3 space-y-2 text-sm text-zinc-700">
                {#each activePreset.layers as layer, index}
                  <div class="flex items-center justify-between">
                    <span>{$t("layers.layerLabel", { index: index + 1 })}</span>
                    <span class="text-xs text-zinc-500">
                      {layer.mode === "single"
                        ? `${layer.rows}x${layer.cols}`
                        : `${layer.stage0.rows}x${layer.stage0.cols} / ${layer.stage1.rows}x${layer.stage1.cols}`}
                    </span>
                  </div>
                {/each}
              </div>
            {:else}
              <p class="mt-3 text-sm text-zinc-500">
                {$t("presets.noActive")}
              </p>
            {/if}
          </div>
        </div>
      </div>
    </section>

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
          <label class="text-sm font-medium text-zinc-700" for="nudge-step">
            {$t("nudge.step")}
          </label>
          <input
            id="nudge-step"
            type="number"
            min="1"
            class={fieldClass}
            value={config.nudge.stepPx}
            on:input={updateNudgeStep}
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
            on:click={addSingleLayer}
            disabled={isLoading || !activePreset}
          >
            {$t("layers.addSingle")}
          </button>
          <button
            type="button"
            class="inline-flex items-center justify-center rounded-lg border border-zinc-300 bg-white px-3 py-2 text-xs font-semibold text-zinc-700 shadow-sm transition hover:border-zinc-400 hover:text-zinc-900 disabled:cursor-not-allowed disabled:opacity-60"
            on:click={addComboLayer}
            disabled={isLoading || !activePreset}
          >
            {$t("layers.addCombo")}
          </button>
        </div>
      </div>
      <p class="mt-2 text-xs text-zinc-500">{$t("layers.subtitle")}</p>

      {#if activePreset}
        <div class="mt-6 space-y-4">
          {#each activePreset.layers as layer, index}
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
                    for={`layer-${index}-mode`}
                  >
                    {$t("layers.mode")}
                  </label>
                  <select
                    id={`layer-${index}-mode`}
                    class={fieldClass}
                    value={layer.mode}
                    on:change={(event) =>
                      switchLayerMode(
                        index,
                        (event.currentTarget as HTMLSelectElement)
                          .value as "single" | "combo",
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
                    on:click={() => moveLayer(index, -1)}
                    disabled={isLoading || index === 0}
                  >
                    {$t("layers.moveUp")}
                  </button>
                  <button
                    type="button"
                    class="inline-flex items-center justify-center rounded-lg border border-zinc-300 bg-white px-2.5 py-1 text-[11px] font-semibold text-zinc-700 shadow-sm transition hover:border-zinc-400 hover:text-zinc-900 disabled:cursor-not-allowed disabled:opacity-60"
                    on:click={() => moveLayer(index, 1)}
                    disabled={isLoading || index === activePreset.layers.length - 1}
                  >
                    {$t("layers.moveDown")}
                  </button>
                  <button
                    type="button"
                    class="inline-flex items-center justify-center rounded-lg border border-zinc-300 bg-white px-2.5 py-1 text-[11px] font-semibold text-zinc-700 shadow-sm transition hover:border-zinc-400 hover:text-zinc-900 disabled:cursor-not-allowed disabled:opacity-60"
                    on:click={() => removeLayer(index)}
                    disabled={isLoading || activePreset.layers.length <= 1}
                  >
                    {$t("layers.remove")}
                  </button>
                </div>
              </div>
              <p class="mt-2 text-xs text-zinc-500">
                {$t("layers.expectedKeys")}
                {layer.mode === "single"
                  ? ` ${layer.rows * layer.cols}`
                  : ` ${layer.stage0.rows * layer.stage0.cols}`}
                {layer.mode === "combo"
                  ? ` / ${layer.stage1.rows * layer.stage1.cols}`
                  : ""}
              </p>

              {#if layer.mode === "single"}
                <div class="mt-4 grid gap-4 md:grid-cols-2">
                  <div>
                    <label
                      class="text-sm font-medium text-zinc-700"
                      for={`layer-${index}-rows`}
                    >
                      {$t("layers.rows")}
                    </label>
                    <input
                      id={`layer-${index}-rows`}
                      type="number"
                      min="1"
                      class={fieldClass}
                      value={layer.rows}
                      on:input={(event) =>
                        updateSingleLayerGrid(index, "rows", event)}
                      disabled={isLoading}
                    />
                  </div>
                  <div>
                    <label
                      class="text-sm font-medium text-zinc-700"
                      for={`layer-${index}-cols`}
                    >
                      {$t("layers.columns")}
                    </label>
                    <input
                      id={`layer-${index}-cols`}
                      type="number"
                      min="1"
                      class={fieldClass}
                      value={layer.cols}
                      on:input={(event) =>
                        updateSingleLayerGrid(index, "cols", event)}
                      disabled={isLoading}
                    />
                  </div>
                </div>
                <div class="mt-4">
                  <div class="flex items-center justify-between">
                    <label
                      class="text-sm font-medium text-zinc-700"
                      for={`layer-${index}-keys`}
                    >
                      {$t("layers.keysHint")}
                    </label>
                    <button
                      type="button"
                      class="inline-flex items-center justify-center rounded-lg border border-zinc-300 bg-white px-2.5 py-1 text-[11px] font-semibold text-zinc-700 shadow-sm transition hover:border-zinc-400 hover:text-zinc-900 disabled:cursor-not-allowed disabled:opacity-60"
                      on:click={() => autoFitSingleLayer(index)}
                      disabled={isLoading}
                    >
                      {$t("layers.autoFit")}
                    </button>
                  </div>
                  <textarea
                    id={`layer-${index}-keys`}
                    class={textAreaClass}
                    value={formatKeys(layer.keys)}
                    on:input={(event) => updateSingleLayerKeys(index, event)}
                    disabled={isLoading}
                  ></textarea>
                  <p class="mt-2 text-xs text-zinc-500">
                    {$t("layers.currentExpected", {
                      current: layer.keys.length,
                      expected: layer.rows * layer.cols,
                    })}
                  </p>
                </div>
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
                        >
                          {$t("layers.rows")}
                        </label>
                        <input
                          id={`layer-${index}-stage0-rows`}
                          type="number"
                          min="1"
                          class={fieldClass}
                          value={layer.stage0.rows}
                          on:input={(event) =>
                            updateComboStageGrid(index, 0, "rows", event)}
                          disabled={isLoading}
                        />
                      </div>
                      <div>
                        <label
                          class="text-sm font-medium text-zinc-700"
                          for={`layer-${index}-stage0-cols`}
                        >
                          {$t("layers.columns")}
                        </label>
                        <input
                          id={`layer-${index}-stage0-cols`}
                          type="number"
                          min="1"
                          class={fieldClass}
                          value={layer.stage0.cols}
                          on:input={(event) =>
                            updateComboStageGrid(index, 0, "cols", event)}
                          disabled={isLoading}
                        />
                      </div>
                    </div>
                    <div class="mt-3">
                      <div class="flex items-center justify-between">
                        <label
                          class="text-sm font-medium text-zinc-700"
                          for={`layer-${index}-stage0-keys`}
                        >
                          {$t("layers.keys")}
                        </label>
                        <button
                          type="button"
                          class="inline-flex items-center justify-center rounded-lg border border-zinc-300 bg-white px-2.5 py-1 text-[11px] font-semibold text-zinc-700 shadow-sm transition hover:border-zinc-400 hover:text-zinc-900 disabled:cursor-not-allowed disabled:opacity-60"
                          on:click={() => autoFitComboStage(index, 0)}
                          disabled={isLoading}
                        >
                          {$t("layers.autoFit")}
                        </button>
                      </div>
                      <textarea
                        id={`layer-${index}-stage0-keys`}
                        class={textAreaClass}
                        value={formatKeys(layer.stage0.keys)}
                        on:input={(event) =>
                          updateComboStageKeys(index, 0, event)}
                        disabled={isLoading}
                      ></textarea>
                      <p class="mt-2 text-xs text-zinc-500">
                        {$t("layers.currentExpected", {
                          current: layer.stage0.keys.length,
                          expected: layer.stage0.rows * layer.stage0.cols,
                        })}
                      </p>
                    </div>
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
                        >
                          {$t("layers.rows")}
                        </label>
                        <input
                          id={`layer-${index}-stage1-rows`}
                          type="number"
                          min="1"
                          class={fieldClass}
                          value={layer.stage1.rows}
                          on:input={(event) =>
                            updateComboStageGrid(index, 1, "rows", event)}
                          disabled={isLoading}
                        />
                      </div>
                      <div>
                        <label
                          class="text-sm font-medium text-zinc-700"
                          for={`layer-${index}-stage1-cols`}
                        >
                          {$t("layers.columns")}
                        </label>
                        <input
                          id={`layer-${index}-stage1-cols`}
                          type="number"
                          min="1"
                          class={fieldClass}
                          value={layer.stage1.cols}
                          on:input={(event) =>
                            updateComboStageGrid(index, 1, "cols", event)}
                          disabled={isLoading}
                        />
                      </div>
                    </div>
                    <div class="mt-3">
                      <div class="flex items-center justify-between">
                        <label
                          class="text-sm font-medium text-zinc-700"
                          for={`layer-${index}-stage1-keys`}
                        >
                          {$t("layers.keys")}
                        </label>
                        <button
                          type="button"
                          class="inline-flex items-center justify-center rounded-lg border border-zinc-300 bg-white px-2.5 py-1 text-[11px] font-semibold text-zinc-700 shadow-sm transition hover:border-zinc-400 hover:text-zinc-900 disabled:cursor-not-allowed disabled:opacity-60"
                          on:click={() => autoFitComboStage(index, 1)}
                          disabled={isLoading}
                        >
                          {$t("layers.autoFit")}
                        </button>
                      </div>
                      <textarea
                        id={`layer-${index}-stage1-keys`}
                        class={textAreaClass}
                        value={formatKeys(layer.stage1.keys)}
                        on:input={(event) =>
                          updateComboStageKeys(index, 1, event)}
                        disabled={isLoading}
                      ></textarea>
                      <p class="mt-2 text-xs text-zinc-500">
                        {$t("layers.currentExpected", {
                          current: layer.stage1.keys.length,
                          expected: layer.stage1.rows * layer.stage1.cols,
                        })}
                      </p>
                    </div>
                  </div>
                </div>
              {/if}
            </div>
          {/each}
        </div>
      {:else}
        <p class="mt-4 text-sm text-zinc-500">{$t("presets.noActive")}</p>
      {/if}
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
        <p class="text-xs text-zinc-500">{$t("hotkeys.subtitle")}</p>
      </div>

      <div class="mt-6 grid gap-6 md:grid-cols-2">
        <div class="space-y-4">
          <h3 class="text-sm font-semibold text-zinc-800">
            {$t("hotkeys.activation")}
          </h3>
          <div>
            <label class="text-sm font-medium text-zinc-700" for="hotkey-left">
              {$t("hotkeys.leftClick")}
            </label>
            <input
              id="hotkey-left"
              class={fieldClass}
              bind:value={config.hotkeys.activation.leftClick}
              on:input={clearFeedback}
              disabled={isLoading}
            />
          </div>
          <div>
            <label class="text-sm font-medium text-zinc-700" for="hotkey-right">
              {$t("hotkeys.rightClick")}
            </label>
            <input
              id="hotkey-right"
              class={fieldClass}
              bind:value={config.hotkeys.activation.rightClick}
              on:input={clearFeedback}
              disabled={isLoading}
            />
          </div>
          <div>
            <label class="text-sm font-medium text-zinc-700" for="hotkey-middle">
              {$t("hotkeys.middleClick")}
            </label>
            <input
              id="hotkey-middle"
              class={fieldClass}
              bind:value={config.hotkeys.activation.middleClick}
              on:input={clearFeedback}
              disabled={isLoading}
            />
          </div>
        </div>

        <div class="space-y-4">
          <h3 class="text-sm font-semibold text-zinc-800">
            {$t("hotkeys.controls")}
          </h3>
          <div>
            <label class="text-sm font-medium text-zinc-700" for="hotkey-cancel">
              {$t("hotkeys.cancel")}
            </label>
            <input
              id="hotkey-cancel"
              class={fieldClass}
              bind:value={config.hotkeys.controls.cancel}
              on:input={clearFeedback}
              disabled={isLoading}
            />
          </div>
          <div>
            <label class="text-sm font-medium text-zinc-700" for="hotkey-undo">
              {$t("hotkeys.undo")}
            </label>
            <input
              id="hotkey-undo"
              class={fieldClass}
              bind:value={config.hotkeys.controls.undo}
              on:input={clearFeedback}
              disabled={isLoading}
            />
          </div>
          <div>
            <label
              class="text-sm font-medium text-zinc-700"
              for="hotkey-direct"
            >
              {$t("hotkeys.directClick")}
            </label>
            <input
              id="hotkey-direct"
              class={fieldClass}
              bind:value={config.hotkeys.controls.directClick}
              on:input={clearFeedback}
              disabled={isLoading}
            />
          </div>
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
        <p class="text-xs text-zinc-500">{$t("overlay.subtitle")}</p>
      </div>

      <div class="mt-6 grid gap-6 md:grid-cols-3">
        <div>
          <label class="text-sm font-medium text-zinc-700" for="overlay-alpha">
            {$t("overlay.alpha")}
          </label>
          <input
            id="overlay-alpha"
            type="number"
            min="0"
            max="255"
            class={fieldClass}
            value={config.overlay.alpha}
            on:input={updateOverlayAlpha}
            disabled={isLoading}
          />
        </div>
        <div>
          <label class="text-sm font-medium text-zinc-700" for="overlay-line">
            {$t("overlay.lineWidth")}
          </label>
          <input
            id="overlay-line"
            type="number"
            min="1"
            class={fieldClass}
            value={config.overlay.lineWidthPx}
            on:input={updateOverlayLineWidth}
            disabled={isLoading}
          />
        </div>
        <div>
          <label class="text-sm font-medium text-zinc-700" for="overlay-font">
            {$t("overlay.fontSize")}
          </label>
          <input
            id="overlay-font"
            type="number"
            min="1"
            class={fieldClass}
            value={config.overlay.font.sizePx}
            on:input={updateOverlayFontSize}
            disabled={isLoading}
          />
        </div>
        <div class="md:col-span-3">
          <div class="grid gap-4 sm:grid-cols-2">
            <label
              class="flex items-center gap-3 text-sm font-medium text-zinc-700"
              for="overlay-show-grid"
            >
              <input
                id="overlay-show-grid"
                type="checkbox"
                class="h-4 w-4 rounded border-zinc-300 text-zinc-900 focus:ring-zinc-900/30"
                bind:checked={config.overlay.showGrid}
                on:change={clearFeedback}
                disabled={isLoading}
              />
              {$t("overlay.showGrid")}
            </label>
            <label
              class="flex items-center gap-3 text-sm font-medium text-zinc-700"
              for="overlay-show-diagonals"
            >
              <input
                id="overlay-show-diagonals"
                type="checkbox"
                class="h-4 w-4 rounded border-zinc-300 text-zinc-900 focus:ring-zinc-900/30"
                bind:checked={config.overlay.showDiagonals}
                on:change={clearFeedback}
                disabled={isLoading}
              />
              {$t("overlay.showDiagonals")}
            </label>
          </div>
        </div>
        <div>
          <label
            class="text-sm font-medium text-zinc-700"
            for="overlay-mask"
          >
            {$t("overlay.maskColor")}
          </label>
          <div class="mt-2 flex items-center gap-3">
            <input
              id="overlay-mask"
              type="color"
              class={colorInputClass}
              bind:value={config.overlay.maskColor}
              on:input={clearFeedback}
              disabled={isLoading}
            />
            <input
              class={inlineFieldClass}
              bind:value={config.overlay.maskColor}
              on:input={clearFeedback}
              disabled={isLoading}
            />
          </div>
        </div>
        <div>
          <label
            class="text-sm font-medium text-zinc-700"
            for="overlay-line-color"
          >
            {$t("overlay.lineColor")}
          </label>
          <div class="mt-2 flex items-center gap-3">
            <input
              id="overlay-line-color"
              type="color"
              class={colorInputClass}
              bind:value={config.overlay.lineColor}
              on:input={clearFeedback}
              disabled={isLoading}
            />
            <input
              class={inlineFieldClass}
              bind:value={config.overlay.lineColor}
              on:input={clearFeedback}
              disabled={isLoading}
            />
          </div>
        </div>
        <div>
          <label
            class="text-sm font-medium text-zinc-700"
            for="overlay-text-color"
          >
            {$t("overlay.textColor")}
          </label>
          <div class="mt-2 flex items-center gap-3">
            <input
              id="overlay-text-color"
              type="color"
              class={colorInputClass}
              bind:value={config.overlay.textColor}
              on:input={clearFeedback}
              disabled={isLoading}
            />
            <input
              class={inlineFieldClass}
              bind:value={config.overlay.textColor}
              on:input={clearFeedback}
              disabled={isLoading}
            />
          </div>
        </div>
        <div class="md:col-span-3">
          <label
            class="text-sm font-medium text-zinc-700"
            for="overlay-font-family"
          >
            {$t("overlay.fontFamily")}
          </label>
          <input
            id="overlay-font-family"
            class={fieldClass}
            bind:value={config.overlay.font.family}
            on:input={clearFeedback}
            disabled={isLoading}
          />
        </div>
      </div>
    </section>

    <p class="text-xs text-zinc-500">
      {$t("footer.note")}
    </p>
  </div>
</main>
