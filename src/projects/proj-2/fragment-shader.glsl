#version 300 es
// Declare the default precision:
// https://developer.mozilla.org/en-US/docs/Web/API/WebGL_API/WebGL_best_practices#implicit_defaults
precision highp float;

in vec4 vs_color;
out vec4 color;

void main(void) { color = vs_color; }
