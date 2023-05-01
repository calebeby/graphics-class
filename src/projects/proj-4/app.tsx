import { useEffect, useErrorBoundary, useRef, useState } from "preact/hooks";
import "./app.css";
import { init_canvas } from "./graphics";
import * as rust from "./pkg";
// This improves HMR for changes to rust file for some reason
import "./pkg/proj_4_bg.wasm?url";

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

export const Proj4 = ({}: Props) => {
  // eslint-disable-next-line @typescript-eslint/naming-convention
  const [error, _reset_error] = useErrorBoundary();
  const canvas_ref = useRef<HTMLCanvasElement>(null);
  const [rust_module, set_rust_module] = useState<rust.InitOutput | null>(null);
  const rust_state_ref = useRef<rust.GameState | null>(null);
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
    rust.default().then((mod) => set_rust_module(mod));
  }, []);

  useEffect(() => {
    if (!rust_module) return;
    const canvas = canvas_ref.current!;
    const rust_state = new rust.GameState();
    const game_state: GameState = {
      rust_state,
      is_active: false,
      input_state: {
        cursor_movement_x: 0.0,
        cursor_movement_y: 0.0,
        input_w: false,
        input_a: false,
        input_s: false,
        input_d: false,
      },
    };
    rust_state_ref.current = rust_state;
    let canvas_cleanup = () => {};
    init_canvas(canvas, game_state).then(({ cleanup, render }) => {
      canvas_cleanup = cleanup;
      render_ref.current = render;
    });
    return () => {
      canvas_cleanup();
      game_state?.rust_state.free();
    };
  }, [rust_module]);

  const range = 3;
  const step = range / 1000;

  return (
    <div class="demo">
      {error}
      <canvas ref={canvas_ref}></canvas>
      <label>
        Target X
        <input
          type="range"
          min={-range}
          max={range}
          step={step}
          onInput={(e) => {
            const state = rust_state_ref.current;
            if (!state) return;
            state.update_target_x(e.currentTarget.valueAsNumber);
            render_ref.current?.();
          }}
        />
      </label>
      <label>
        Target Y
        <input
          type="range"
          min={-range}
          max={range}
          step={step}
          onInput={(e) => {
            const state = rust_state_ref.current;
            if (!state) return;
            state.update_target_y(-e.currentTarget.valueAsNumber);
            render_ref.current?.();
          }}
        />
      </label>
      <label>
        Target Z
        <input
          type="range"
          min={-range}
          max={range}
          step={step}
          onInput={(e) => {
            const state = rust_state_ref.current;
            if (!state) return;
            state.update_target_z(e.currentTarget.valueAsNumber);
            render_ref.current?.();
          }}
        />
      </label>
      <button onClick={capture_screenshot}>Download screenshot</button>
    </div>
  );
};
