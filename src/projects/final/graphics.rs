extern crate console_error_panic_hook;
pub(crate) extern crate nalgebra;
pub(crate) extern crate num_traits;
pub(crate) extern crate parry2d_f64 as parry2d;
pub(crate) extern crate parry3d_f64 as parry3d;
pub(crate) extern crate rand;
pub(crate) extern crate rand_chacha;
pub(crate) extern crate wasm_bindgen;
mod face;

use face::Face;
use nalgebra::{
    point, vector, Matrix4, Point3, Rotation3, Scale3, Translation3, Unit, UnitQuaternion,
    UnitVector3, Vector3,
};
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

fn points_to_float32array(points: &[Vector3<f64>]) -> Vec<f32> {
    points
        .iter()
        .flat_map(|point| [point.x as _, point.y as _, point.z as _, 1.0])
        .collect()
}

#[wasm_bindgen]
pub struct GameState {
    camera_position: Point3<f64>,
    camera_direction: UnitVector3<f64>,
    camera_velocity: Vector3<f64>,
    aspect_ratio: f64,
}

/// A little wrapper around the nalgebra matrix4 class, for JS use,
/// so we only have to generate binding code for the methods we actually need.
#[wasm_bindgen]
#[derive(Clone, Copy)]
pub struct TransformMatrix(Matrix4<f64>);

#[wasm_bindgen]
impl TransformMatrix {
    pub fn translation(x: f64, y: f64, z: f64) -> Self {
        Self(Translation3::new(x, y, z).to_homogeneous())
    }
    pub fn rotation_euler(roll: f64, pitch: f64, yaw: f64) -> Self {
        Self(Rotation3::from_euler_angles(roll, pitch, yaw).to_homogeneous())
    }
    pub fn identity() -> Self {
        Self(Matrix4::identity())
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
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        console_error_panic_hook::set_once();
        Self {
            aspect_ratio: 1.0,
            camera_position: point![0.0, -10.0, 0.0],
            camera_direction: UnitVector3::new_normalize(vector![-1.0, 0.0, 0.0]),
            camera_velocity: Vector3::zeros(),
        }
    }

    fn up(&self) -> UnitVector3<f64> {
        Unit::new_normalize(vector![0.0, 1.0, 0.0])
    }

