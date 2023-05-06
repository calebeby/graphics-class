import { useEffect, useErrorBoundary, useRef, useState } from "preact/hooks";
import type { JSX } from "preact/jsx-runtime";
import "./app.css";
import { init_canvas } from "./graphics";
import { init_layer_canvas } from "./layer-render";
import * as rust from "./pkg";
// This improves HMR for changes to rust file for some reason
import "./pkg/final_bg.wasm?url";

export interface SnapshotParameters {
  julia_c: {
    x: number;
    y: number;
  };
  zoom_factor: number;
  center_of_view: [x: number, y: number, z: number];
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

export const Final = ({}: Props) => {
  // eslint-disable-next-line @typescript-eslint/naming-convention
  const [error, _reset_error] = useErrorBoundary();
  const canvas_ref = useRef<HTMLCanvasElement>(null);
  const layer_canvas_ref = useRef<HTMLCanvasElement>(null);
  const [rust_module, set_rust_module] = useState<rust.InitOutput | null>(null);
  const state_ref = useRef<GameState | null>(null);
  const render_ref = useRef<() => void>();
  const render_layer_ref =
    useRef<(snapshot_parameters: SnapshotParameters) => void>();

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
    const layer_canvas = layer_canvas_ref.current!;
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
    state_ref.current = game_state;
    let canvas_cleanup = () => {};
    init_canvas(canvas, game_state).then(({ cleanup, render }) => {
      canvas_cleanup = cleanup;
      render_ref.current = render;
    });
    init_layer_canvas(layer_canvas, game_state).then(({ cleanup, render }) => {
      canvas_cleanup = cleanup;
      render(snapshot_parameters.current);
      render_layer_ref.current = render;
    });
    return () => {
      canvas_cleanup();
      try {
        game_state?.rust_state.free();
      } catch {}
    };
  }, [rust_module]);

  const snapshot_parameters = useRef<SnapshotParameters>({
    julia_c: {
      x: 0.0,
      y: 0.0,
    },
    zoom_factor: 1.0,
    center_of_view: [0.0, 0.0, 0.0],
  });

  return (
    <div class="demo">
      {error}
      <canvas class="layer-canvas" ref={layer_canvas_ref}></canvas>
      <RangeInput
        min={0}
        max={2}
        step={0.0001}
        on_change={(val) => {
          snapshot_parameters.current.julia_c.x = val;
          render_layer_ref.current?.(snapshot_parameters.current);
        }}
      />
      <RangeInput
        min={0}
        max={2}
        step={0.0001}
        on_change={(val) => {
          snapshot_parameters.current.julia_c.y = val;
          render_layer_ref.current?.(snapshot_parameters.current);
        }}
      />
      <RangeInput
        min={0.5}
        max={10000}
        step={0.01}
        on_change={(val) => {
          snapshot_parameters.current.zoom_factor = val;
          render_layer_ref.current?.(snapshot_parameters.current);
        }}
      />
      <CoordinateInput
        name="Center of View"
        min={-2}
        max={2}
        step={0.0001}
        on_change={(x, y, z) => {
          snapshot_parameters.current.center_of_view[0] = x;
          snapshot_parameters.current.center_of_view[1] = y;
          snapshot_parameters.current.center_of_view[2] = z;
          render_layer_ref.current?.(snapshot_parameters.current);
        }}
      />

      <canvas ref={canvas_ref}></canvas>
      <button onClick={capture_screenshot}>Download screenshot</button>
    </div>
  );
};

const RangeInput = ({
  on_change,
  ...props
}: {
  on_change: (val: number) => void;
} & JSX.IntrinsicElements["input"]) => {
  const [val, set_val] = useState(0);
  useEffect(() => {
    on_change(val);
  }, [val]);
  return (
    <input
      type="range"
      value={val}
      {...props}
      onInput={(e) => set_val(e.currentTarget.valueAsNumber)}
    />
  );
};

const CoordinateInput = ({
  on_change,
  name,
  min,
  max,
  step,
}: {
  on_change: (x: number, y: number, z: number) => void;
  name: string;
  min: number;
  max: number;
  step: number;
}) => {
  const [x, set_x] = useState(0.5);
  const [y, set_y] = useState(0.5);
  const [z, set_z] = useState(0.5);
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
          step={step}
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
          step={step}
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
          step={step}
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
          step={step}
          value={x}
          onInput={(e) => {
            set_x(e.currentTarget.valueAsNumber);
          }}
        />
        <input
          type="range"
          min={min}
          max={max}
          step={step}
          value={y}
          onInput={(e) => {
            set_y(e.currentTarget.valueAsNumber);
          }}
        />
        <input
          type="range"
          min={min}
          max={max}
          step={step}
          value={z}
          onInput={(e) => {
            set_z(e.currentTarget.valueAsNumber);
          }}
        />
      </div>
    </>
  );
};
