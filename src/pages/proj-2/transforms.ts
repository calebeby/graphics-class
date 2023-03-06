export enum Axis {
  X = "x",
  Y = "y",
  Z = "z",
}

export enum TransformType {
  Rotate = "rotate",
  Scale = "scale",
  Translate = "translate",
  Invert = "invert",
  Reflect = "reflect",
  Skew = "skew",
}

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

export class SkewTransform implements BaseTransform {
  id = get_id();
  type = TransformType.Skew as const;
  s: number;
  t: number;
  skew_axis: Axis;

  get_name() {
    return `Skew${[Axis.X, Axis.Y, Axis.Z]
      .filter((ax) => ax !== this.skew_axis)
      .join("")
      .toUpperCase()}(s=${this.s.toFixed(2)}, t=${this.t.toFixed(2)})`;
  }

  constructor(s: number, t: number, skew_axis: Axis) {
    this.skew_axis = skew_axis;
    this.s = s;
    this.t = t;
  }

  get_matrix(): DOMMatrix {
    const m = new DOMMatrix();
    if (this.skew_axis === Axis.X) {
      m.m21 = this.s;
      m.m31 = this.t;
    } else if (this.skew_axis === Axis.Y) {
      m.m12 = this.s;
      m.m32 = this.t;
    } else {
      m.m13 = this.s;
      m.m23 = this.t;
    }

    return m;
  }
}

export class ReflectTransform implements BaseTransform {
  id = get_id();
  type = TransformType.Reflect as const;
  reflect_along_axis: Axis;

  get_name() {
    return `Reflect(${[Axis.X, Axis.Y, Axis.Z]
      .filter((ax) => ax !== this.reflect_along_axis)
      .join("-")
      .toUpperCase()} Plane)`;
  }

  constructor(reflect_along_axis: Axis) {
    this.reflect_along_axis = reflect_along_axis;
  }

  get_matrix(): DOMMatrix {
    const m = new DOMMatrix();
    if (this.reflect_along_axis === Axis.X) {
      m.m11 = -1;
    } else if (this.reflect_along_axis === Axis.Y) {
      m.m22 = -1;
    } else {
      m.m33 = -1;
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

export const transform_types = {
  scale: ScaleTransform,
  rotate: RotateTransform,
  translate: TranslateTransform,
  invert: InvertTransform,
  reflect: ReflectTransform,
  skew: SkewTransform,
} as const;

export type Transform =
  (typeof transform_types)[keyof typeof transform_types]["prototype"];
