pub(crate) extern crate nalgebra;
pub(crate) extern crate num_traits;
pub(crate) extern crate wasm_bindgen;
mod bounding_box;
mod face;
mod ray;

use std::f64::consts::PI;

use nalgebra::{point, vector, Matrix4, Point3, Scale3, Translation3, UnitVector3, Vector3};
use wasm_bindgen::prelude::*;

pub(crate) trait Number:
    'static
    + std::fmt::Debug
    + Copy
    + nalgebra::ClosedAdd
    + nalgebra::ClosedSub
    + nalgebra::ClosedMul
    + num_traits::identities::Zero
    + nalgebra::SimdComplexField
    + nalgebra::SimdRealField
    + num_traits::Float
    + std::convert::Into<f64>
{
}
impl Number for f64 {}
impl Number for f32 {}

static UP: Vector3<f64> = vector![0.0, -1.0, 0.0];

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log(s: String);
}

macro_rules! console_log {
    ($($t:tt)*) => {
        #[cfg(target_arch = "wasm32")] {
            log(format!($($t)*));
        }
        #[cfg(not(target_arch = "wasm32"))] {
            println!($($t)*);
        }
    };
}

#[wasm_bindgen]
pub struct GameState {
    camera_position: Point3<f64>,
    camera_direction: UnitVector3<f64>,
    camera_velocity: Vector3<f64>,
}

/// A little wrapper around the nalgebra matrix4 class, for JS use,
/// so we only have to generate binding code for the methods we actually need.
#[wasm_bindgen]
pub struct TransformMatrix(Matrix4<f64>);

#[wasm_bindgen]
impl TransformMatrix {
    #[wasm_bindgen(constructor)]
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self(Translation3::new(x, y, z).to_homogeneous())
    }
    pub fn to_f64_array(&self) -> Vec<f64> {
        self.0.as_slice().to_vec()
    }
    pub fn times(&self, other: &TransformMatrix) -> TransformMatrix {
        Self(self.0 * other.0)
    }
}

#[wasm_bindgen]
impl GameState {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Default::default()
    }

    // It would be a good idea to have this accept the arguments as a struct,
    // but then I'd have to deal with JS binding generation for that struct.
    #[allow(clippy::too_many_arguments)]
    pub fn update(
        &mut self,
        input_w: bool,
        input_a: bool,
        input_s: bool,
        input_d: bool,
        cursor_x: f64,
        cursor_y: f64,
        delta_time_ms: usize,
    ) {
        // In seconds, time since last render
        let dt = delta_time_ms as f64 / 1000.0;
        // Ranges from -pi to +pi
        let angle_x = cursor_x * PI;
        let angle_y = cursor_y * PI;
        self.camera_direction = UnitVector3::new_normalize(
            vector![angle_x.sin(), 0.0, -angle_x.cos()]
                + vector![0.0, -angle_y.sin(), -angle_y.cos()],
        );
        self.camera_position += self.camera_velocity * dt;
        let accel = 15.0;
        let decel = 0.15;
        let forwards = self.camera_direction.into_inner();
        if input_w {
            self.camera_velocity -= accel * dt * forwards;
        } else if input_s {
            self.camera_velocity += accel * dt * forwards;
        } else {
            self.camera_velocity -= decel * (self.camera_velocity.dot(&forwards)) * forwards;
        }
        let right = forwards.cross(&UP);
        if input_a {
            self.camera_velocity -= accel * dt * right;
        } else if input_d {
            self.camera_velocity += accel * dt * right;
        } else {
            self.camera_velocity -= decel * (self.camera_velocity.dot(&right)) * right;
        }
        // There was a bug where when turning the camera it could continue drifting
        // Since the third component of the "camera directions" didn't have code to handle deceleration
        // So I added this
        let camera_up = forwards.cross(&right);
        self.camera_velocity -= decel * (self.camera_velocity.dot(&camera_up)) * camera_up;
    }

    pub fn world_to_camera(&self) -> TransformMatrix {
        // Scale everything in the z direction down
        // (does not affect the positions of any vertices)
        // This just reduces the scope of z values to reduce clipping
        let transforms = Scale3::new(1.0, 1.0, 0.01).to_homogeneous()
            * Matrix4::new_perspective(1.0, 30.0, 0.01, 100.0)
            * Matrix4::look_at_rh(
                &Point3::origin(),
                &Point3::new(
                    self.camera_direction.x,
                    self.camera_direction.y,
                    self.camera_direction.z,
                ),
                &UP,
            )
            * Matrix4::new_translation(&Vector3::new(
                self.camera_position.x,
                self.camera_position.y,
                self.camera_position.z,
            ));
        TransformMatrix(transforms)
    }
}

impl Default for GameState {
    #[inline]
    fn default() -> Self {
        Self {
            camera_position: point![0.0, 0.0, -2.0],
            camera_direction: UnitVector3::new_normalize(vector![0.0, 0.0, -1.0]),
            camera_velocity: Vector3::zeros(),
        }
    }
}

/// Breaks a polygon into a bunch of triangle points (so they can be psased directly into webgl)
fn break_into_triangles<'pts, Point>(face_points: &[&'pts Point]) -> Vec<&'pts Point> {
    face_points[1..]
        .windows(2)
        .flat_map(|pair_of_points| vec![face_points[0], pair_of_points[0], pair_of_points[1]])
        .collect()
}

#[wasm_bindgen]
pub fn generate_maze_points() -> Vec<f32> {
    let front_right_bottom = Point3::new(0.0, 0.0, 1.0);
    let front_left_bottom = Point3::new(-1.0, 0.0, 1.0);
    let front_right_top = Point3::new(0.0, 1.0, 1.0);
    let front_left_top = Point3::new(-1.0, 1.0, 1.0);

    let back_right_bottom = Point3::new(0.0, 0.0, 0.0);
    let back_left_bottom = Point3::new(-1.0, 0.0, 0.0);
    let back_right_top = Point3::new(0.0, 1.0, 0.0);
    let back_left_top = Point3::new(-1.0, 1.0, 0.0);

    let faces = [
        [
            &front_right_top,
            &front_right_bottom,
            &front_left_bottom,
            &front_left_top,
        ],
        [
            &front_right_top,
            &back_right_top,
            &back_right_bottom,
            &front_right_bottom,
        ],
        [
            &front_left_top,
            &back_left_top,
            &back_left_bottom,
            &front_left_bottom,
        ],
        [
            &front_right_top,
            &back_right_top,
            &back_left_top,
            &front_left_top,
        ],
        [
            &front_right_bottom,
            &back_right_bottom,
            &back_left_bottom,
            &front_left_bottom,
        ],
        [
            &back_right_top,
            &back_right_bottom,
            &back_left_bottom,
            &back_left_top,
        ],
    ];

    let points: Vec<_> = faces
        .iter()
        .flat_map(|face| break_into_triangles(face))
        .collect();

    points
        .iter()
        .flat_map(|point| [point.x, point.y, point.z, 1.0])
        .collect()
}
