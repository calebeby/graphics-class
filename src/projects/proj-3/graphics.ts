import vsSource from "./vertex-shader.glsl?raw";
import fsSource from "./fragment-shader.glsl?raw";
import type { GameState } from "./app";
import * as rust from "./pkg";

export const init_canvas = (
  canvas: HTMLCanvasElement,
  game_state: GameState,
) => {
  const gl = canvas.getContext("webgl2")!;
  const rendering_program = init_shader_program(gl);
  const vertex_array_object = gl.createVertexArray();
  gl.bindVertexArray(vertex_array_object);
  const matrix_id_transform = gl.getUniformLocation(
    rendering_program,
    "transform",
  );

  // Hardcoded to match the layout locations declared in the vertex shader
  const attrib_id_obj_vertex = 0;

  for (const object of game_state.objects) {
    object.obj_vert_buffer = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, object.obj_vert_buffer);
    gl.bufferData(gl.ARRAY_BUFFER, object.vertex_coords, gl.STATIC_DRAW); // We only need to set this once (right now)

    gl.enableVertexAttribArray(attrib_id_obj_vertex);
    gl.bindBuffer(gl.ARRAY_BUFFER, object.obj_vert_buffer);
  }

  //Depth Test Enable (only render things 'forward' of other things)
  gl.enable(gl.DEPTH_TEST);
  // Passes if the fragment's depth values is less than stored value
  gl.depthFunc(gl.LEQUAL);

  // Track changes to the canvas element size and update the canvas internal rendering size
  const resize_listener: ResizeObserverCallback = () => {
    canvas.width = canvas.clientWidth * window.devicePixelRatio;
    canvas.height = canvas.clientHeight * window.devicePixelRatio;
    const min_dimension = Math.min(canvas.width, canvas.height);
    // Force the axes to be square (matching x and y scale in terms of pixels,
    // even if the canvas "rendering box" is rectangular)
    gl.viewport(
      // Force 0, 0 to be in the middle by shifting the axes based on which dimension is larger
      (canvas.width - min_dimension) / 2,
      (canvas.height - min_dimension) / 2,
      min_dimension,
      min_dimension,
    );
    cancelAnimationFrame(frame_req);
    render();
  };
  const resize_observer = new ResizeObserver(resize_listener);
  resize_observer.observe(canvas, {});

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
  const mouse_listener = (event: MouseEvent) => {
    // Normalize the mouse position into x and y coordinates between -1 and 1
    game_state.input_state.cursor_x =
      (2 * event.offsetX) / canvas.offsetWidth - 1;
    game_state.input_state.cursor_y =
      (2 * event.offsetY) / canvas.offsetHeight - 1;
  };
  const key_down_listener = key_listener(true);
  const key_up_listener = key_listener(false);
  window.addEventListener("keydown", key_down_listener);
  window.addEventListener("keyup", key_up_listener);
  canvas.addEventListener("mousemove", mouse_listener);

  let last_render_time = new Date().getTime();

  const render = () => {
    const now = new Date().getTime();
    // Animate the cat
    game_state.objects[1].transform_matrix = new rust.TransformMatrix(
      2.5,
      Math.sin(now / 1000),
      0.0,
    );
    game_state.rust_state.update(
      game_state.input_state.input_w,
      game_state.input_state.input_a,
      game_state.input_state.input_s,
      game_state.input_state.input_d,
      game_state.input_state.cursor_x,
      game_state.input_state.cursor_y,
      now - last_render_time,
    );
    last_render_time = now;
    gl.useProgram(rendering_program);
    gl.clearBufferfv(gl.COLOR, 0, [0, 0.5, 1, 1]);

    const world_to_camera = game_state.rust_state.world_to_camera();
    for (const object of game_state.objects) {
      if (!object.obj_vert_buffer) throw new Error("missing obj_vert_buffer");
      gl.bindBuffer(gl.ARRAY_BUFFER, object.obj_vert_buffer);
      gl.enableVertexAttribArray(attrib_id_obj_vertex);
      gl.vertexAttribPointer(
        attrib_id_obj_vertex, // Attribute in question
        4, // Number of elements (vec4)
        gl.FLOAT, // Type of element
        false, // Normalize? Nope
        0, // No stride (steps between indexes)
        0, // initial offset
      );
      const transform_matrix = world_to_camera.times(object.transform_matrix);
      gl.uniformMatrix4fv(
        matrix_id_transform,
        false,
        transform_matrix.to_f64_array(),
        0,
        16,
      );
      gl.drawArrays(gl.TRIANGLES, 0, object.vertex_coords.length / 4);
      // Reset Attribute Array
      gl.disableVertexAttribArray(attrib_id_obj_vertex);
    }
    frame_req = requestAnimationFrame(render);
  };

  let frame_req = requestAnimationFrame(render);

  return {
    cleanup() {
      // Remove all listeners, clean up webgl memory, etc.
      cancelAnimationFrame(frame_req);
      gl.deleteVertexArray(vertex_array_object);
      gl.deleteProgram(rendering_program);
      resize_observer.unobserve(canvas);
      window.removeEventListener("keydown", key_down_listener);
      window.removeEventListener("keyup", key_up_listener);
      canvas.removeEventListener("mousemove", mouse_listener);
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
