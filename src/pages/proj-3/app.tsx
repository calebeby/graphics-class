import {
  useEffect,
  useErrorBoundary,
  useMemo,
  useRef,
  useState,
} from "preact/hooks";
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
import obj from "./test.obj?raw";
import { load_obj } from "./load-obj";

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

interface Props {}

export const TransformDemo = ({}: Props) => {
  // eslint-disable-next-line @typescript-eslint/naming-convention
  const [error, _reset_error] = useErrorBoundary();
  const canvas_ref = useRef<HTMLCanvasElement>(null);
  const transform_matrix_ref = useRef<DOMMatrix>(new DOMMatrix());

  const transform_matrix = useMemo(() => {
    const perspective = new DOMMatrix();
    perspective.m34 = 0.3;

    return perspective;
  }, []);

  useEffect(() => {
    transform_matrix_ref.current = transform_matrix;
  }, [transform_matrix]);

  useEffect(() => {
    const canvas = canvas_ref.current!;

    const { cleanup } = init_canvas(
      canvas,
      transform_matrix_ref,
      load_obj(obj),
    );
    return () => cleanup();
  }, []);

  return (
    <div class="transform-demo">
      {error}
      <canvas ref={canvas_ref}></canvas>
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
