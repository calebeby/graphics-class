import { useEffect, useMemo, useRef, useState } from "preact/hooks";
import "./app.css";
import { init_canvas } from "./graphics";

enum TransformType {
  Rotate = "rotate",
  Scale = "scale",
}

enum Axis {
  X,
  Y,
  Z,
}

export interface BaseTransform {
  id: number;
  type: TransformType;
  get_matrix(): DOMMatrix;
}

let id_counter = 0;
const get_id = () => ++id_counter;

class ScaleTransform implements BaseTransform {
  id: number = get_id();
  type = TransformType.Scale as const;
  x: number;
  y: number;
  z: number;

  constructor(x: number, y: number, z: number) {
    this.x = x;
    this.y = y;
    this.z = z;
  }

  get_matrix(): DOMMatrix {
    const m = new DOMMatrix();
    m.m11 = this.x;
    m.m22 = this.y;
    m.m33 = this.z;
    return m;
  }
}

class RotateTransform implements BaseTransform {
  id: number = get_id();
  type = TransformType.Rotate as const;
  angle_degrees: number;
  rotation_axis: Axis;

  constructor(angle_degrees: number, rotation_axis: Axis) {
    this.angle_degrees = angle_degrees;
    this.rotation_axis = rotation_axis;
  }

  get_matrix(): DOMMatrix {
    const m = new DOMMatrix();
    const cos = Math.cos(this.angle_degrees);
    const sin = Math.sin(this.angle_degrees);
    if (this.rotation_axis === Axis.X) {
      // Rotate about X:
      m.m22 = cos;
      m.m23 = -sin;
      m.m32 = sin;
      m.m33 = cos;
    } else if (this.rotation_axis === Axis.Y) {
      // Rotate about Y:
      m.m11 = cos;
      m.m13 = sin;
      m.m31 = -sin;
      m.m33 = cos;
    } else {
      // Rotate about Z:
      m.m11 = cos;
      m.m12 = -sin;
      m.m21 = sin;
      m.m22 = cos;
    }

    return m;
  }
}

type Transform = ScaleTransform | RotateTransform;

interface Props {
  initial_transforms?: Transform[];
}

export const TransformDemo = ({ initial_transforms = [] }: Props) => {
  const canvas_ref = useRef<HTMLCanvasElement>(null);
  const select_ref = useRef<HTMLSelectElement>(null);
  const transform_matrix_ref = useRef<DOMMatrix>(new DOMMatrix());

  const [transforms, set_transforms] =
    useState<Transform[]>(initial_transforms);

  const transform_matrix = useMemo(
    () =>
      transforms.reduce(
        (combined_matrix, transform) =>
          // (transform * combined) reverses the order,
          // to make the actual transformations happen top-to-bottom as displayed
          transform.get_matrix().multiply(combined_matrix),
        new DOMMatrix(),
      ),
    [transforms],
  );

  useEffect(() => {
    transform_matrix_ref.current = transform_matrix;
  }, [transform_matrix]);

  const transform_matrix_str = [...transform_matrix.toFloat64Array()].map(
    (num) => num.toFixed(2).padStart(5),
  );

  useEffect(() => {
    const canvas = canvas_ref.current!;

    const { cleanup } = init_canvas(canvas, transform_matrix_ref);
    return () => cleanup();
  }, []);

  return (
    <div class="transform-demo">
      <canvas ref={canvas_ref}></canvas>
      <select ref={select_ref as any}>
        <option value="scale">Scale</option>
        <option value="rotate">Rotate</option>
      </select>
      <button
        onClick={() => {
          const type = select_ref.current!.value as TransformType;
          set_transforms((t) => [
            ...t,
            type === TransformType.Rotate
              ? new RotateTransform(0, Axis.X)
              : new ScaleTransform(1, 1, 1),
          ]);
        }}
      >
        Add Transform
      </button>
      <pre>{`${transform_matrix_str.slice(0, 4).join(" ")}
${transform_matrix_str.slice(4, 8).join(" ")}
${transform_matrix_str.slice(8, 12).join(" ")}
${transform_matrix_str.slice(12, 16).join(" ")}
`}</pre>
      {transforms.length > 0 && (
        <ol>
          {transforms.map((transform, i) => (
            <li key={transform.id} data-key={transform.id}>
              <div class="transform-title">
                <h1>
                  {transform.type === TransformType.Rotate ? "Rotate" : "Scale"}
                </h1>
                <div class="transform-builtin-controls">
                  <button
                    disabled={i === 0}
                    onClick={() => {
                      let tmp = transforms[i];
                      transforms[i] = transforms[i - 1];
                      transforms[i - 1] = tmp;
                      set_transforms([...transforms]);
                    }}
                  >
                    Move Up
                  </button>
                  <button
                    disabled={i === transforms.length - 1}
                    onClick={() => {
                      let tmp = transforms[i];
                      transforms[i] = transforms[i + 1];
                      transforms[i + 1] = tmp;
                      set_transforms([...transforms]);
                    }}
                  >
                    Move Down
                  </button>
                  <button
                    onClick={() => {
                      set_transforms((all) =>
                        all.filter((t) => t !== transform),
                      );
                    }}
                  >
                    Remove
                  </button>
                </div>
              </div>
              {transform.type === TransformType.Scale ? (
                <>
                  <TransformControl
                    name="Scale X"
                    value={transform.x}
                    range={5}
                    on_input={(v) => {
                      transform.x = v;
                      set_transforms((t) => [...t]);
                    }}
                  />
                  <TransformControl
                    name="Scale Y"
                    value={transform.y}
                    range={5}
                    on_input={(v) => {
                      transform.y = v;
                      set_transforms((t) => [...t]);
                    }}
                  />
                  <TransformControl
                    name="Scale Z"
                    value={transform.z}
                    range={5}
                    on_input={(v) => {
                      transform.z = v;
                      set_transforms((t) => [...t]);
                    }}
                  />
                </>
              ) : (
                <>
                  <select
                    onChange={(e) => {
                      const v = e.currentTarget.value;
                      if (v === "x") {
                        transform.rotation_axis = Axis.X;
                      } else if (v === "y") {
                        transform.rotation_axis = Axis.Y;
                      } else {
                        transform.rotation_axis = Axis.Z;
                      }
                      set_transforms((t) => [...t]);
                    }}
                  >
                    <option value="x">Rotate About X Axis</option>
                    <option value="y">Rotate About Y Axis</option>
                    <option value="z">Rotate About Z Axis</option>
                  </select>
                  <TransformControl
                    name="Rotation (degrees)"
                    value={(transform.angle_degrees * 180) / Math.PI}
                    range={180}
                    on_input={(v) => {
                      transform.angle_degrees = (v * Math.PI) / 180;
                      set_transforms((t) => [...t]);
                    }}
                  />
                </>
              )}
            </li>
          ))}
        </ol>
      )}
      {transforms !== initial_transforms && (
        <button
          onClick={() => {
            set_transforms(initial_transforms);
          }}
        >
          Revert Changes
        </button>
      )}
    </div>
  );
};

const TransformControl = ({
  name,
  value,
  range,
  on_input,
}: {
  name: string;
  value: number;
  range: number;
  on_input: (value: number) => void;
}) => {
  return (
    <label class="transform-control">
      <span>{name}</span>
      <input
        type="range"
        value={value}
        min={-range}
        max={range}
        step={0.01}
        onInput={(e) => {
          on_input(e.currentTarget.valueAsNumber);
        }}
      />
      <span>{value.toFixed(2)}</span>
    </label>
  );
};
