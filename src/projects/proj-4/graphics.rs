extern crate console_error_panic_hook;
pub(crate) extern crate nalgebra;
pub(crate) extern crate num_traits;
pub(crate) extern crate parry2d_f64 as parry2d;
pub(crate) extern crate parry3d_f64 as parry3d;
pub(crate) extern crate rand;
pub(crate) extern crate rand_chacha;
pub(crate) extern crate wasm_bindgen;
mod bounding_box;
mod face;
mod load_obj;
mod ray;

use std::f64::consts::PI;

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

#[derive(Clone)]
#[wasm_bindgen]
pub struct GameObject {
    parent_index: usize,
    faces: Vec<Face<f64>>,
    obj_vert_buffer: Option<JsValue>,
    obj_normals_buffer: Option<JsValue>,
    obj_uvs_buffer: Option<JsValue>,
    num_points: usize,
    initial_transform: Matrix4<f64>,
    dynamic_transform: Matrix4<f64>,
}

#[wasm_bindgen]
impl GameObject {
    #[wasm_bindgen(constructor)]
    #[allow(clippy::new_without_default)]
    pub fn new(obj_text: String, parent_index: usize, initial_transform: TransformMatrix) -> Self {
        let faces = load_obj::load_obj(&obj_text);

        let num_points = faces
            .iter()
            .fold(0, |count, face| count + (face.break_into_triangles().len()));

        Self {
            parent_index,
            faces,
            obj_vert_buffer: None,
            obj_normals_buffer: None,
            obj_uvs_buffer: None,
            num_points,
            initial_transform: initial_transform.0,
            dynamic_transform: Matrix4::identity(),
        }
    }

    #[wasm_bindgen(getter)]
    pub fn obj_vert_buffer(&self) -> JsValue {
        self.obj_vert_buffer.clone().unwrap_or(JsValue::UNDEFINED)
    }

    #[wasm_bindgen(setter)]
    pub fn set_obj_vert_buffer(&mut self, obj_vert_buffer: JsValue) {
        self.obj_vert_buffer = Some(obj_vert_buffer);
    }

    #[wasm_bindgen(getter)]
    pub fn obj_normals_buffer(&self) -> JsValue {
        self.obj_normals_buffer
            .clone()
            .unwrap_or(JsValue::UNDEFINED)
    }

    #[wasm_bindgen(setter)]
    pub fn set_obj_normals_buffer(&mut self, obj_normals_buffer: JsValue) {
        self.obj_normals_buffer = Some(obj_normals_buffer);
    }

    #[wasm_bindgen(getter)]
    pub fn obj_uvs_buffer(&self) -> JsValue {
        self.obj_uvs_buffer.clone().unwrap_or(JsValue::UNDEFINED)
    }

    #[wasm_bindgen(setter)]
    pub fn set_obj_uvs_buffer(&mut self, obj_uvs_buffer: JsValue) {
        self.obj_uvs_buffer = Some(obj_uvs_buffer);
    }

