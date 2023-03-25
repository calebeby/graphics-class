import vsSource from "./vertex-shader.glsl?raw";
import fsSource from "./fragment-shader.glsl?raw";
import type { Face } from "./load-obj";
import type { GameState } from "./pkg/proj_3";

export const init_canvas = (
  canvas: HTMLCanvasElement,
  game_state: GameState,
  faces: Face[],
) => {
  const num_vertices = faces.reduce((count, face) => count + face.length, 0);
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
  const obj_vert_buffer = gl.createBuffer();
  gl.bindBuffer(gl.ARRAY_BUFFER, obj_vert_buffer);
  gl.bufferData(
    gl.ARRAY_BUFFER,
    new Float32Array(faces.flat().flat()),
    gl.STATIC_DRAW,
  ); // We only need to set this once (right now)

  gl.enableVertexAttribArray(attrib_id_obj_vertex);
  gl.bindBuffer(gl.ARRAY_BUFFER, obj_vert_buffer);
  gl.vertexAttribPointer(
    attrib_id_obj_vertex, // Attribute in question
    4, // Number of elements (vec4)
    gl.FLOAT, // Type of element
    false, // Normalize? Nope
    0, // No stride (steps between indexes)
    0, // initial offset
  );

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

  const render = () => {
    gl.useProgram(rendering_program);

    gl.enableVertexAttribArray(attrib_id_obj_vertex);
    gl.bindBuffer(gl.ARRAY_BUFFER, obj_vert_buffer);
    gl.vertexAttribPointer(
      attrib_id_obj_vertex, // Attribute in question
      4, // Number of elements (vec4)
      gl.FLOAT, // Type of element
      false, // Normalize? Nope
      0, // No stride (steps between indexes)
      0, // initial offset
    );
    const transform_matrix = game_state.get_transform_matrix();
    gl.uniformMatrix4fv(matrix_id_transform, false, transform_matrix, 0, 16);
    gl.clearBufferfv(gl.COLOR, 0, [0, 0.5, 1, 1]);
    gl.drawArrays(gl.TRIANGLES, 0, num_vertices);
    // Reset Attribute Array
    gl.disableVertexAttribArray(attrib_id_obj_vertex);
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