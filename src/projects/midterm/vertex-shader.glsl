#version 300 es
// Declare the default precision:
// https://developer.mozilla.org/en-US/docs/Web/API/WebGL_API/WebGL_best_practices#implicit_defaults
precision highp float;

out vec4 vs_color;

uniform mat4 transform;
layout(location = 0) in vec4 obj_vertex;
layout(location = 1) in vec4 obj_normal;

void main(void) {
  gl_Position = transform * obj_vertex;

  float origin_dist = 2.0 * obj_vertex[2];
  vs_color = vec4(origin_dist, origin_dist, origin_dist, 1);
}
