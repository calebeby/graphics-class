import DOMMatrix from "@thednp/dommatrix";
// To fix astro's running components in node, which doesn't have DOMMatrix built-in
globalThis.DOMMatrix = DOMMatrix as any;
DOMMatrix.prototype.inverse = function () {
  // TODO: hacky. It is there because the polyfill does not implement .inverse
  return this;
};
