import { useEffect, useErrorBoundary, useRef, useState } from "preact/hooks";
import "./app.css";
import { init_canvas } from "./graphics";
import obj_kermit from "./kermit.obj?raw";
import obj_cat from "./cat.obj?raw";
import obj_ico from "./ico.obj?raw";
import { load_obj } from "./load-obj";
import * as rust from "./pkg";
// This improves HMR for changes to rust file for some reason
import "./pkg/proj_3_bg.wasm?url";

export interface GameObject {
  transform_matrix: rust.TransformMatrix;
  vertex_coords: Float32Array;
  obj_vert_buffer?: WebGLBuffer | null;
}

export interface GameState {
  rust_state: rust.GameState;
  objects: GameObject[];
  input_state: {
    cursor_x: number;
    cursor_y: number;
    input_w: boolean;
    input_a: boolean;
    input_s: boolean;
    input_d: boolean;
  };
}

interface Props {}

export const Proj3 = ({}: Props) => {
  // eslint-disable-next-line @typescript-eslint/naming-convention
  const [error, _reset_error] = useErrorBoundary();
  const canvas_ref = useRef<HTMLCanvasElement>(null);
  const [game_state, set_game_state] = useState<GameState | undefined>(
    undefined,
  );

  useEffect(() => {
    rust.default().then(() => {
      set_game_state({
        rust_state: new rust.GameState(),
        objects: [
          {
            transform_matrix: new rust.TransformMatrix(0, 0, 0),
            vertex_coords: new Float32Array(load_obj(obj_kermit).flat().flat()),
          },
          {
            transform_matrix: new rust.TransformMatrix(2.5, 0, 0.0),
            vertex_coords: new Float32Array(load_obj(obj_cat).flat().flat()),
          },
          {
            transform_matrix: new rust.TransformMatrix(-2.5, 0, 0.0),
            vertex_coords: new Float32Array(load_obj(obj_ico).flat().flat()),
          },
        ],
        input_state: {
          cursor_x: 0.0,
          cursor_y: 0.0,
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
