extern crate console_error_panic_hook;
pub(crate) extern crate nalgebra;
pub(crate) extern crate num_traits;
pub(crate) extern crate parry2d_f64 as parry2d;
pub(crate) extern crate parry3d_f64 as parry3d;
pub(crate) extern crate rand;
pub(crate) extern crate wasm_bindgen;
mod bounding_box;
mod face;
mod load_obj;
mod maze;
mod ray;

use face::Face;
use maze::{Environment, EnvironmentIdentifier, Maze};
use nalgebra::{
    vector, Matrix4, Point3, Scale3, Translation3, UnitQuaternion, UnitVector3, Vector3,
};
use wasm_bindgen::prelude::*;

use crate::ray::Ray;

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
    + std::convert::From<f32>
{
}
impl Number for f64 {}
impl Number for f32 {}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log(s: String);
}

#[macro_export]
macro_rules! console_log {
    ($($t:tt)*) => {
        #[cfg(target_arch = "wasm32")] {
            $crate::log(format!($($t)*));
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
    aspect_ratio: f64,
    maze: Maze,
    current_environment: EnvironmentIdentifier,
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
        console_error_panic_hook::set_once();
        Default::default()
    }

    fn up(&self) -> UnitVector3<f64> {
        self.environment().up(self.camera_position)
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
        cursor_movement_x: f64,
        cursor_movement_y: f64,
        delta_time_ms: usize,
    ) {
        let up = self.up().into_inner();
        // In seconds, time since last render
        let dt = delta_time_ms as f64 / 1000.0;
        let rotation_scale = 0.01;
        let rotation_x = rotation_scale * cursor_movement_x;
        let rotation_y = rotation_scale * cursor_movement_y;
        let forwards = self.camera_direction.into_inner();
        let right = forwards.cross(&up);
        self.camera_direction = UnitVector3::new_normalize(
            UnitQuaternion::new(up * rotation_x + right * rotation_y)
                .transform_vector(&self.camera_direction),
        );
        let new_camera_position = self.camera_position + self.camera_velocity * dt;
        let camera_movement_ray = Ray::new(self.camera_position, new_camera_position);
        let environment = self.environment();
        // If the camera passes through an "exit face",
        // the camera has switched to a new environment.
        let exit_faces = environment.exit_faces();
        let new_environment = exit_faces.iter().find_map(|(new_environment, exit_face)| {
            let crosses = parry3d::query::intersection_test(
                &nalgebra::Isometry::identity(),
                &exit_face.to_convex_polyhedron(),
                &nalgebra::Isometry::identity(),
                &camera_movement_ray.to_segment(),
            )
            .unwrap();
            if crosses {
                console_log!("Crossed into {:?}", new_environment);
                Some(new_environment)
            } else {
                None
            }
        });
        if let Some(&new_environment) = new_environment {
            self.current_environment = new_environment;
            self.camera_position = new_camera_position;
        } else {
            let has_intersection = self.environment().faces().iter().any(|face| {
                let dist = parry3d::query::distance(
                    &nalgebra::Isometry::identity(),
                    &face.to_convex_polyhedron(),
                    &nalgebra::Isometry::identity(),
                    &camera_movement_ray.to_segment(),
                )
                .unwrap();
                dist < 0.1
            });
            if !has_intersection {
                self.camera_position = new_camera_position;
            }
        }
        let accel = 15.0;
        let decel = 0.15;
        let forwards = self.camera_direction.into_inner();
        let right = forwards.cross(&up);
        if input_w {
            self.camera_velocity += accel * dt * forwards;
        } else if input_s {
            self.camera_velocity -= accel * dt * forwards;
        } else {
            self.camera_velocity -= decel * (self.camera_velocity.dot(&forwards)) * forwards;
        }
        if input_a {
            self.camera_velocity += accel * dt * right;
        } else if input_d {
            self.camera_velocity -= accel * dt * right;
        } else {
            self.camera_velocity -= decel * (self.camera_velocity.dot(&right)) * right;
        }
        // There was a bug where when turning the camera it could continue drifting
        // Since the third component of the "camera directions" didn't have code to handle deceleration
        // So I added this
        let camera_up = forwards.cross(&right);
        self.camera_velocity -= decel * (self.camera_velocity.dot(&camera_up)) * camera_up;
    }

    #[inline]
    fn environment(&self) -> &dyn Environment {
        match self.current_environment {
            EnvironmentIdentifier::Tunnel(id) => &self.maze.tunnels()[id],
            EnvironmentIdentifier::Landing(id) => &self.maze.landings()[id],
        }
    }

    pub fn world_to_camera(&self) -> TransformMatrix {
        self.world_to_camera_without_camera_translation()
            .times(&TransformMatrix(Matrix4::new_translation(&Vector3::new(
                -self.camera_position.x,
                -self.camera_position.y,
                -self.camera_position.z,
            ))))
    }

    pub fn world_to_camera_without_camera_translation(&self) -> TransformMatrix {
        TransformMatrix(
            // Scale everything in the z direction down
            // (does not affect the positions of any vertices)
            // This just reduces the scope of z values to reduce clipping
            Scale3::new(1.0, 1.0, 0.01).to_homogeneous()
                * Matrix4::new_perspective(self.aspect_ratio, 30.0, 0.01, 100.0)
                * Matrix4::look_at_rh(
                    &Point3::origin(),
                    &Point3::new(
                        self.camera_direction.x,
                        self.camera_direction.y,
                        self.camera_direction.z,
                    ),
                    &self.up(),
                ),
        )
    }

    #[wasm_bindgen(getter)]
    pub fn aspect_ratio(&self) -> f64 {
        self.aspect_ratio
    }

    #[wasm_bindgen(setter)]
    pub fn set_aspect_ratio(&mut self, aspect_ratio: f64) {
        self.aspect_ratio = aspect_ratio;
    }

    #[wasm_bindgen]
    pub fn points_to_float32array(&self) -> Vec<f32> {
        self.maze.points_to_float32array()
    }

    #[wasm_bindgen]
    pub fn normals_to_float32array(&self) -> Vec<f32> {
        self.maze.normals_to_float32array()
    }
}

