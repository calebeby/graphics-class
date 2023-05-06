#version 300 es
// Declare the default precision:
// https://developer.mozilla.org/en-US/docs/Web/API/WebGL_API/WebGL_best_practices#implicit_defaults
precision highp float;

// Based on work by Kent Jones

in vec4 c;

uniform vec2 juliaC;
int mode = 0;

out vec4 color;

int mandel_potential() {
  vec2 z = c.xy; // x,y starting point in complex plane
  float x2, y2;  // next x,y point in recurrence
  int n = 0;     // count of iterations

  for (n = 0; n < 400; n++) {
    x2 = z.x * z.x;
    y2 = z.y * z.y;

    // compute next complex vector z
    if (mode == 0) { // Mandelbrot
      z = vec2(x2 - y2, 2.0 * z.x * z.y) + c.xy;
    } else { // Julia
      // Experiment: mixing julia with mandelbrot (looks cool)
      /* float m = 0.9; */
      /* z = vec2(x2 - y2, 2.0 * z.x * z.y) + */
      /*     vec2(mix(c.x, juliaC.x, m), mix(c.y, juliaC.y, m)); */
      // Well this is cool
      /* z = vec2(x2 - y2, cos(sin(2.0 * z.x * z.y))) + juliaC; */
      // Well this is cool
      /* z = vec2(x2 - y2, sin(2.0 * z.x * z.y)) + juliaC; */
      // Well this is cool
      /* z = vec2(x2 - y2, cos(2.0 * z.x * z.y)) + juliaC; */
      // Original
      z = vec2(x2 - y2, 2.0 * z.x * z.y) + juliaC;
    }

    if (x2 + y2 > 4.0)
      /* return n > 2 ? 80 : 0; */
      return n; // outside the mandelbrot set
  }
  return 0; // In the mandelbrot set
}

void main(void) {
  int n = mandel_potential();

  color = vec4((-cos(0.025 * float(n)) + 1.0) / 2.0,
               (-cos(0.08 * float(n)) + 1.0) / 2.0,
               (-cos(0.12 * float(n)) + 1.0) / 2.0, 1.0);
}
