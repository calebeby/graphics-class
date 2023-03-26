extern crate nalgebra as na;
extern crate wasm_bindgen;

use na::{point, vector, Matrix4, Point3, Scale3, UnitVector3};
use wasm_bindgen::prelude::*;

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
}

#[wasm_bindgen]
impl GameState {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Default::default()
    }

    pub fn set_z(&mut self, z: f64) {
        self.camera_position.z = z + 0.3;
    }

    pub fn set_rotation(&mut self, rot: f64) {
        self.camera_direction =
            UnitVector3::new_normalize(vector![rot.sin(), self.camera_direction.y, rot.cos()]);
    }

    pub fn get_transform_matrix(&self) -> Vec<f64> {
        let mut persp = Matrix4::identity();
        persp.m43 = 0.4;
        let transform = Scale3::new(1.0, 1.0, 0.1).to_homogeneous()
            * persp
            * Matrix4::face_towards(
                &self.camera_position,
                &Point3::new(
                    self.camera_position.x + self.camera_direction.x,
                    self.camera_position.y + self.camera_direction.y,
                    self.camera_position.z + self.camera_direction.z,
                ),
                &vector![0.0, 1.0, 0.0],
            );

        transform.as_slice().to_vec()
    }
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            camera_position: point![0.0, 0.0, 0.0],
            camera_direction: UnitVector3::new_normalize(vector![0.0, 0.0, 1.0]),
        }
    }
}
