#version 300 es
// Declare the default precision:
// https://developer.mozilla.org/en-US/docs/Web/API/WebGL_API/WebGL_best_practices#implicit_defaults
precision highp float;

out vec4 vs_color;
out vec2 vs_uv;

uniform mat4 camera_transform;
uniform mat4 transform;
layout(location = 0) in vec4 obj_vertex;
layout(location = 1) in vec4 obj_normal;
layout(location = 2) in vec2 obj_uv;

void main(void) {
  gl_Position = camera_transform * transform * obj_vertex;

  vec3 obj_normal2 = normalize(vec3(transform * obj_normal));
  float shade_1 = clamp(dot(obj_normal2, vec3(-0.3, 0.3, -0.3)), 0.0, 1.0);
  float shade_2 = clamp(dot(obj_normal2, vec3(0.0, 1.0, 0.0)), 0.0, 1.0);
  float shade_3 = clamp(dot(obj_normal2, vec3(0.0, -1.0, 0.0)), 0.0, 1.0);
  vs_color =
      vec4(vec3(0.7, 0.6, 0.5) * shade_1 + vec3(0.4, 0.4, 0.4) * shade_2 +
               vec3(0.0, 0.0, 0.05) * shade_3 + vec3(0.05, 0.05, 0.05),
           1.0);
  vs_uv = obj_uv;
}
