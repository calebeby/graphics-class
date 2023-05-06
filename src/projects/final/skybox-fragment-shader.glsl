#version 300 es
// Declare the default precision:
// https://developer.mozilla.org/en-US/docs/Web/API/WebGL_API/WebGL_best_practices#implicit_defaults

precision highp float;

in vec4 texture_coordinates;
uniform samplerCube skybox;
out vec4 color;

void main() { color = texture(skybox, vec3(texture_coordinates)); }
