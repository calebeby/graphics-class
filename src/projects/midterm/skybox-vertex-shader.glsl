#version 300 es
// Declare the default precision:
// https://developer.mozilla.org/en-US/docs/Web/API/WebGL_API/WebGL_best_practices#implicit_defaults

precision highp float;
in vec4 cube_vertex;
out vec4 texture_coordinates;

uniform mat4 transform;

void main() {
  // Starting by passing texture coordinates (flipped texture mapping)
  texture_coordinates = cube_vertex;

  // All modifications are pulled in via attributes
  // The skybox had its up set to the -y direction so the negative here flips it
  gl_Position = -transform * cube_vertex;
}
