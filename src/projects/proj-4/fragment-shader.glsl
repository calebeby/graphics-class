#version 300 es
// Declare the default precision:
// https://developer.mozilla.org/en-US/docs/Web/API/WebGL_API/WebGL_best_practices#implicit_defaults
precision highp float;

uniform sampler2D twoDTex;

in vec4 vs_color;
in vec2 vs_uv;
out vec4 color;

void main(void) {
  // vec4 tex = texture(twoDTex, vs_uv * vec2(1.0, 1.0));
  color = vs_color;
}
