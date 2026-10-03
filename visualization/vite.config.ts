import tailwindcss from "@tailwindcss/vite";
import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";

export default defineConfig({
  plugins: [tailwindcss(), react()],
  optimizeDeps: {
    // The settings browser package loads its wasm-bindgen module through
    // `new URL("settings_wasm_bg.wasm", import.meta.url)`. Dev-server pre-bundling would move
    // the JS into `.vite/deps` without the wasm asset, so the settings session never initializes.
    // Excluding the package also serves its `/react` entry unbundled, so its shared UI import
    // (which pulls CommonJS-only transitive dependencies) must still be pre-bundled explicitly.
    exclude: ["@moritzbrantner/settings-browser"],
    include: ["@moritzbrantner/ui/stable"],
  },
});
