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
  random_seed: number;
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

const get_random_seed = () => Math.floor(Math.random() * 100000);
const get_seed_from_url = () => {
  const seed_from_url = new URL(window.location.href).searchParams.get("seed");
  return seed_from_url ? Number.parseInt(seed_from_url) : undefined;
};

export const Midterm = ({}: Props) => {
  // eslint-disable-next-line @typescript-eslint/naming-convention
  const [error, _reset_error] = useErrorBoundary();
  const canvas_ref = useRef<HTMLCanvasElement>(null);
  const [random_seed, set_random_seed] = useState<number>(
    () => get_seed_from_url() || get_random_seed(),
  );
  const [rust_module, set_rust_module] = useState<rust.InitOutput | null>(null);
  const render_ref = useRef<() => void>();

  const capture_screenshot = () => {
    const link = document.createElement("a");
    link.download = "maze-screenshot.png";
    render_ref.current?.();
    link.href = canvas_ref.current!.toDataURL();
    link.click();
  };

  useEffect(() => {
    const m_listener = (e: KeyboardEvent) => {
      if (e.key === "m") capture_screenshot();
    };
    window.addEventListener("keypress", m_listener);
    return () => window.removeEventListener("keypress", m_listener);
  }, []);

  useEffect(() => {
    const popstate_listener = () => {
      let seed = get_seed_from_url();
      if (seed !== undefined) set_random_seed(seed);
    };
    window.addEventListener("popstate", popstate_listener);
    return () => window.removeEventListener("popstate", popstate_listener);
  }, []);

  useEffect(() => {
    rust.default().then((mod) => set_rust_module(mod));
  }, []);

  useEffect(() => {
    if (!rust_module) return;
    const canvas = canvas_ref.current!;
    const rust_state = new rust.GameState(random_seed);
    const game_state: GameState = {
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
      random_seed,
    };
    const { cleanup, render } = init_canvas(canvas, game_state);
    render_ref.current = render;
    return () => {
      cleanup();
      game_state?.rust_state.free();
    };
  }, [rust_module, random_seed]);

  useEffect(() => {
    const url = new URL(window.location.href);
    if (url.searchParams.get("seed") !== String(random_seed)) {
      url.searchParams.set("seed", String(random_seed));
      window.history.pushState({}, "", url);
    }
  }, [random_seed]);

  return (
    <div class="demo">
      {error}
      <canvas ref={canvas_ref}></canvas>
      <label>
        <div>Random seed</div>
        <input
          type="text"
          pattern="\d*"
          value={random_seed}
          onChange={(e) => {
            const seed = Number.parseInt(e.currentTarget.value);
            if (!Number.isNaN(seed)) {
              set_random_seed(seed);
            }
          }}
        />
      </label>
      <button
        onClick={() => {
          set_random_seed(get_random_seed());
        }}
      >
        Randomize Maze
      </button>
      <button onClick={capture_screenshot}>Download screenshot</button>
    </div>
  );
};
