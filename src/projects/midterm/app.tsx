import { useEffect, useErrorBoundary, useRef, useState } from "preact/hooks";
import "./app.css";
import { init_canvas } from "./graphics";
import * as rust from "./pkg";
// This improves HMR for changes to rust file for some reason
import "./pkg/midterm_bg.wasm?url";

export interface GameObject {
  transform_matrix: rust.TransformMatrix;
  vertex_coords: Float32Array;
  vertex_normals: Float32Array;
  vertex_uvs: Float32Array;
  obj_vert_buffer?: WebGLBuffer | null;
  obj_normals_buffer?: WebGLBuffer | null;
  obj_uvs_buffer?: WebGLBuffer | null;
}

export interface GameState {
  rust_state: rust.GameState;
  objects: GameObject[];
  is_active: boolean;
  input_state: {
    cursor_movement_x: number;
    cursor_movement_y: number;
    input_w: boolean;
    input_a: boolean;
    input_s: boolean;
    input_d: boolean;
  };
  skybox_vert_buffer?: WebGLBuffer | null;
}

interface Props {}

export const Midterm = ({}: Props) => {
  // eslint-disable-next-line @typescript-eslint/naming-convention
  const [error, _reset_error] = useErrorBoundary();
  const canvas_ref = useRef<HTMLCanvasElement>(null);
  const [game_state, set_game_state] = useState<GameState | undefined>(
    undefined,
  );

  useEffect(() => {
    rust.default().then(() => {
      const rust_state = new rust.GameState();
      set_game_state({
        rust_state,
        objects: [
          {
            transform_matrix: new rust.TransformMatrix(0, 0, 0),
            vertex_coords: rust_state.points_to_float32array(),
            vertex_normals: rust_state.normals_to_float32array(),
            vertex_uvs: rust_state.uvs_to_float32array(),
          },
        ],
        is_active: false,
        input_state: {
          cursor_movement_x: 0.0,
          cursor_movement_y: 0.0,
          input_w: false,
          input_a: false,
          input_s: false,
          input_d: false,
        },
      });
    });
  }, [rust.default]);

  useEffect(() => {
    const canvas = canvas_ref.current!;

    if (!game_state) return;
    const { cleanup } = init_canvas(canvas, game_state);
    return () => cleanup();
  }, [game_state]);

  return (
    <div class="transform-demo">
      {error}
      <canvas ref={canvas_ref}></canvas>
    </div>
  );
};