impl Default for GameState {
    #[inline]
    fn default() -> Self {
        let maze = Maze::generate();
        let starting_landing = maze.landings()[0].landing();
        Self {
            aspect_ratio: 1.0,
            camera_position: starting_landing.point,
            camera_direction: UnitVector3::new_normalize(vector![0.0, 0.0, -1.0]),
            camera_velocity: Vector3::zeros(),
            current_environment: EnvironmentIdentifier::Landing(starting_landing.id),
            maze,
        }
    }
}

#[wasm_bindgen]
pub fn generate_skybox_points() -> Vec<f32> {
    let faces = {
        let scale = 100.0;
        let front_right_bottom = Point3::new(0.5, -0.5, 0.5) * scale;
        let front_left_bottom = Point3::new(-0.5, -0.5, 0.5) * scale;
        let front_right_top = Point3::new(0.5, 0.5, 0.5) * scale;
        let front_left_top = Point3::new(-0.5, 0.5, 0.5) * scale;

        let back_right_bottom = Point3::new(0.5, -0.5, -0.5) * scale;
        let back_left_bottom = Point3::new(-0.5, -0.5, -0.5) * scale;
        let back_right_top = Point3::new(0.5, 0.5, -0.5) * scale;
        let back_left_top = Point3::new(-0.5, 0.5, -0.5) * scale;

        vec![
            Face::new(vec![
                front_right_top,
                front_right_bottom,
                front_left_bottom,
                front_left_top,
            ]),
            Face::new(vec![
                front_right_top,
                back_right_top,
                back_right_bottom,
                front_right_bottom,
            ]),
            Face::new(vec![
                front_left_top,
                front_left_bottom,
                back_left_bottom,
                back_left_top,
            ]),
            Face::new(vec![
                front_right_top,
                front_left_top,
                back_left_top,
                back_right_top,
            ]),
            Face::new(vec![
                front_right_bottom,
                back_right_bottom,
                back_left_bottom,
                front_left_bottom,
            ]),
            Face::new(vec![
                back_right_top,
                back_left_top,
                back_left_bottom,
                back_right_bottom,
            ]),
        ]
    };

    let points: Vec<_> = faces
        .iter()
        .flat_map(|face| face.break_into_triangles())
        .collect();

    points
        .iter()
        .flat_map(|point| [point.x as _, point.y as _, point.z as _, 1.0])
        .collect()
}
