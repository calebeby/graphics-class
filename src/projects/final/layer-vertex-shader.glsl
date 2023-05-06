#version 300 es
// Declare the default precision:
// https://developer.mozilla.org/en-US/docs/Web/API/WebGL_API/WebGL_best_practices#implicit_defaults
precision highp float;

// Based on work by Kent Jones

// Output to fragment shader
out vec4 c;

// Inputs
in vec4 v_position;        // Vertex Position
uniform float zoomFactor;  // Scaling of the output
uniform vec3 centerOfView; // Where we scale in the image

void main(void) {
  mat4 translate =
      /* mat4(1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, */
      /*      -centerOfView.x, -centerOfView.y, centerOfView.z, zoomFactor); */
      mat4(zoomFactor, 0.0, 0.0, 0.0, 0.0, zoomFactor, 0.0, 0.0, 0.0, 0.0,
           zoomFactor, 0.0, -zoomFactor * centerOfView.x,
           -zoomFactor * centerOfView.y, centerOfView.z, 1.0);

  // Vertex processing
  // Set vertex points (remember ouput will be interpolated)
  gl_Position = translate * v_position;
  c = v_position; // Set up c bounds
}
