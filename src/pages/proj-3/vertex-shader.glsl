#version 300 es
// Declare the default precision:
// https://developer.mozilla.org/en-US/docs/Web/API/WebGL_API/WebGL_best_practices#implicit_defaults
precision highp float;

out vec4 vs_color;

uniform mat4 transform;
layout(location = 0) in vec4 obj_vertex;
layout(location = 1) in vec4 obj_normal;

void main(void) {
  float scale = 0.4;
  vec4 base = transform * obj_vertex;

  gl_Position = base;

  float dist = dot(vec4(-0.5, 0.3, -0.5, 0), base) * length(base);
  vs_color = vec4(0.5 * dist, 2.0 * dist, 2.0 * dist, 1);
}
