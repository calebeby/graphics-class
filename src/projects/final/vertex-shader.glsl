#version 300 es
// Declare the default precision:
// https://developer.mozilla.org/en-US/docs/Web/API/WebGL_API/WebGL_best_practices#implicit_defaults
precision highp float;

out vec3 vs_vertex;
out vec4 vs_color;

uniform mat4 camera_transform;
layout(location = 0) in vec4 obj_vertex;
layout(location = 1) in vec4 obj_color;

void main(void) {
  gl_Position = camera_transform * obj_vertex;

  vs_vertex = vec3(obj_vertex);
  vs_color = obj_color;
}
