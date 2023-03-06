import { useEffect, useMemo, useRef, useState } from "preact/hooks";
import "./app.css";
import { init_canvas } from "./graphics";

enum TransformType {
  Rotate = "rotate",
  Scale = "scale",
  Translate = "translate",
  Invert = "invert",
}

export enum Axis {
  X = "x",
  Y = "y",
  Z = "z",
}

const clone = <T extends Transform>(o: Readonly<T> | T) => {
  const t: T = Object.create(Object.getPrototypeOf(o));
  Object.assign(t, o);
  return t;
};

export interface BaseTransform {
  id: number;
  type: TransformType;
  get_matrix(all_transforms: Readonly<Transform>[]): DOMMatrix;
  get_name(all_transforms: Readonly<Transform>[]): string;
}

let id_counter = 0;
const get_id = () => ++id_counter;

export class ScaleTransform implements BaseTransform {
  id = get_id();
  type = TransformType.Scale as const;
  x: number;
  y: number;
  z: number;

  get_name() {
    return `Scale(${this.x.toFixed(2)}, ${this.y.toFixed(2)}, ${this.z.toFixed(
      2,
    )})`;
  }

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

export class TranslateTransform implements BaseTransform {
  id = get_id();
  type = TransformType.Translate as const;
  x: number;
  y: number;
  z: number;

  get_name() {
    return `Translate(${this.x.toFixed(2)}, ${this.y.toFixed(
      2,
    )}, ${this.z.toFixed(2)})`;
  }

  constructor(x: number, y: number, z: number) {
    this.x = x;
    this.y = y;
    this.z = z;
  }

  get_matrix(): DOMMatrix {
    const m = new DOMMatrix();
    m.m41 = this.x;
    m.m42 = this.y;
    m.m43 = this.z;
    return m;
  }
}

export class RotateTransform implements BaseTransform {
  id = get_id();
  type = TransformType.Rotate as const;
  angle_degrees: number;
  rotation_axis: Axis;

  get_name() {
    return `Rotate${this.rotation_axis.toUpperCase()}(${this.angle_degrees.toFixed(
      2,
    )}°)`;
  }

  constructor(angle_degrees: number, rotation_axis: Axis) {
    this.angle_degrees = angle_degrees;
    this.rotation_axis = rotation_axis;
  }

