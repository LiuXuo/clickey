/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ["./src/**/*.{html,js,svelte,ts}"],
  theme: {
    extend: {
      fontFamily: {
        sans: ["Space Grotesk", "Noto Sans SC", "Segoe UI", "sans-serif"],
      },
    },
  },
  plugins: [],
};
