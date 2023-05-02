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
  light_position: [x: number, y: number, z: number];
}

interface Props {}

export const Proj4 = ({}: Props) => {
  // eslint-disable-next-line @typescript-eslint/naming-convention
  const [error, _reset_error] = useErrorBoundary();
  const canvas_ref = useRef<HTMLCanvasElement>(null);
  const [rust_module, set_rust_module] = useState<rust.InitOutput | null>(null);
  const state_ref = useRef<GameState | null>(null);
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
      light_position: [10.0, 10.0, 0.0],
    };
    state_ref.current = game_state;
    let canvas_cleanup = () => {};
    init_canvas(canvas, game_state).then(({ cleanup, render }) => {
      canvas_cleanup = cleanup;
      render_ref.current = render;
    });
    return () => {
      canvas_cleanup();
      try {
        game_state?.rust_state.free();
      } catch {}
    };
  }, [rust_module]);

  return (
    <div class="demo">
      {error}
      <canvas ref={canvas_ref}></canvas>
      <CoordinateInput
        name="Target Position"
        min={-3}
        max={3}
        on_change={(x, y, z) => {
          const state = state_ref.current;
          if (!state) return;
          state.rust_state.update_target(x, y, z);
          render_ref.current?.();
        }}
      />
      <CoordinateInput
        name="Light Source Position"
        min={-20}
        max={20}
        on_change={(x, y, z) => {
          const state = state_ref.current;
          if (!state) return;
          state.light_position = [x, y, z];
          state.rust_state.update_light_position(x, y, z);
          render_ref.current?.();
        }}
      />
      <button onClick={capture_screenshot}>Download screenshot</button>
    </div>
  );
};

const CoordinateInput = ({
  on_change,
  name,
  min,
  max,
}: {
  on_change: (x: number, y: number, z: number) => void;
  name: string;
  min: number;
  max: number;
}) => {
  const [x, set_x] = useState(0.5);
  const [y, set_y] = useState(0.5);
  const [z, set_z] = useState(0.5);
  const step_text = (max - min) / 100;
  const step_range = (max - min) / 1000;
  useEffect(() => {
    on_change(x, y, z);
  }, [x, y, z]);
  return (
    <>
      <h2>{name}</h2>
      <div>
        (
        <input
          type="number"
          min={min}
          max={max}
          step={step_text}
          value={x}
          onInput={(e) => {
            set_x(e.currentTarget.valueAsNumber);
          }}
        />
        ,
        <input
          type="number"
          min={min}
          max={max}
          step={step_text}
          value={y}
          onInput={(e) => {
            set_y(e.currentTarget.valueAsNumber);
          }}
        />
        ,
        <input
          type="number"
          min={min}
          max={max}
          step={step_text}
          value={z}
          onInput={(e) => {
            set_z(e.currentTarget.valueAsNumber);
          }}
        />
        )
      </div>

      <div class="coordinate-input-ranges">
        <input
          type="range"
          min={min}
          max={max}
          step={step_range}
          value={x}
          onInput={(e) => {
            set_x(e.currentTarget.valueAsNumber);
          }}
        />
        <input
          type="range"
          min={min}
          max={max}
          step={step_range}
          value={y}
          onInput={(e) => {
            set_y(e.currentTarget.valueAsNumber);
          }}
        />
        <input
          type="range"
          min={min}
          max={max}
          step={step_range}
          value={z}
          onInput={(e) => {
            set_z(e.currentTarget.valueAsNumber);
          }}
        />
      </div>
    </>
  );
};
