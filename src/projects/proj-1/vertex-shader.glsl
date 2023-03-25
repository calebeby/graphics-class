#version 300 es
// Declare the default precision:
// https://developer.mozilla.org/en-US/docs/Web/API/WebGL_API/WebGL_best_practices#implicit_defaults
precision highp float;

// Offset is passed in based on the cursor position
layout(location = 0) in vec4 offset;

void main(void) {
  // each vec4 is (x, y, z, w)
  // -> w is for perspective; see "Clipping" section in SB ch3
  const vec4 vertices[3] =
      vec4[3](vec4(0.0, 0, 0.0, 1.0), vec4(0.5, 0, 0.0, 1.0),
              vec4(0.0, 0.25, 0.0, 1.0));
  gl_Position = vertices[gl_VertexID] + offset;
}
