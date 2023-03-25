#version 300 es
// Declare the default precision:
// https://developer.mozilla.org/en-US/docs/Web/API/WebGL_API/WebGL_best_practices#implicit_defaults
precision highp float;

out vec4 vs_color;

uniform mat4 transform;

void main(void) {
  // Create six triangles
  // First four: all oriented around orgin, and 'pointing' towards camera
  // Last two: Bottom 'square, just filling out the bottom of the shape
  const vec4 vertices[] =
      vec4[](vec4(0.2, 0.2, 0.0, 1.0), // T1 Right
             vec4(0.2, -0.2, 0.0, 1.0), vec4(0.0, 0.0, -0.288, 1.0),
             vec4(-0.2, 0.2, 0.0, 1.0), // T2 Top
             vec4(0.2, 0.2, 0.0, 1.0), vec4(0.0, 0.0, -0.288, 1.0),
             vec4(-0.2, 0.2, 0.0, 1.0), // T3 Left
             vec4(-0.2, -0.2, 0.0, 1.0), vec4(0.0, 0.0, -0.288, 1.0),
             vec4(0.2, -0.2, 0.0, 1.0), // T4 Botom
             vec4(-0.2, -0.2, 0.0, 1.0), vec4(0.0, 0.0, -0.288, 1.0),
             vec4(0.2, 0.2, 0.0, 1.0), // T5 Under, tr
             vec4(0.2, -0.2, 0.0, 1.0), vec4(-0.2, 0.2, 0.0, 1.0),
             vec4(-0.2, 0.2, 0.0, 1.0), // T6 Under, bl
             vec4(0.2, -0.2, 0.0, 1.0), vec4(-0.2, -0.2, 0.0, 1.0));

  // First triangle is blue
  // Second is Green
  // Third red
  // fourth is yellow
  // bottom two are black
  // White is added to give indication of 'up'
  const vec4 colors[] = vec4[](vec4(0.0, 0.0, 1.0, 1.0), // Blue
                               vec4(0.0, 0.0, 1.0, 1.0), // Blue
                               vec4(1.0, 1.0, 1.0, 1.0), // White
                               vec4(0.0, 1.0, 0.0, 1.0), // Green
                               vec4(0.0, 1.0, 0.0, 1.0), // Green
                               vec4(1.0, 1.0, 1.0, 1.0), // White
                               vec4(1.0, 0.0, 0.0, 1.0), // Red
                               vec4(1.0, 0.0, 0.0, 1.0), // Red
                               vec4(1.0, 1.0, 1.0, 1.0), // White
                               vec4(1.0, 1.0, 0.0, 1.0), // Yellow
                               vec4(1.0, 1.0, 0.0, 1.0), // Yellow
                               vec4(1.0, 1.0, 1.0, 1.0), // White
                               vec4(0.0, 0.0, 0.0, 1.0), // Black
                               vec4(0.0, 0.0, 0.0, 1.0), // Black
                               vec4(0.0, 0.0, 0.0, 1.0), // Black
                               vec4(0.0, 0.0, 0.0, 1.0), // Black
                               vec4(0.0, 0.0, 0.0, 1.0), // Black
                               vec4(0.0, 0.0, 0.0, 1.0)  // Black
  );

  float d = 2.0; // Projection plane defintion
  // Perspective project matrix
  mat4 persp =
      mat4(1.0, 0.0, 0.0, 0.0, // x col
           0.0, 1.0, 0.0, 0.0, // y col
           0.0, 0.0, 1.0,
           1.0 / d, // z col This is kind of like shearing in the z/w axis
           0.0, 0.0, 0.0, 1.0); // w col

  float angle_x = 1.31; // ~75 degres
  // Rotational matrix (kind of a simple camera transform)
  mat4 rot_x = mat4(1.0, 0.0, 0.0, 0.0,                    // x col
                    0.0, cos(angle_x), sin(angle_x), 0.0,  // y col
                    0.0, -sin(angle_x), cos(angle_x), 0.0, // z col
                    0.0, 0.0, 0.0, 1.0);                   // w col

  // Resulting point, dependent on if we apply external transform or not
  vec4 base;

  // Define base depending on which vertex we are currently in
  /* if (gl_VertexID < 18) { */
  /*   // Original Triangles */
  /*   // We can think of this as object space */
  /*   // origin at world origin, colinear axes */
  /*   base = vertices[gl_VertexID % 18]; */
  /* } else { */
  // Transform Triangles
  // We can think of this as our translation into (or through) world
  // coordinates We are functionally 'placing' our object into the world
  base = transform * vertices[gl_VertexID % 18];
  /* } */

  //'overhead' view of scene, no camera, no perspective
  gl_Position = base;

  // Camera rotation and perspective
  // gl_Position = persp * rot_x  * base;

  vs_color = colors[gl_VertexID % 18];
}
