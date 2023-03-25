#version 300 es
// Declare the default precision:
// https://developer.mozilla.org/en-US/docs/Web/API/WebGL_API/WebGL_best_practices#implicit_defaults
precision highp float;
out vec4 color;

void main(void) { color = vec4(1.0, 0.0, 1.0, 1.0); }
