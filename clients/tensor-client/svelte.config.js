import { vitePreprocess } from "@sveltejs/vite-plugin-svelte";
import path from "path";
export default {
  // Consult https://svelte.dev/docs#compile-time-svelte-preprocess
  // for more information about preprocessors
  preprocess: vitePreprocess(),
  kit: {
    // ... other config
    alias: {
      "$contexts/*": path.resolve("./src/contexts/*"),
      "$lib/*": path.resolve("./src/lib/*"),
      "$types/*": path.resolve("./src/types/*"),
    },
  },
};
