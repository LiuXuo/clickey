<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import defaultConfig from "$lib/shared/default-config.json";
  import type { AppConfig } from "$lib/core";

  let configText = $state(JSON.stringify(defaultConfig, null, 2));
  let status = $state("");
  let error = $state("");

  async function applyConfig() {
    error = "";
    status = "Applying...";
    try {
      const parsed = JSON.parse(configText) as AppConfig;
      await invoke("apply_config", { config: parsed });
      status = "Applied";
    } catch (err) {
      status = "";
      error = err instanceof Error ? err.message : String(err);
    }
  }

  onMount(() => {
    void applyConfig();
  });
</script>

<main>
  <header>
    <h1>Clickey Settings</h1>
    <p class="subtitle">Edit configuration JSON and apply it.</p>
  </header>

  <section class="panel">
    <textarea bind:value={configText} spellcheck="false"></textarea>
    <div class="actions">
      <button type="button" on:click={applyConfig}>Apply Config</button>
      {#if status}
        <span class="status">{status}</span>
      {/if}
      {#if error}
        <span class="error">{error}</span>
      {/if}
    </div>
  </section>

  <p class="hint">
    Settings only edits configuration and applies it. Overlay runtime state is
    managed elsewhere.
  </p>
</main>

<style>
  :global(body) {
    margin: 0;
    font-family: "Segoe UI", system-ui, sans-serif;
    background: #f4f5f7;
    color: #1f2328;
  }

  main {
    max-width: 960px;
    margin: 0 auto;
    padding: 32px 24px 40px;
  }

  header {
    margin-bottom: 20px;
  }

  h1 {
    margin: 0 0 6px;
    font-size: 28px;
    font-weight: 600;
  }

  .subtitle {
    margin: 0;
    color: #5b6270;
  }

  .panel {
    background: #ffffff;
    border: 1px solid #d8dee4;
    border-radius: 12px;
    padding: 16px;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.06);
  }

  textarea {
    width: 100%;
    min-height: 320px;
    resize: vertical;
    border: 1px solid #c9d0d8;
    border-radius: 8px;
    padding: 12px;
    font-family: "Consolas", "SFMono-Regular", monospace;
    font-size: 13px;
    line-height: 1.5;
    background: #f9fafb;
    color: #111827;
  }

  .actions {
    display: flex;
    align-items: center;
    gap: 12px;
    margin-top: 12px;
  }

  button {
    border: none;
    background: #1f6feb;
    color: #ffffff;
    padding: 10px 16px;
    border-radius: 8px;
    font-weight: 600;
    cursor: pointer;
  }

  button:hover {
    background: #1a5fd0;
  }

  .status {
    color: #0b5f31;
    font-weight: 600;
  }

  .error {
    color: #c01f1f;
    font-weight: 600;
  }

  .hint {
    margin-top: 16px;
    color: #5b6270;
    font-size: 13px;
  }
</style>
