import { defineConfig } from "astro/config";
import preact from "@astrojs/preact";

// https://astro.build/config
import mdx from "@astrojs/mdx";

// https://astro.build/config
export default defineConfig({
  // Enable Preact to support Preact JSX components.
  integrations: [preact(), mdx()],
  vite: {
    // eslint-disable-next-line @typescript-eslint/naming-convention
    optimizeDeps: {
      entries: [],
      include: ["preact"],
      // disabled: true,
    },
  },
  // vite: {
  //   plugins: [
  //     {
  //       enforce: "pre",
  //       name: "remove-bad-files",
  //       // eslint-disable-next-line @typescript-eslint/naming-convention
  //       async resolveId(source, importer, opts) {
  //         console.log("resolve", source, importer);
  //       },
  //     },
  //   ],
  // },
});
