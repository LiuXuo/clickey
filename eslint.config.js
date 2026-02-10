import js from "@eslint/js";
import globals from "globals";
import svelte from "eslint-plugin-svelte";
import svelteParser from "svelte-eslint-parser";
import tseslint from "typescript-eslint";

const svelteGlobals = {
  $state: "readonly",
  $derived: "readonly",
  $effect: "readonly",
  $props: "readonly",
  $bindable: "readonly",
  $inspect: "readonly",
};

export default [
  {
    ignores: [
      "node_modules/**",
      ".svelte-kit/**",
      "dist/**",
      "build/**",
      "coverage/**",
      "src-tauri/target/**",
    ],
  },
  {
    settings: {
      svelte: {
        ignoreWarnings: ["event_directive_deprecated"],
      },
    },
  },
  {
    files: ["**/*.{js,cjs,mjs,ts}"],
    languageOptions: {
      globals: {
        ...globals.browser,
        ...globals.node,
      },
    },
  },
  js.configs.recommended,
  ...tseslint.configs.recommended,
  ...svelte.configs["flat/recommended"],
  {
    files: ["**/*.svelte"],
    languageOptions: {
      parser: svelteParser,
      parserOptions: {
        parser: tseslint.parser,
        sourceType: "module",
        ecmaVersion: "latest",
      },
      globals: {
        ...globals.browser,
        ...globals.node,
        ...svelteGlobals,
      },
    },
  },
  {
    files: ["**/*.svelte"],
    rules: {
      "svelte/valid-compile": ["error", { ignoreWarnings: true }],
    },
  },
];
