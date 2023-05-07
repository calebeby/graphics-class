import type { GameState, SnapshotParameters } from "./app";
import vs_source from "./layer-vertex-shader.glsl?raw";
import fs_source from "./layer-fragment-shader.glsl?raw";

export const init_layer_canvas = async (
  canvas: HTMLCanvasElement,
  game_state: GameState,
) => {
  const gl = canvas.getContext("webgl2", { antialias: true })!;
  const rendering_program = init_shader_program(gl, vs_source, fs_source);

  canvas.width = 1500;
  canvas.height = 1500;
  gl.viewport(0, 0, canvas.width, canvas.height);

  const vertex_array_object = gl.createVertexArray();
  gl.bindVertexArray(vertex_array_object);

  // Hardcoded to match the layout locations declared in the vertex shader
  const attrib_id_vertex = 0;

  // Depth Test Enable (only render things 'forward' of other things)
  gl.enable(gl.DEPTH_TEST);
  // Passes if the fragment's depth values is less than stored value
  gl.depthFunc(gl.LEQUAL);

  const vert_buffer = gl.createBuffer();
  const vertices = [
    [-5.0, 3.0, 0.0, 1.0],
    [2.0, 3.0, 0.0, 1.0],
    [2.0, -3.0, 0.0, 1.0],
    [-5.0, -3.0, 0.0, 1.0],
  ];
  const points = [
    vertices[0], // Triangle #1
    vertices[1], // Triangle #1
    vertices[2], // Triangle #1
    vertices[0], // Triangle #2
    vertices[2], // Triangle #2
    vertices[3], // Triangle #2
  ];
  gl.bindBuffer(gl.ARRAY_BUFFER, vert_buffer);
  gl.bufferData(
    gl.ARRAY_BUFFER,
    Float32Array.from(points.flat()),
    gl.STATIC_DRAW,
  );

  const key_listener = (is_down: boolean) => (event: KeyboardEvent) => {
    if (event.key === "w") {
      game_state.input_state.input_w = is_down;
    } else if (event.key === "a") {
      game_state.input_state.input_a = is_down;
    } else if (event.key === "s") {
      game_state.input_state.input_s = is_down;
    } else if (event.key === "d") {
      game_state.input_state.input_d = is_down;
    }
  };
  const mousemove_listener = (event: MouseEvent) => {
    game_state.input_state.cursor_movement_x = event.movementX;
    game_state.input_state.cursor_movement_y = event.movementY;
  };
  const key_down_listener = key_listener(true);
  const key_up_listener = key_listener(false);

  const fullscreen = () => {
    canvas.requestFullscreen();
  };
  const click_listener = () => {};
  const dblclick_listener = () => {
    fullscreen();
  };

  const id_julia_c = gl.getUniformLocation(rendering_program, "juliaC");
  const id_center_of_view = gl.getUniformLocation(
    rendering_program,
    "centerOfView",
  );
  const id_zoom_factor = gl.getUniformLocation(rendering_program, "zoomFactor");

  const render = (snapshot_parameters: SnapshotParameters) => {
    const num_vertices = 6;

    gl.clearBufferfv(gl.COLOR, 0, [0, 0, 0, 1]);

    // Draw game objects
    gl.useProgram(rendering_program);

    gl.uniform2fv(
      id_julia_c,
      Float32Array.from([
        snapshot_parameters.julia_c.x,
        snapshot_parameters.julia_c.y,
      ]),
    );
    gl.uniform1f(id_zoom_factor, snapshot_parameters.zoom_factor);
    gl.uniform3fv(id_center_of_view, snapshot_parameters.center_of_view);

    gl.bindBuffer(gl.ARRAY_BUFFER, vert_buffer);
    gl.enableVertexAttribArray(attrib_id_vertex);
    gl.vertexAttribPointer(
      attrib_id_vertex, // Attribute in question
      4, // Number of elements (vec4)
      gl.FLOAT, // Type of element
      false, // Normalize? Nope
      0, // No stride (steps between indexes)
      0, // initial offset
    );
    gl.drawArrays(gl.TRIANGLES, 0, num_vertices);
    // Reset Attribute Array
    gl.disableVertexAttribArray(attrib_id_vertex);
    let pixel_buf = new Uint8Array(canvas.width * canvas.height * 4);
    gl.readPixels(0, 0, 150, 150, gl.RGBA, gl.UNSIGNED_BYTE, pixel_buf);
  };

  window.addEventListener("keydown", key_down_listener);
  window.addEventListener("keyup", key_up_listener);
  canvas.addEventListener("mousemove", mousemove_listener);
  canvas.addEventListener("click", click_listener);
  canvas.addEventListener("dblclick", dblclick_listener);

  return {
    render,
    cleanup() {
      // Remove all listeners, clean up webgl memory, etc.
      gl.deleteVertexArray(vertex_array_object);
      gl.deleteProgram(rendering_program);
      window.removeEventListener("keydown", key_down_listener);
      window.removeEventListener("keyup", key_up_listener);
      canvas.removeEventListener("mousemove", mousemove_listener);
      canvas.removeEventListener("click", click_listener);
      canvas.removeEventListener("dblclick", dblclick_listener);
    },
  };
};

const init_shader_program = (
  gl: WebGL2RenderingContext,
  vs_source: string,
  fs_source: string,
) => {
  // Based on https://developer.mozilla.org/en-US/docs/Web/API/WebGL_API/Tutorial/Adding_2D_content_to_a_WebGL_context#initializing_the_shaders
  const vertex_shader = load_shader(gl, gl.VERTEX_SHADER, vs_source)!;
  const fragment_shader = load_shader(gl, gl.FRAGMENT_SHADER, fs_source)!;

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
