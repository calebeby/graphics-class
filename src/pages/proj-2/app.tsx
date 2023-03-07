import { useEffect, useMemo, useRef, useState } from "preact/hooks";
import "./app.css";
import { init_canvas } from "./graphics";
import {
  Axis,
  InvertTransform,
  ReflectTransform,
  RotateTransform,
  ScaleTransform,
  SkewTransform,
  Transform,
  TransformType,
  TranslateTransform,
  transform_types,
} from "./transforms";

const clone = <T extends Transform>(o: Readonly<T> | T) => {
  const t: T = Object.create(Object.getPrototypeOf(o));
  Object.assign(t, o);
  return t;
};

const hydrate_initial_transforms = (initial_transforms: Transform[]) =>
  initial_transforms.map((transform) => {
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
  }, [transforms, perspective_amount]);

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
          {transforms.map((transform, i) => {
            const mutate_copy = <T extends Transform>(
              mutate_fn: (t: T) => void,
            ) =>
              set_transforms((transforms) => {
                const cloned = [...transforms];
                const t_copy = clone(cloned[i] as T);
                mutate_fn(t_copy);
                cloned[i] = t_copy as any;
                return cloned;
              });
            return (
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
                  <ScaleControls
                    transform={transform}
                    mutate_copy={mutate_copy}
                  />
                ) : transform.type === TransformType.Translate ? (
                  <TranslateControls
                    transform={transform}
                    mutate_copy={mutate_copy}
                  />
                ) : transform.type === TransformType.Invert ? (
                  <InvertControls
                    transform={transform}
                    mutate_copy={mutate_copy}
                    transforms={transforms}
                  />
                ) : transform.type === TransformType.Reflect ? (
                  <ReflectControls
                    transform={transform}
                    mutate_copy={mutate_copy}
                  />
                ) : transform.type === TransformType.Skew ? (
                  <SkewControls
                    transform={transform}
                    mutate_copy={mutate_copy}
                  />
                ) : (
                  <RotateControls
                    transform={transform}
                    mutate_copy={mutate_copy}
                  />
                )}
              </li>
            );
          })}
        </ol>
      )}
      <div class="add-transform-controls">
        <select ref={select_ref as any}>
          <option value={TransformType.Translate}>Translate</option>
          <option value={TransformType.Scale}>Scale</option>
          <option value={TransformType.Rotate}>Rotate</option>
          <option value={TransformType.Invert}>Invert</option>
          <option value={TransformType.Reflect}>Reflect</option>
          <option value={TransformType.Skew}>Skew</option>
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
                ? new InvertTransform(
                    transforms.length > 0 ? transforms[0].id : -1,
                  )
                : type === TransformType.Reflect
                ? new ReflectTransform(Axis.X)
                : type === TransformType.Skew
                ? new SkewTransform(1, 1, Axis.X)
                : new ScaleTransform(1, 1, 1),
            ]);
          }}
        >
          Add Transform
        </button>
      </div>
      <RangeInput
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

const RangeInput = ({
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
        }, 10)
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

const RotateControls = ({
  transform,
  mutate_copy,
}: {
  transform: RotateTransform;
  mutate_copy: (cb: (t: RotateTransform) => void) => void;
}) => {
  return (
    <>
      <select
        value={transform.rotation_axis}
        onChange={(e) => {
          const axis = e.currentTarget.value as Axis;
          mutate_copy((t) => {
            t.rotation_axis = axis;
          });
        }}
      >
        <option value={Axis.X}>Rotate About X Axis</option>
        <option value={Axis.Y}>Rotate About Y Axis</option>
        <option value={Axis.Z}>Rotate About Z Axis</option>
      </select>
      <RangeInput
        name="Rotation (degrees)"
        value={transform.angle_degrees}
        range={180}
        on_input={(v) =>
          mutate_copy((t) => {
            t.angle_degrees = v;
          })
        }
      />
    </>
  );
};

const SkewControls = ({
  transform,
  mutate_copy,
}: {
  transform: SkewTransform;
  mutate_copy: (cb: (t: SkewTransform) => void) => void;
}) => {
  return (
    <>
      <select
        value={transform.skew_axis}
        onChange={(e) => {
          const axis = e.currentTarget.value as Axis;
          mutate_copy((t) => {
            t.skew_axis = axis;
          });
        }}
      >
        <option value={Axis.X}>Skew Y-Z</option>
        <option value={Axis.Y}>Skew X-Z</option>
        <option value={Axis.Z}>Skew X-Y</option>
      </select>
      <RangeInput
        name="S"
        value={transform.s}
        range={1}
        on_input={(s) =>
          mutate_copy((t) => {
            t.s = s;
          })
        }
      />
      <RangeInput
        name="T"
        value={transform.t}
        range={1}
        on_input={(t) =>
          mutate_copy((transform) => {
            transform.t = t;
          })
        }
      />
    </>
  );
};

const ReflectControls = ({
  transform,
  mutate_copy,
}: {
  transform: ReflectTransform;
  mutate_copy: (cb: (t: ReflectTransform) => void) => void;
}) => {
  return (
    <select
      value={transform.reflect_along_axis}
      onChange={(e) => {
        const axis = e.currentTarget.value as Axis;
        mutate_copy((t) => (t.reflect_along_axis = axis));
      }}
    >
      <option value={Axis.X}>Reflect across Y-Z plane</option>
      <option value={Axis.Y}>Reflect across X-Z plane</option>
      <option value={Axis.Z}>Reflect across X-Y plane</option>
    </select>
  );
};

const InvertControls = ({
  transform,
  mutate_copy,
  transforms,
}: {
  transform: InvertTransform;
  transforms: Readonly<Transform>[];
  mutate_copy: (cb: (t: InvertTransform) => void) => void;
}) => {
  return transforms.filter((t) => t !== transform).length > 0 ? (
    <select
      value={transform.id_to_invert}
      onChange={(e) =>
        mutate_copy((t) => {
          t.id_to_invert = Number(e.currentTarget.value);
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
  ) : (
    <p>Add another transform to invert it with this transform</p>
  );
};

const TranslateControls = ({
  transform,
  mutate_copy,
}: {
  transform: TranslateTransform;
  mutate_copy: (cb: (t: TranslateTransform) => void) => void;
}) => {
  return (
    <>
      <RangeInput
        name="Translate X"
        value={transform.x}
        range={0.5}
        on_input={(x) =>
          mutate_copy((t) => {
            t.x = x;
          })
        }
      />
      <RangeInput
        name="Translate Y"
        value={transform.y}
        range={0.5}
        on_input={(y) =>
          mutate_copy((t) => {
            t.y = y;
          })
        }
      />
      <RangeInput
        name="Translate Z"
        value={transform.z}
        range={0.5}
        on_input={(z) =>
          mutate_copy((t) => {
            t.z = z;
          })
        }
      />
    </>
  );
};

const ScaleControls = ({
  transform,
  mutate_copy,
}: {
  transform: ScaleTransform;
  mutate_copy: (cb: (t: ScaleTransform) => void) => void;
}) => {
  return (
    <>
      <RangeInput
        name="Scale X"
        value={transform.x}
        range={5}
        on_input={(x) =>
          mutate_copy((t) => {
            t.x = x;
          })
        }
      />
      <RangeInput
        name="Scale Y"
        value={transform.y}
        range={5}
        on_input={(y) =>
          mutate_copy((t) => {
            t.y = y;
          })
        }
      />
      <RangeInput
        name="Scale Z"
        value={transform.z}
        range={5}
        on_input={(z) =>
          mutate_copy((t) => {
            t.z = z;
          })
        }
      />
    </>
  );
};
