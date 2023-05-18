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
  map_z_to_n: boolean;
  layer_dimensions: number;
  min_parameter: number;
  max_parameter: number;
}

const default_snapshot_parameters: SnapshotParameters = {
  julia_c: {
    x: 0.0,
    y: 0.0,
  },
  zoom_factor: 1.0,
  center_of_view: [0.0, 0.0, 0.0],
  map_z_to_n: false,
  layer_dimensions: 1000,
  min_parameter: -0.1,
  max_parameter: 0.1,
};

const get_snapshot_params_from_url = (): SnapshotParameters | null => {
  try {
    const url = new URL(window.location.href);
    const params = url.searchParams.get("params");
    if (!params) return null;
    return JSON.parse(params);
  } catch {
    return null;
  }
};

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
  obj_vert_buffer?: WebGLBuffer | null;
  obj_colors_buffer?: WebGLBuffer | null;
  obj_num_points: number;
}

type UnPromise<T> = T extends Promise<infer R> ? R : never;
interface Props {}

export const Final = ({}: Props) => {
  // eslint-disable-next-line @typescript-eslint/naming-convention
  const [error, _reset_error] = useErrorBoundary();
  const canvas_ref = useRef<HTMLCanvasElement>(null);
  const layer_canvas_ref = useRef<HTMLCanvasElement>(null);
  const [rust_module, set_rust_module] = useState<rust.InitOutput | null>(null);
  const state_ref = useRef<GameState | null>(null);
  const graphics_ref = useRef<UnPromise<ReturnType<typeof init_canvas>>>();
  const render_layer_ref =
    useRef<(snapshot_parameters: SnapshotParameters) => Uint8Array>();

  const capture_screenshot = () => {
    const link = document.createElement("a");
    link.download = "fractal-screenshot.png";
    graphics_ref.current?.render();
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
      obj_num_points: 0,
    };
    state_ref.current = game_state;
    let canvas_cleanup = () => {};
    init_canvas(canvas, game_state).then((graphics) => {
      canvas_cleanup = graphics.cleanup;
      graphics_ref.current = graphics;
    });
    init_layer_canvas(layer_canvas, game_state).then(
      ({ cleanup, render: render_layer }) => {
        canvas_cleanup = cleanup;
        render_layer_ref.current = render_layer;
        render_2d(snapshot_parameters.current);
        requestIdleCallback(() => {
          setTimeout(() => {
            update_3d.current?.();
          }, 100);
        });
      },
    );
    return () => {
      canvas_cleanup();
      try {
        game_state?.rust_state.free();
      } catch {}
    };
  }, [rust_module]);

  const snapshot_parameters = useRef<SnapshotParameters>(
    get_snapshot_params_from_url() || default_snapshot_parameters,
  );

  const update_3d = useRef<null | (() => void)>(null);

  const render_2d = (snapshot_parameters: SnapshotParameters) => {
    const url = new URL(window.location.href);
    url.searchParams.set("params", JSON.stringify(snapshot_parameters));
    window.history.replaceState({}, "", url);
    const render_layer = render_layer_ref.current;
    if (render_layer) {
      // Render once for the visual update
      render_layer(snapshot_parameters);
      console.log(snapshot_parameters);
      update_3d.current = () => {
        let mesh;
        if (snapshot_parameters.map_z_to_n) {
          const pixels = render_layer(snapshot_parameters);
          mesh = rust.layer_to_mesh_n_to_z(pixels);
        } else {
          const min_val = snapshot_parameters.min_parameter;
          const max_val = snapshot_parameters.max_parameter;
          const num_layers = 200;
          const step = (max_val - min_val) / num_layers;
          const layer_size =
            // 4 for four channels (R, G, B, A)
            4 *
            snapshot_parameters.layer_dimensions *
            snapshot_parameters.layer_dimensions;
          const pixels_layers_buf = new Uint8Array(num_layers * layer_size);
          for (let i = 0; i < num_layers; i++) {
            const modified_params: SnapshotParameters = {
              ...snapshot_parameters,
              julia_c: {
                x: snapshot_parameters.julia_c.x,
                y: i * step + min_val,
              },
            };
            const pixels = render_layer(modified_params);
            pixels_layers_buf.set(pixels, layer_size * i);
          }
          // To pass it into rust, cannot be a 2d array, must be 1d
          mesh = rust.layers_to_mesh(pixels_layers_buf, num_layers);
        }
        graphics_ref.current?.set_mesh(mesh);
        graphics_ref.current?.render();
      };
      // Render again for the visual update in the top view
      render_layer(snapshot_parameters);
    }
  };

  return (
    <div class="demo">
      {error}
      <canvas class="layer-canvas" ref={layer_canvas_ref}></canvas>
      <RangeInput
        label="Layer width/height"
        min={100}
        max={5000}
        step={10}
        initial_value={snapshot_parameters.current.layer_dimensions}
        on_change={(val) => {
          snapshot_parameters.current.layer_dimensions = val;
          render_2d(snapshot_parameters.current);
        }}
      />
      <RangeInput
        label="Julia C (Re)"
        min={-2}
        max={2}
        step={0.0001}
        initial_value={snapshot_parameters.current.julia_c.x}
        on_change={(val) => {
          snapshot_parameters.current.julia_c.x = val;
          render_2d(snapshot_parameters.current);
        }}
      />
      <RangeInput
        label="Julia C (Im)"
        min={-2}
        max={2}
        step={0.0001}
        initial_value={snapshot_parameters.current.julia_c.y}
        on_change={(val) => {
          snapshot_parameters.current.julia_c.y = val;
          render_2d(snapshot_parameters.current);
        }}
      />
      <RangeInput
        label="Zoom"
        min={1}
        max={10}
        step={0.01}
        initial_value={Math.log(snapshot_parameters.current.zoom_factor / 0.1)}
        on_change={(val) => {
          snapshot_parameters.current.zoom_factor = 0.1 * Math.exp(val);
          render_2d(snapshot_parameters.current);
        }}
      />
      <RangeInput
        label="Parameter min"
        min={-2}
        max={2}
        step={0.001}
        initial_value={snapshot_parameters.current.min_parameter}
        on_change={(val) => {
          snapshot_parameters.current.min_parameter = val;
          // Set the y for the sake of previewing
          snapshot_parameters.current.julia_c.y = val;
          render_2d(snapshot_parameters.current);
        }}
      />
      <RangeInput
        label="Parameter max"
        min={-2}
        max={2}
        step={0.001}
        initial_value={snapshot_parameters.current.max_parameter}
        on_change={(val) => {
          snapshot_parameters.current.max_parameter = val;
          // Set the y for the sake of previewing
          snapshot_parameters.current.julia_c.y = val;
          render_2d(snapshot_parameters.current);
        }}
      />
      <label>
        Map Z to N
        <input
          type="checkbox"
          checked={snapshot_parameters.current.map_z_to_n}
          onChange={(e) => {
            snapshot_parameters.current.map_z_to_n = e.currentTarget.checked;
            render_2d(snapshot_parameters.current);
          }}
        />
      </label>
      <CoordinateInput
        name="Center of View"
        min={-2}
        max={2}
        step={0.0001}
        initial_value={snapshot_parameters.current.center_of_view}
        on_change={(coords) => {
          snapshot_parameters.current.center_of_view = coords;
          render_2d(snapshot_parameters.current);
        }}
      />
      <button onClick={() => update_3d.current?.()}>Update</button>

      <canvas ref={canvas_ref}></canvas>
      <button onClick={capture_screenshot}>Download screenshot</button>
    </div>
  );
};

const RangeInput = ({
  on_change,
  initial_value,
  name,
  label,
  ...props
}: {
  initial_value: number;
  label: string;
  on_change: (val: number) => void;
} & JSX.IntrinsicElements["input"]) => {
  const [val, set_val] = useState(initial_value);
  useEffect(() => {
    on_change(val);
  }, [val]);
  return (
    <label>
      {label}
      <input
        type="range"
        value={val}
        {...props}
        onInput={(e) => set_val(e.currentTarget.valueAsNumber)}
      />
      <input
        type="number"
        value={val}
        onInput={(e) => set_val(e.currentTarget.valueAsNumber)}
      />
    </label>
  );
};

const CoordinateInput = ({
  on_change,
  name,
  min,
  max,
  step,
  initial_value,
}: {
  on_change: (coords: [x: number, y: number, z: number]) => void;
  initial_value: [x: number, y: number, z: number];
  name: string;
  min: number;
  max: number;
  step: number;
}) => {
  const [x, set_x] = useState(initial_value[0]);
  const [y, set_y] = useState(initial_value[1]);
  const [z, set_z] = useState(initial_value[2]);
  useEffect(() => {
    on_change([x, y, z]);
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