  get_matrix(): DOMMatrix {
    const m = new DOMMatrix();
    const cos = Math.cos((Math.PI * this.angle_degrees) / 180);
    const sin = Math.sin((Math.PI * this.angle_degrees) / 180);
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

export class InvertTransform implements BaseTransform {
  id = get_id();
  type = TransformType.Invert as const;
  id_to_invert: number;

  get_name(all_transforms: readonly Transform[]): string {
    const transform = this.find_corresponding_transform(all_transforms);
    return `Invert(${
      transform ? transform.get_name(all_transforms) : "Unknown"
    })`;
  }

  constructor(id_to_invert: number) {
    this.id_to_invert = id_to_invert;
  }

  find_corresponding_transform(
    transforms: readonly Transform[],
  ): Transform | undefined {
    return transforms.find((t) => t.id === this.id_to_invert);
  }

  get_matrix(transforms: readonly Transform[]): DOMMatrix {
    const transform = this.find_corresponding_transform(transforms);
    return transform
      ? transform.get_matrix(transforms).inverse()
      : new DOMMatrix();
  }
}

const transform_types = {
  scale: ScaleTransform,
  rotate: RotateTransform,
  translate: TranslateTransform,
  invert: InvertTransform,
} as const;

type Transform =
  (typeof transform_types)[keyof typeof transform_types]["prototype"];

const hydrate_initial_transforms = (initial_transforms: Transform[]) =>
  initial_transforms.map((transform) => {
    id_counter = Math.max(transform.id, id_counter);
    // This "hydrates" the passed-in transforms from astro
    // In astro's SSR it serializes them to JSON which means when they are revived,
    // they are plain objects (not instances of classes)
    // This switches them back to instances of classes
    if (transform.constructor === Object) {
      const base = transform_types[transform.type];
      const t = Object.create(base.prototype);
      Object.assign(t, transform);
      return t;
    } else {
      return transform;
    }
  });

interface Props {
  initial_transforms?: Transform[];
}

export const TransformDemo = ({
  initial_transforms: initial_transforms_unnormalized = [],
}: Props) => {
  const initial_transforms = useMemo(
    () => hydrate_initial_transforms(initial_transforms_unnormalized),
    [initial_transforms_unnormalized],
  );
  const canvas_ref = useRef<HTMLCanvasElement>(null);
  const select_ref = useRef<HTMLSelectElement>(null);
  const transform_matrix_ref = useRef<DOMMatrix>(new DOMMatrix());

  const [transforms, set_transforms] =
    useState<Readonly<Transform>[]>(initial_transforms);

  const [perspective_amount, set_perspective_amount] = useState(0);

  const transform_matrix = useMemo(() => {
    const combined = transforms.reduce(
      (combined_matrix, transform) =>
        // (transform * combined) reverses the order,
        // to make the actual transformations happen top-to-bottom as displayed
        transform.get_matrix(transforms).multiply(combined_matrix),
      new DOMMatrix(),
    );

    const perspective = new DOMMatrix();
    perspective.m34 = perspective_amount;

    return perspective.multiply(combined);
  }, [transforms]);

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
                <h2>
                  {`(${i + 1}) `}
                  <code>{transform.get_name(transforms)}</code>
                </h2>
                <div class="transform-builtin-controls">
                  {transforms.length > 1 && (
                    <>
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
                    </>
                  )}
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
                    on_input={(v) =>
                      set_transforms((t) => {
                        const cloned = [...t];
                        const t2 = clone(cloned[i] as typeof transform);
                        t2.x = v;
                        cloned[i] = t2;
                        return cloned;
                      })
                    }
                  />
                  <TransformControl
                    name="Scale Y"
                    value={transform.y}
                    range={5}
                    on_input={(v) =>
                      set_transforms((t) => {
                        const cloned = [...t];
                        const t2 = clone(cloned[i] as typeof transform);
                        t2.y = v;
                        cloned[i] = t2;
                        return cloned;
                      })
                    }
                  />
                  <TransformControl
                    name="Scale Z"
                    value={transform.z}
                    range={5}
                    on_input={(v) =>
                      set_transforms((t) => {
                        const cloned = [...t];
                        const t2 = clone(cloned[i] as typeof transform);
                        t2.z = v;
                        cloned[i] = t2;
                        return cloned;
                      })
                    }
                  />
                </>
              ) : transform.type === TransformType.Translate ? (
                <>
                  <TransformControl
                    name="Translate X"
                    value={transform.x}
                    range={0.4}
                    on_input={(v) =>
                      set_transforms((t) => {
                        const cloned = [...t];
                        const t2 = clone(cloned[i] as typeof transform);
                        t2.x = v;
                        cloned[i] = t2;
                        return cloned;
                      })
                    }
                  />
                  <TransformControl
                    name="Translate Y"
                    value={transform.y}
                    range={0.4}
                    on_input={(v) =>
                      set_transforms((t) => {
                        const cloned = [...t];
                        const t2 = clone(cloned[i] as typeof transform);
                        t2.y = v;
                        cloned[i] = t2;
                        return cloned;
                      })
                    }
                  />
                  <TransformControl
                    name="Translate Z"
                    value={transform.z}
                    range={0.4}
                    on_input={(v) =>
                      set_transforms((t) => {
                        const cloned = [...t];
                        const t2 = clone(cloned[i] as typeof transform);
                        t2.z = v;
                        cloned[i] = t2;
                        return cloned;
                      })
                    }
                  />
                </>
              ) : transform.type === TransformType.Invert ? (
                <>
                  <select
                    value={transform.id_to_invert}
                    onChange={(e) =>
                      set_transforms((t) => {
                        const cloned = [...t];
                        const t2 = clone(cloned[i] as typeof transform);
                        t2.id_to_invert = Number(e.currentTarget.value);
                        cloned[i] = t2;
                        return cloned;
                      })
                    }
                  >
                    {transforms
                      .map((t, i) => [t, i] as const)
                      .filter(([t]) => t !== transform)
                      .map(([t, i]) => (
                        <option value={t.id}>
                          {`Invert (${i + 1}) ${t.get_name(transforms)}`}
                        </option>
                      ))}
                  </select>
                </>
              ) : (
                <>
                  <select
                    value={transform.rotation_axis}
                    onChange={(e) => {
                      const v = e.currentTarget.value;
                      set_transforms((t) => {
                        const cloned = [...t];
                        const t2 = clone(cloned[i] as typeof transform);
                        if (v === "x") {
                          t2.rotation_axis = Axis.X;
                        } else if (v === "y") {
                          t2.rotation_axis = Axis.Y;
                        } else {
                          t2.rotation_axis = Axis.Z;
                        }
                        cloned[i] = t2;
                        return cloned;
                      });
                    }}
                  >
                    <option value="x">Rotate About X Axis</option>
                    <option value="y">Rotate About Y Axis</option>
                    <option value="z">Rotate About Z Axis</option>
                  </select>
                  <TransformControl
                    name="Rotation (degrees)"
                    value={transform.angle_degrees}
                    range={180}
                    on_input={(v) =>
                      set_transforms((t) => {
                        const cloned = [...t];
                        const t2 = clone(cloned[i] as typeof transform);
                        t2.angle_degrees = v;
                        cloned[i] = t2;
                        return cloned;
                      })
                    }
                  />
                </>
              )}
            </li>
          ))}
        </ol>
      )}
      <div class="add-transform-controls">
        <select ref={select_ref as any}>
          <option value={TransformType.Translate}>Translate</option>
          <option value={TransformType.Scale}>Scale</option>
          <option value={TransformType.Rotate}>Rotate</option>
          <option value={TransformType.Invert}>Invert</option>
        </select>
        <button
          onClick={() => {
            const type = select_ref.current!.value as TransformType;
            set_transforms((t) => [
              ...t,
              type === TransformType.Rotate
                ? new RotateTransform(0, Axis.X)
                : type === TransformType.Translate
                ? new TranslateTransform(0, 0, 0)
                : type === TransformType.Invert
                ? new InvertTransform(transforms[0].id)
                : new ScaleTransform(1, 1, 1),
            ]);
          }}
        >
          Add Transform
        </button>
      </div>
      <TransformControl
        name="Perspective Amount"
        value={perspective_amount}
        range={1.5}
        on_input={(v) => set_perspective_amount(v)}
      />
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
  const [is_animated, set_is_animated] = useState(false);
  // on_input is stored in the ref so the most up-to-date callback is called in the setInterval loop
  const on_input_ref = useRef(on_input);
  useEffect(() => {
    on_input_ref.current = on_input;
  }, [on_input]);
  useEffect(() => {
    const start = new Date().getTime();
    const i = is_animated
      ? setInterval(() => {
          on_input_ref.current(
            range * Math.sin((new Date().getTime() - start) / 1000),
          );
        }, 20)
      : false;
    return () => i && clearInterval(i);
  }, [is_animated]);
  return (
    <label class="transform-control">
      <span>{name}</span>
      <input
        type="range"
        disabled={is_animated}
        value={value}
        min={-range}
        max={range}
        step={0.01}
        onInput={(e) => {
          on_input(e.currentTarget.valueAsNumber);
        }}
      />
      <input
        type="number"
        disabled={is_animated}
        size={6}
        value={value.toFixed(2)}
        onChange={(e) => {
          on_input(e.currentTarget.valueAsNumber);
        }}
      />
      {is_animated ? (
        <button onClick={() => set_is_animated(false)}>Stop</button>
      ) : (
        <button onClick={() => set_is_animated(true)}>Animate</button>
      )}
    </label>
  );
};