    // It would be a good idea to have this accept the arguments as a struct,
    // but then I'd have to deal with JS binding generation for that struct.
    #[allow(clippy::too_many_arguments)]
    pub fn update(
        &mut self,
        is_active: bool,
        input_w: bool,
        input_a: bool,
        input_s: bool,
        input_d: bool,
        cursor_movement_x: f64,
        cursor_movement_y: f64,
        delta_time_ms: usize,
    ) {
        if !is_active {
            // Reset the velocity so when the frame becomes active again it doesn't jump
            self.camera_velocity = Vector3::zeros();
            // Don't update the position if not active
            return;
        }
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
        self.camera_position += self.camera_velocity * dt;
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

    pub fn camera_position(&self) -> Vec<f32> {
        vec![
            self.camera_position.x as _,
            self.camera_position.y as _,
            self.camera_position.z as _,
        ]
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

#[wasm_bindgen]
pub struct ColoredMesh {
    points: Vec<f32>,
    colors: Vec<f32>,
}

#[wasm_bindgen]
impl ColoredMesh {
    #[inline]
    pub fn points(&self) -> Vec<f32> {
        self.points.clone()
    }
    #[inline]
    pub fn colors(&self) -> Vec<f32> {
        self.colors.clone()
    }
}

#[wasm_bindgen]
pub fn layer_to_mesh(layer: &[u8]) -> ColoredMesh {
    const NUM_CHANNELS: usize = 4; // R, G, B, A
    let dimension = ((layer.len() / NUM_CHANNELS) as f64).sqrt() as usize;
    assert_eq!(dimension * dimension * NUM_CHANNELS, layer.len());
    type Color = Vector3<f64>;
    let mut faces_with_colors: Vec<(Face<f64>, Color)> = vec![];

    let scale = 100.0 / (dimension as f64);
    const LAYER_HEIGHT: f64 = 1.0;
    let front_right_bottom = Point3::new(0.5, -0.5 * LAYER_HEIGHT, 0.5);
    let front_left_bottom = Point3::new(-0.5, -0.5 * LAYER_HEIGHT, 0.5);
    let front_right_top = Point3::new(0.5, 0.5 * LAYER_HEIGHT, 0.5);
    let front_left_top = Point3::new(-0.5, 0.5 * LAYER_HEIGHT, 0.5);

    let back_right_bottom = Point3::new(0.5, -0.5 * LAYER_HEIGHT, -0.5);
    let back_left_bottom = Point3::new(-0.5, -0.5 * LAYER_HEIGHT, -0.5);
    let back_right_top = Point3::new(0.5, 0.5 * LAYER_HEIGHT, -0.5);
    let back_left_top = Point3::new(-0.5, 0.5 * LAYER_HEIGHT, -0.5);
    let centering_offset = (dimension as f64) / 2.0;

    let color_front_back = vector![0.0, 0.0, 0.5];
    let color_top_bottom = vector![0.0, 0.0, 0.5];
    let color_left_right = vector![0.0, 0.0, 0.5];

    const LAYER_LIMIT: u8 = 50;

    #[inline(always)]
    fn get_pixel(
        pixels_layers: &[Vec<bool>],
        dimension: usize,
        i: usize,
        layer: usize,
        offset_rows: isize,
        offset_cols: isize,
        offset_layers: isize,
    ) -> bool {
        let row = (i / dimension) as isize;
        let col = (i % dimension) as isize;
        let new_row = row + offset_rows;
        let new_col = col + offset_cols;
        let new_layer = layer as isize + offset_layers;

        if new_layer < 0 {
            return true;
        }
        if new_row < 0
            || new_col < 0
            || new_row >= dimension as isize
            || new_col >= dimension as isize
            || new_layer >= LAYER_LIMIT as isize
        {
            false
        } else {
            pixels_layers[new_layer as usize][dimension * new_row as usize + new_col as usize]
        }
    }
    let pixel_layers: Vec<Vec<bool>> = (0..LAYER_LIMIT)
        .map(|n| {
            layer
                .chunks_exact(4)
                .map(|pixel| {
                    let red = pixel[0];
                    red > 255 / LAYER_LIMIT * n
                })
                .collect()
        })
        .collect();
    for (n, pixel_layer) in pixel_layers.iter().enumerate() {
        for (i, &pixel_filled) in pixel_layer.iter().enumerate() {
            let row = i / dimension;
            let col = i % dimension;
            let offset = vector![
                row as f64 - centering_offset,
                -(n as f64) * LAYER_HEIGHT,
                col as f64 - centering_offset
            ];
            let layer_color = vector![n as f64, n as f64, n as f64] * 1.0 / (LAYER_LIMIT as f64);
            if pixel_filled {
                let front_right_bottom = (front_right_bottom + offset) * scale;
                let front_left_bottom = (front_left_bottom + offset) * scale;
                let front_right_top = (front_right_top + offset) * scale;
                let front_left_top = (front_left_top + offset) * scale;

                let back_right_bottom = (back_right_bottom + offset) * scale;
                let back_left_bottom = (back_left_bottom + offset) * scale;
                let back_right_top = (back_right_top + offset) * scale;
                let back_left_top = (back_left_top + offset) * scale;

                if !get_pixel(&pixel_layers, dimension, i, n, 0, 1, 0) {
                    let front_face = (
                        Face::new(vec![
                            front_right_top,
                            front_right_bottom,
                            front_left_bottom,
                            front_left_top,
                        ]),
                        color_front_back + layer_color,
                    );

                    faces_with_colors.push(front_face);
                }
                if !get_pixel(&pixel_layers, dimension, i, n, 0, -1, 0) {
                    let back_face = (
                        Face::new(vec![
                            back_right_top,
                            back_left_top,
                            back_left_bottom,
                            back_right_bottom,
                        ]),
                        color_front_back + layer_color,
                    );
                    faces_with_colors.push(back_face);
                }
                if !get_pixel(&pixel_layers, dimension, i, n, -1, 0, 0) {
                    let left_face = (
                        Face::new(vec![
                            front_left_top,
                            front_left_bottom,
                            back_left_bottom,
                            back_left_top,
                        ]),
                        color_left_right + layer_color,
                    );
                    faces_with_colors.push(left_face);
                }
                if !get_pixel(&pixel_layers, dimension, i, n, 1, 0, 0) {
                    let right_face = (
                        Face::new(vec![
                            front_right_top,
                            back_right_top,
                            back_right_bottom,
                            front_right_bottom,
                        ]),
                        color_left_right + layer_color,
                    );
                    faces_with_colors.push(right_face);
                }
                if !get_pixel(&pixel_layers, dimension, i, n, 0, 0, -1) {
                    let top_face = (
                        Face::new(vec![
                            front_right_top,
                            front_left_top,
                            back_left_top,
                            back_right_top,
                        ]),
                        color_top_bottom + layer_color,
                    );
                    faces_with_colors.push(top_face);
                }
                if !get_pixel(&pixel_layers, dimension, i, n, 0, 0, 1) {
                    let bottom_face = (
                        Face::new(vec![
                            front_right_bottom,
                            back_right_bottom,
                            back_left_bottom,
                            front_left_bottom,
                        ]),
                        color_top_bottom + layer_color,
                    );
                    faces_with_colors.push(bottom_face);
                }
            }
        }
    }
    let points = points_to_float32array(
        &faces_with_colors
            .iter()
            .flat_map(|(face, _color)| {
                face.break_into_triangles()
                    .iter()
                    .map(|p| p.coords)
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>(),
    );

    let colors = points_to_float32array(
        &faces_with_colors
            .iter()
            .flat_map(|(face, color)| {
                face.break_into_triangles()
                    .iter()
                    .map(|_| *color)
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>(),
    );

    ColoredMesh { points, colors }
}
