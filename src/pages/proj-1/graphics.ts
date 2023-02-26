import vsSource from "./vertex-shader.glsl?raw";
import fsSource from "./fragment-shader.glsl?raw";

export const init_canvas = (canvas: HTMLCanvasElement) => {
  const gl = canvas.getContext("webgl2")!;
  const rendering_program = init_shader_program(gl);
  const vertex_array_object = gl.createVertexArray();
  gl.bindVertexArray(vertex_array_object);

  // Track the mouse x/y position via mousemove events
  let mouse = { x: 0, y: 0 };
  const mouse_listener = (e: MouseEvent) => {
    mouse.x = e.offsetX;
    mouse.y = e.offsetY;
  };
  canvas.addEventListener("mousemove", mouse_listener);

  // Track changes to the canvas element size and update the canvas internal rendering size
  const resize_listener: ResizeObserverCallback = () => {
    canvas.width = canvas.clientWidth;
    canvas.height = canvas.clientHeight;
    cancelAnimationFrame(frame_req);
    render();
  };
  const resize_observer = new ResizeObserver(resize_listener);
  resize_observer.observe(canvas, {});

  const render = () => {
    gl.clearBufferfv(gl.COLOR, 0, [0, 0, 0, 1]);
    gl.vertexAttrib4fv(0, [mouse.x / 1000, -mouse.y / 1000, 0, 1]);
    gl.useProgram(rendering_program);
    gl.drawArrays(gl.TRIANGLES, 0, 3);
    frame_req = requestAnimationFrame(render);
  };

  let frame_req = requestAnimationFrame(render);

  return {
    cleanup() {
      // Remove all listeners, clean up webgl memory, etc.
      cancelAnimationFrame(frame_req);
      gl.deleteVertexArray(vertex_array_object);
      gl.deleteProgram(rendering_program);
      canvas.removeEventListener("mousemove", mouse_listener);
      resize_observer.unobserve(canvas);
    },
  };
};

const init_shader_program = (gl: WebGL2RenderingContext) => {
  // Based on https://developer.mozilla.org/en-US/docs/Web/API/WebGL_API/Tutorial/Adding_2D_content_to_a_WebGL_context#initializing_the_shaders
  const vertex_shader = load_shader(gl, gl.VERTEX_SHADER, vsSource)!;
  const fragment_shader = load_shader(gl, gl.FRAGMENT_SHADER, fsSource)!;

  const rendering_program = gl.createProgram()!;
  gl.attachShader(rendering_program, vertex_shader);
  gl.attachShader(rendering_program, fragment_shader);
  gl.deleteShader(vertex_shader);
  gl.deleteShader(fragment_shader);

  gl.linkProgram(rendering_program);

  if (!gl.getProgramParameter(rendering_program, gl.LINK_STATUS)) {
    throw new Error(
      `Unable to initialize the shader program: ${gl.getProgramInfoLog(
        rendering_program,
      )}`,
    );
  }

  return rendering_program;
};

// Based on https://developer.mozilla.org/en-US/docs/Web/API/WebGL_API/Tutorial/Getting_started_with_WebGL
const load_shader = (
  gl: WebGL2RenderingContext,
  type: GLenum,
  source: string,
) => {
  const shader = gl.createShader(type)!;
  gl.shaderSource(shader, source);
  gl.compileShader(shader);

  if (!gl.getShaderParameter(shader, gl.COMPILE_STATUS)) {
    throw new Error(
      `An error occurred compiling the shaders: ${gl.getShaderInfoLog(shader)}`,
    );
  }

  return shader;
};
