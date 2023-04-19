use nalgebra::{point, vector, Point3, Unit, UnitVector3, Vector3};
use wasm_bindgen::prelude::wasm_bindgen;

use crate::face::Face;

const TUNNEL_WIDTH: f64 = 2.0;
const TUNNEL_HEIGHT: f64 = 4.0;
const LANDING_RADIUS: f64 = 2.0;

#[derive(Clone)]
struct Landing {
    up: UnitVector3<f64>,
    point: Point3<f64>,
    tunnel_ids: Vec<usize>,
}
impl Landing {
    fn new(point: Point3<f64>, up: UnitVector3<f64>) -> Self {
        Self {
            point,
            up,
            tunnel_ids: vec![],
        }
    }
}

#[derive(Clone)]
struct Tunnel {
    start_landing_id: usize,
    end_landing_id: usize,
}

impl Tunnel {
    fn faces(&self, maze: &Maze) -> Vec<Face<f64>> {
        let mut faces = vec![];
        let start_landing = &maze.landings[self.start_landing_id];
        let end_landing = &maze.landings[self.end_landing_id];
        let tunnel_vec = end_landing.point.coords - start_landing.point.coords;
        // Dot products should be zero,
        // showing that the "direction" of the tunnel should be perpendicular
        // to the "up"s of the start and end landings
        assert!(start_landing.up.dot(&tunnel_vec) < f64::EPSILON);
        assert!(end_landing.up.dot(&tunnel_vec) < f64::EPSILON);

        let tunnel_dir = Unit::new_normalize(tunnel_vec);
        let start_point = start_landing.point + (tunnel_dir.into_inner() * LANDING_RADIUS);
        let end_point = end_landing.point - (tunnel_dir.into_inner() * LANDING_RADIUS);
        let inner_tunnel_vec = end_point - start_point;

        let up = start_landing.up.into_inner();
        let right = -up.cross(&tunnel_dir);
        let top_right_start = start_point + up * TUNNEL_HEIGHT / 2.0 + right * TUNNEL_WIDTH / 2.0;
        let top_right_end = end_point + up * TUNNEL_HEIGHT / 2.0 + right * TUNNEL_WIDTH / 2.0;
        let bottom_right_start =
            start_point - up * TUNNEL_HEIGHT / 2.0 + right * TUNNEL_WIDTH / 2.0;
        let bottom_right_end = end_point - up * TUNNEL_HEIGHT / 2.0 + right * TUNNEL_WIDTH / 2.0;
        let bottom_left_start = start_point - up * TUNNEL_HEIGHT / 2.0 - right * TUNNEL_WIDTH / 2.0;
        let bottom_left_end = end_point - up * TUNNEL_HEIGHT / 2.0 - right * TUNNEL_WIDTH / 2.0;
        let top_left_start = start_point + up * TUNNEL_HEIGHT / 2.0 - right * TUNNEL_WIDTH / 2.0;
        let top_left_end = end_point + up * TUNNEL_HEIGHT / 2.0 - right * TUNNEL_WIDTH / 2.0;
        faces.extend_from_slice(&[
            Face::new(vec![
                top_right_start,
                top_right_end,
                bottom_right_end,
                bottom_right_start,
            ]),
            Face::new(vec![
                top_left_start,
                top_left_end,
                top_right_end,
                top_right_start,
            ]),
            Face::new(vec![
                bottom_left_start,
                bottom_left_end,
                top_left_end,
                top_left_start,
            ]),
            Face::new(vec![
                bottom_right_start,
                bottom_right_end,
                bottom_left_end,
                bottom_left_start,
            ]),
        ]);
        faces
    }
}

#[wasm_bindgen]
#[derive(Clone)]
pub struct Maze {
    tunnels: Vec<Tunnel>,
    landings: Vec<Landing>,
}

fn points_to_float32array(points: &[Vector3<f64>]) -> Vec<f32> {
    points
        .iter()
        .flat_map(|point| [point.x as _, point.y as _, point.z as _, 1.0])
        .collect()
}

#[wasm_bindgen]
impl Maze {
    pub fn generate() -> Self {
        let mut m = Self {
            tunnels: vec![],
            landings: vec![],
        };
        let landing_0 = Landing::new(
            point![0.0, 0.0, 0.0],
            Unit::new_normalize(vector![0.0, 1.0, 0.0]),
        );
        m.landings.push(landing_0);
        let landing_1 = Landing::new(
            point![20.0, 0.0, 0.0],
            Unit::new_normalize(vector![0.0, 1.0, 0.0]),
        );
        m.landings.push(landing_1);
        let landing_2 = Landing::new(
            point![0.0, 0.0, 20.0],
            Unit::new_normalize(vector![0.0, 1.0, 0.0]),
        );
        m.landings.push(landing_2);
        m.tunnels.push(Tunnel {
            start_landing_id: 0,
            end_landing_id: 1,
        });
        m.tunnels.push(Tunnel {
            start_landing_id: 0,
            end_landing_id: 2,
        });
        m.tunnels.push(Tunnel {
            start_landing_id: 1,
            end_landing_id: 2,
        });
        m
    }
    #[wasm_bindgen]
    pub fn points_to_float32array(&self) -> Vec<f32> {
        points_to_float32array(
            &self
                .faces()
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

    #[wasm_bindgen]
    pub fn normals_to_float32array(&self) -> Vec<f32> {
        points_to_float32array(
            &self
                .faces()
                .iter()
                .flat_map(|face| {
                    face.break_into_triangles()
                        .into_iter()
                        .map(move |_triangle| face.normal().into_inner())
                })
                .collect::<Vec<_>>(),
        )
    }
    #[inline]
    pub(crate) fn faces(&self) -> Vec<Face<f64>> {
        self.tunnels
            .iter()
            .flat_map(|tunnel| tunnel.faces(self))
            .collect()
    }
}
