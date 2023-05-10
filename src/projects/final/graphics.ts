import vs_source from "./vertex-shader.glsl?raw";
import fs_source from "./fragment-shader.glsl?raw";

import skybox_vs_source from "./skybox-vertex-shader.glsl?raw";
import skybox_fs_source from "./skybox-fragment-shader.glsl?raw";
import * as rust from "./pkg";

import type { GameState } from "./app";
import skybox_right_texture from "./assets/skybox-right.png";
import skybox_left_texture from "./assets/skybox-left.png";
import skybox_up_texture from "./assets/skybox-up.png";
import skybox_down_texture from "./assets/skybox-down.png";
import skybox_back_texture from "./assets/skybox-back.png";
import skybox_front_texture from "./assets/skybox-front.png";
import metal_texture from "./assets/metal-texture.jpg";

const load_image = (src: string) => {
  const img = new Image();

  return new Promise<HTMLImageElement>((resolve) => {
    img.onload = () => {
      resolve(img);
    };
    img.src = src;
  });
};

export const init_canvas = async (
  canvas: HTMLCanvasElement,
  game_state: GameState,
) => {
  const gl = canvas.getContext("webgl2")!;
  const rendering_program = init_shader_program(gl, vs_source, fs_source);
  const skybox_rendering_program = init_shader_program(
    gl,
    skybox_vs_source,
    skybox_fs_source,
  );
  const vertex_array_object = gl.createVertexArray();
  gl.bindVertexArray(vertex_array_object);
  const matrix_id_camera_transform = gl.getUniformLocation(
    rendering_program,
    "camera_transform",
  );
  const id_camera_position = gl.getUniformLocation(
    rendering_program,
    "camera_position",
  );
  const matrix_id_skybox_transform = gl.getUniformLocation(
    skybox_rendering_program,
    "transform",
  );

  // Sky box code based on https://github.com/lesnitsky/webgl-month/blob/dev/src/skybox.js
  Promise.all([
    load_image(skybox_right_texture),
    load_image(skybox_left_texture),
    load_image(skybox_up_texture),
    load_image(skybox_down_texture),
    load_image(skybox_back_texture),
    load_image(skybox_front_texture),
  ]).then((images) => {
    const texture = gl.createTexture();
    gl.bindTexture(gl.TEXTURE_CUBE_MAP, texture);

    gl.texParameteri(gl.TEXTURE_CUBE_MAP, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
    gl.texParameteri(gl.TEXTURE_CUBE_MAP, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
    gl.texParameteri(gl.TEXTURE_CUBE_MAP, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_CUBE_MAP, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);

    images.forEach((image, index) => {
      gl.texImage2D(
        gl.TEXTURE_CUBE_MAP_POSITIVE_X + index,
        0,
        gl.RGBA,
        gl.RGBA,
        gl.UNSIGNED_BYTE,
        image,
      );
    });
  });

  load_image(metal_texture).then((image) => {
    const texture = gl.createTexture();
    gl.bindTexture(gl.TEXTURE_2D, texture);
    gl.texImage2D(
      gl.TEXTURE_2D, //What kind of texture are we loading in
      0, // Level of detail, 0 base level
      gl.RGBA, // Internal (target) format of data, in this case Red, Gree, Blue, Alpha
      image.width, // Width of texture data (max is 1024, but maybe more)
      image.height, // Height of texture data
      0, //border (must be zero)
      gl.RGBA, //Format of input data (in this case we added the alpha when reading in data)
      gl.UNSIGNED_BYTE, //Type of data being passed in
      image,
    );
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.NEAREST);
  });

  // Hardcoded to match the layout locations declared in the vertex shader
  const attrib_id_obj_vertex = 0;
  const attrib_id_obj_colors = 1;

  game_state.skybox_vert_buffer = gl.createBuffer();
  game_state.obj_vert_buffer = gl.createBuffer();
  game_state.obj_colors_buffer = gl.createBuffer();
  gl.bindBuffer(gl.ARRAY_BUFFER, game_state.skybox_vert_buffer);
  const skybox_points = rust.generate_skybox_points();
  gl.bufferData(gl.ARRAY_BUFFER, skybox_points, gl.STATIC_DRAW);

  // Depth Test Enable (only render things 'forward' of other things)
  gl.enable(gl.DEPTH_TEST);
  // Passes if the fragment's depth values is less than stored value
  gl.depthFunc(gl.LEQUAL);

  // Track changes to the canvas element size and update the canvas internal rendering size
  const resize_listener: ResizeObserverCallback = () => {
    canvas.width = canvas.clientWidth * window.devicePixelRatio;
    canvas.height = canvas.clientHeight * window.devicePixelRatio;
    gl.viewport(0, 0, canvas.width, canvas.height);
    game_state.rust_state.aspect_ratio = canvas.width / canvas.height;
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
  const mousemove_listener = (event: MouseEvent) => {
    game_state.input_state.cursor_movement_x = event.movementX;
    game_state.input_state.cursor_movement_y = event.movementY;
  };
  const key_down_listener = key_listener(true);
  const key_up_listener = key_listener(false);

  const fullscreen = () => {
    canvas.requestFullscreen().then(() => {
      canvas.requestPointerLock();
    });
  };
  const keypress_listener = (event: KeyboardEvent) => {
    if (event.key === "f") {
      fullscreen();
    }
  };
  const click_listener = () => {
    game_state.is_active = true;
    canvas.requestPointerLock();
  };
  const dblclick_listener = () => {
    fullscreen();
  };

  let last_render_time = new Date().getTime();

  const render = () => {
    cancelAnimationFrame(frame_req);
    const is_active = document.pointerLockElement === canvas;
    game_state.is_active = is_active;
    if (is_active) {
      canvas.classList.add("active");
    } else {
      canvas.classList.remove("active");
    }
    const now = new Date().getTime();
    game_state.rust_state.update(
      is_active,
      game_state.input_state.input_w,
      game_state.input_state.input_a,
      game_state.input_state.input_s,
      game_state.input_state.input_d,
      game_state.input_state.cursor_movement_x,
      game_state.input_state.cursor_movement_y,
      now - last_render_time,
    );
    // Reset the cursor movement so if the mouse mousemove handler doesn't fire before the next render,
    // the previous movement values aren't reused.
    game_state.input_state.cursor_movement_x = 0;
    game_state.input_state.cursor_movement_y = 0;
    last_render_time = now;

    gl.clearBufferfv(gl.COLOR, 0, [0, 0, 0, 1]);

    // Draw skybox
    gl.useProgram(skybox_rendering_program);

    gl.bindBuffer(gl.ARRAY_BUFFER, game_state.skybox_vert_buffer!);
    gl.enableVertexAttribArray(attrib_id_obj_vertex);
    gl.vertexAttribPointer(
      attrib_id_obj_vertex, // Attribute in question
      4, // Number of elements (vec4)
      gl.FLOAT, // Type of element
      false, // Normalize? Nope
      0, // No stride (steps between indexes)
      0, // initial offset
    );
    gl.uniformMatrix4fv(
      matrix_id_skybox_transform,
      false,
      game_state.rust_state
        .world_to_camera_without_camera_translation()
        .to_f64_array(),
      0,
      16,
    );
    gl.drawArrays(gl.TRIANGLES, 0, skybox_points.length / 4);
    // Reset Attribute Array
    gl.disableVertexAttribArray(attrib_id_obj_vertex);

    // Draw game objects
    gl.useProgram(rendering_program);

    if (game_state.obj_vert_buffer) {
      gl.bindBuffer(gl.ARRAY_BUFFER, game_state.obj_vert_buffer);
      gl.enableVertexAttribArray(attrib_id_obj_vertex);
      gl.vertexAttribPointer(
        attrib_id_obj_vertex, // Attribute in question
        4, // Number of elements (vec4)
        gl.FLOAT, // Type of element
        false, // Normalize? Nope
        0, // No stride (steps between indexes)
        0, // initial offset
      );
    }
    if (game_state.obj_colors_buffer) {
      gl.bindBuffer(gl.ARRAY_BUFFER, game_state.obj_colors_buffer);
      gl.enableVertexAttribArray(attrib_id_obj_colors);
      gl.vertexAttribPointer(
        attrib_id_obj_colors, // Attribute in question
        4, // Number of elements (vec4)
        gl.FLOAT, // Type of element
        false, // Normalize? Nope
        0, // No stride (steps between indexes)
        0, // initial offset
      );
    }
    const camera_transform_matrix = game_state.rust_state.world_to_camera();
    gl.uniformMatrix4fv(
      matrix_id_camera_transform,
      false,
      camera_transform_matrix.to_f64_array(),
      0,
      16,
    );
    gl.uniform3fv(
      id_camera_position,
      Float32Array.from(game_state.rust_state.camera_position()),
    );
    gl.drawArrays(gl.TRIANGLES, 0, game_state.obj_num_points);
    // Reset Attribute Array
    gl.disableVertexAttribArray(attrib_id_obj_vertex);

    if (game_state.is_active) {
      frame_req = requestAnimationFrame(render);
    }
  };

  let frame_req = requestAnimationFrame(render);

  const set_points = (mesh: rust.ColoredMesh) => {
    const points = mesh.points();
    const colors = mesh.colors();
    gl.bindBuffer(gl.ARRAY_BUFFER, game_state.obj_vert_buffer!);
    gl.bufferData(gl.ARRAY_BUFFER, points, gl.STATIC_DRAW);
    gl.bindBuffer(gl.ARRAY_BUFFER, game_state.obj_colors_buffer!);
    gl.bufferData(gl.ARRAY_BUFFER, colors, gl.STATIC_DRAW);
    game_state.obj_num_points = points.length / 4;
  };

  window.addEventListener("keydown", key_down_listener);
  window.addEventListener("keyup", key_up_listener);
  window.addEventListener("keypress", keypress_listener);
  canvas.addEventListener("mousemove", mousemove_listener);
  canvas.addEventListener("click", click_listener);
  canvas.addEventListener("dblclick", dblclick_listener);
  document.addEventListener("pointerlockchange", render);

  return {
    render,
    set_mesh: set_points,
    cleanup() {
      // Remove all listeners, clean up webgl memory, etc.
      cancelAnimationFrame(frame_req);
      gl.deleteVertexArray(vertex_array_object);
      gl.deleteProgram(rendering_program);
      resize_observer.unobserve(canvas);
      window.removeEventListener("keydown", key_down_listener);
      window.removeEventListener("keyup", key_up_listener);
      window.removeEventListener("keypress", keypress_listener);
      canvas.removeEventListener("mousemove", mousemove_listener);
      canvas.removeEventListener("click", click_listener);
      canvas.removeEventListener("dblclick", dblclick_listener);
      document.removeEventListener("pointerlockchange", render);
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
