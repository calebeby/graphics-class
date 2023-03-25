/// <reference path="../.astro/types.d.ts" />
/// <reference types="astro/client" />
declare module "@thednp/dommatrix/dist/dommatrix.mjs" {
  export default DOMMatrix;
}

declare module "*.wasm?init" {
  const wasm_init: () => Promise<{ exports: Record<string, any> }>;
  export default wasm_init;
}
