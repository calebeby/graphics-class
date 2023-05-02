#version 300 es
// Declare the default precision:
// https://developer.mozilla.org/en-US/docs/Web/API/WebGL_API/WebGL_best_practices#implicit_defaults
precision highp float;

out vec2 vs_uv;
out vec3 vs_vertex;
out vec3 vs_normal;

uniform mat4 camera_transform;
uniform mat4 transform;
uniform vec3 light_position;
uniform vec3 camera_position;
layout(location = 0) in vec4 obj_vertex;
layout(location = 1) in vec4 obj_normal;
layout(location = 2) in vec2 obj_uv;

void main(void) {
  gl_Position = camera_transform * transform * obj_vertex;

  vs_normal = -normalize(mat3(transform) * vec3(obj_normal));
  vs_uv = obj_uv;
  vs_vertex = vec3(transform * obj_vertex);
}
