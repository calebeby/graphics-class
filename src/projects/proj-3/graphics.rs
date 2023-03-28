extern crate nalgebra as na;
extern crate wasm_bindgen;

use na::{point, vector, Matrix4, Point3, Scale3, Translation3, UnitVector3, Vector3};
use wasm_bindgen::prelude::*;

static UP: Vector3<f64> = vector![0.0, 1.0, 0.0];

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

    pub fn update(
        &mut self,
        input_w: bool,
        input_a: bool,
        input_s: bool,
        input_d: bool,
        delta_time_ms: usize,
    ) {
        // In seconds, time since last render
        let dt = delta_time_ms as f64 / 1000.0;
        self.camera_position += self.camera_velocity * dt;
        let accel = 5.0;
        let decel = 0.05;
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
        // console_log!("camera position, {:?}", self.camera_position);
        // console_log!(
        //     "camera direction, {:?}, camera position: {:?}",
        //     self.camera_direction,
        //     self.camera_position
        // );
    }

    pub fn set_x(&mut self, x: f64) {
        self.camera_position.x = x;
    }

    pub fn set_y(&mut self, y: f64) {
        self.camera_position.y = y;
    }

    pub fn set_z(&mut self, z: f64) {
        self.camera_position.z = z;
    }

    pub fn set_rotation(&mut self, rot: f64) {
        self.camera_direction =
            UnitVector3::new_normalize(vector![rot.sin(), self.camera_direction.y, rot.cos()]);
    }

    pub fn world_to_camera(&self) -> TransformMatrix {
        let mut persp = Matrix4::identity();
        persp.m43 = 0.4;
        TransformMatrix(
            // Scale everything in the z direction down
            // (does not affect the positions of any vertices)
            // This just reduces the scope of z values to reduce clipping
            Scale3::new(1.0, 1.0, 0.1).to_homogeneous()
                * persp
                * Matrix4::new_translation(&Vector3::new(
                    self.camera_position.x,
                    self.camera_position.y,
                    self.camera_position.z,
                ))
                * Matrix4::look_at_rh(
                    // &(self.camera_position / (self.camera_position - Point3::origin()).magnitude()),
                    // &Point3::origin(),
                    &Point3::origin(),
                    &Point3::new(
                        self.camera_direction.x,
                        self.camera_direction.y,
                        self.camera_direction.z,
                    ),
                    &UP,
                ),
            // * Matrix4::face_towards(
            //     &self.camera_position,
            //     &Point3::new(
            //         self.camera_position.x + self.camera_direction.x,
            //         self.camera_position.y + self.camera_direction.y,
            //         self.camera_position.z + self.camera_direction.z,
            //     ),
            //     &vector![0.0, 1.0, 0.0],
            // ),
        )
    }
}

impl Default for GameState {
    #[inline]
    fn default() -> Self {
        Self {
            camera_position: point![0.0, 0.0, 0.0],
            camera_direction: UnitVector3::new_normalize(vector![0.0, 0.0, 1.0]),
            camera_velocity: Vector3::zeros(),
        }
    }
}