    pub fn points_to_float32array(&self) -> Vec<f32> {
        points_to_float32array(
            &self
                .faces
                .iter()
                .flat_map(|face| {
                    face.break_into_triangles()
                        .iter()
                        .map(|p| p.coords)
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>(),
        )
    }

    pub fn normals_to_float32array(&self) -> Vec<f32> {
        points_to_float32array(
            &self
                .faces
                .iter()
                .flat_map(|face| {
                    face.break_into_triangles()
                        .into_iter()
                        .map(move |_triangle| face.normal().into_inner())
                })
                .collect::<Vec<_>>(),
        )
    }

    pub fn uvs_to_float32array(&self) -> Vec<f32> {
        self.faces
            .iter()
            .flat_map(|face| {
                face.break_into_uv_triangles()
                    .iter()
                    .flat_map(|triangle| [triangle.x as f32, triangle.y as f32])
                    .collect::<Vec<_>>()
            })
            .collect()
    }

    fn get_transform_matrix(&self, parent_transform: &Matrix4<f64>) -> Matrix4<f64> {
        parent_transform * self.initial_transform * self.dynamic_transform
    }
}

#[wasm_bindgen]
pub struct GameState {
    camera_position: Point3<f64>,
    camera_direction: UnitVector3<f64>,
    camera_velocity: Vector3<f64>,
    aspect_ratio: f64,
    game_objects: Vec<GameObject>,
    target: Point3<f64>,
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
#[derive(Clone)]
pub struct ObjectRenderSnapshot {
    obj_vert_buffer: JsValue,
    obj_normals_buffer: JsValue,
    obj_uvs_buffer: JsValue,
    pub num_points: usize,
    pub transform: TransformMatrix,
}

#[wasm_bindgen]
impl ObjectRenderSnapshot {
    pub fn get_obj_vert_buffer(&self) -> JsValue {
        self.obj_vert_buffer.clone()
    }
    pub fn get_obj_normals_buffer(&self) -> JsValue {
        self.obj_normals_buffer.clone()
    }
    pub fn get_obj_uvs_buffer(&self) -> JsValue {
        self.obj_uvs_buffer.clone()
    }
}

#[wasm_bindgen]
pub struct RenderSnapshot {
    objects: Vec<ObjectRenderSnapshot>,
}

#[wasm_bindgen]
impl RenderSnapshot {
    pub fn object_ids(&self) -> Vec<usize> {
        self.objects.iter().enumerate().map(|(i, _)| i).collect()
    }
    pub fn get_object(&self, object_index: usize) -> ObjectRenderSnapshot {
        self.objects[object_index].clone()
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
            camera_position: point![3.0, 0.0, 0.0],
            camera_direction: UnitVector3::new_normalize(vector![-1.0, 0.0, 0.0]),
            camera_velocity: Vector3::zeros(),
            game_objects: vec![],
            target: Point3::origin(),
        }
    }

    fn up(&self) -> UnitVector3<f64> {
        Unit::new_normalize(vector![0.0, 1.0, 0.0])
    }

    #[wasm_bindgen]
    pub fn get_render_snapshot(&self) -> RenderSnapshot {
        let mut snapshot_objects: Vec<ObjectRenderSnapshot> = self
            .game_objects
            .iter()
            .map(|game_object| ObjectRenderSnapshot {
                obj_vert_buffer: game_object.obj_vert_buffer(),
                obj_normals_buffer: game_object.obj_normals_buffer(),
                obj_uvs_buffer: game_object.obj_uvs_buffer(),
                num_points: game_object.num_points,
                // Just a placeholder until we go back and accumulate the parent transforms below
                transform: TransformMatrix(game_object.initial_transform),
            })
            .collect();

        for (i, game_object) in self.game_objects.iter().enumerate() {
            let parent_index = game_object.parent_index;
            assert!(parent_index <= i);
            // Since we are iterating through in order,
            // we will have already calculated the parent's transform,
            // and can pass it into get_transform_matrix

            // If parent index is itself, that means it has no parent,
            // so we'll pass the identity matrix as the parent.
            let identity = Matrix4::identity();
            snapshot_objects[i].transform =
                TransformMatrix(game_object.get_transform_matrix(if parent_index == i {
                    &identity
                } else {
                    &snapshot_objects[game_object.parent_index].transform.0
                }))
        }

        RenderSnapshot {
            objects: snapshot_objects,
        }
    }

    #[wasm_bindgen]
    pub fn add_game_object(&mut self, game_object: GameObject) {
        self.game_objects.push(game_object);
    }

    #[wasm_bindgen]
    pub fn update_target_x(&mut self, x: f64) {
        self.target.x = x;
        self.update_inverse_kinematics();
    }

    #[wasm_bindgen]
    pub fn update_target_y(&mut self, y: f64) {
        self.target.y = y;
        self.update_inverse_kinematics();
    }

    #[wasm_bindgen]
    pub fn update_target_z(&mut self, z: f64) {
        self.target.z = z;
        self.update_inverse_kinematics();
    }

    #[inline]
    fn update_inverse_kinematics(&mut self) {
        // Target icosahedron
        self.game_objects[0].dynamic_transform = Translation3::from(self.target).to_homogeneous();

        // Onshape exports the OBJ in meters,
        // this scales to inches to match how my model is defined in onshape
        const INCHES: f64 = 0.0254;

        const ARM_1_LENGTH: f64 = 72.0 * INCHES;
        const ARM_2_LENGTH: f64 = 48.0 * INCHES;
        const ARM_1_START_HEIGHT: f64 = 10.0 * INCHES;

        // Turn the shoulder joint to face the target position
        self.game_objects[2].dynamic_transform = Rotation3::from_euler_angles(
            0.0,
            0.0,
            PI / 2.0 + f64::atan2(self.target.z, self.target.x),
        )
        .to_homogeneous();

        let delta_y = self.target.y + ARM_1_START_HEIGHT;
        let dist_to_target =
            (self.target.x.powi(2) + delta_y.powi(2) + self.target.z.powi(2)).sqrt();
        let dist_in_xz_plane = (self.target.x.powi(2) + self.target.z.powi(2)).sqrt();

        let n = (-ARM_1_LENGTH.powi(2) + ARM_2_LENGTH.powi(2) + dist_to_target.powi(2))
            / (2.0 * dist_to_target);
        let m = dist_to_target - n;
        let h = (ARM_2_LENGTH.powi(2) - n.powi(2)).sqrt();
        let c = (n / h).atan();
        let d = (m / h).atan();
        let arm_2_angle = c + d;
        let arm_1_angle = (h / m).atan() - (delta_y / dist_in_xz_plane).atan();

        if !arm_1_angle.is_nan() {
            // Arm 1 joint
            self.game_objects[3].dynamic_transform =
                Rotation3::from_euler_angles(0.0, 0.0, -arm_1_angle).to_homogeneous();
        }
        if !arm_2_angle.is_nan() {
            // Arm 2 joint
            self.game_objects[4].dynamic_transform =
                Rotation3::from_euler_angles(0.0, 0.0, arm_2_angle).to_homogeneous();
        }
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
