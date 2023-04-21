use nalgebra::{point, vector, Point3, Unit, UnitVector3, Vector3};
use wasm_bindgen::prelude::wasm_bindgen;

use crate::face::Face;

const TUNNEL_WIDTH: f64 = 3.0;
const TUNNEL_HEIGHT: f64 = 4.0;
const LANDING_RADIUS: f64 = 2.0;
const TUNNEL_SUBDIVISIONS: usize = 20;

/// The kinds of thing you can be "in" - like a room.
/// The usize represents the corresponding id.
#[derive(Debug, Copy, Clone)]
pub(crate) enum EnvironmentIdentifier {
    Tunnel(usize),
    Landing(usize),
}

pub(crate) trait Environment {
    fn faces(&self) -> &[Face<f64>];
    /// Faces (not displayed) that when passed through,
    /// trigger a "handoff" of player control into the next environment
    fn exit_faces(&self) -> &[(EnvironmentIdentifier, Face<f64>)];
}

pub(crate) struct LandingEnvironment {
    faces: Vec<Face<f64>>,
    exit_faces: Vec<(EnvironmentIdentifier, Face<f64>)>,
    landing: Landing,
}

impl LandingEnvironment {
    pub(crate) fn landing(&self) -> &Landing {
        &self.landing
    }
}
impl Environment for LandingEnvironment {
    fn faces(&self) -> &[Face<f64>] {
        &self.faces
    }
    fn exit_faces(&self) -> &[(EnvironmentIdentifier, Face<f64>)] {
        &self.exit_faces
    }
}

#[derive(Clone)]
pub(crate) struct Landing {
    pub(crate) id: usize,
    pub(crate) up: UnitVector3<f64>,
    pub(crate) point: Point3<f64>,
    tunnel_ids: Vec<usize>,
}

impl Landing {
    fn new(id: usize, point: Point3<f64>, up: UnitVector3<f64>) -> Self {
        Self {
            id,
            point,
            up,
            tunnel_ids: vec![],
        }
    }
    fn to_environment(&self, maze: &MazeDescriptor) -> LandingEnvironment {
        let mut exit_faces: Vec<(EnvironmentIdentifier, Face<f64>)> = vec![];

        let floor_center = self.point - self.up.into_inner() * TUNNEL_HEIGHT / 2.0;
        let floor_to_ceiling = self.up.into_inner() * TUNNEL_HEIGHT;
        // TODO: probably need to sort the tunnels here?
        let floor = Face::new(
            self.tunnel_ids
                .iter()
                .flat_map(|&tunnel_id| {
                    let tunnel = &maze.tunnels[tunnel_id];
                    let towards_tunnel = Unit::new_normalize(
                        if self.id == maze.landings[tunnel.start_landing_id].id {
                            // self is start landing
                            maze.landings[tunnel.end_landing_id].point - self.point
                        } else {
                            // self is end landing
                            maze.landings[tunnel.start_landing_id].point - self.point
                        },
                    )
                    .into_inner();
                    let right = towards_tunnel.cross(&self.up);
                    let door_bottom_left =
                        floor_center + TUNNEL_WIDTH / 2.0 * right + LANDING_RADIUS * towards_tunnel;
                    let door_bottom_right =
                        floor_center - TUNNEL_WIDTH / 2.0 * right + LANDING_RADIUS * towards_tunnel;
                    exit_faces.push((
                        EnvironmentIdentifier::Tunnel(tunnel_id),
                        Face::new(vec![
                            door_bottom_left,
                            door_bottom_right,
                            door_bottom_right + floor_to_ceiling,
                            door_bottom_left + floor_to_ceiling,
                        ]),
                    ));
                    [door_bottom_left, door_bottom_right]
                })
                .chain(if self.tunnel_ids.len() >= 2 {
                    vec![]
                } else {
                    vec![self.point - self.up.into_inner() * TUNNEL_HEIGHT / 2.0]
                })
                .collect(),
        );
        let ceiling = Face::new(
            floor
                .points()
                .iter()
                .map(|point| point + floor_to_ceiling)
                .collect(),
        );
        let faces = vec![floor, ceiling];

        LandingEnvironment {
            faces,
            exit_faces,
            landing: self.clone(),
        }
    }
}

pub(crate) struct TunnelEnvironment {
    faces: Vec<Face<f64>>,
    start_face: Face<f64>,
    end_face: Face<f64>,
    tunnel: Tunnel,
    exit_faces: Vec<(EnvironmentIdentifier, Face<f64>)>,
}

impl TunnelEnvironment {
    pub(crate) fn tunnel(&self) -> &Tunnel {
        &self.tunnel
    }
}
impl Environment for TunnelEnvironment {
    fn faces(&self) -> &[Face<f64>] {
        &self.faces
    }
    fn exit_faces(&self) -> &[(EnvironmentIdentifier, Face<f64>)] {
        &self.exit_faces
    }
}

#[derive(Clone)]
pub(crate) struct Tunnel {
    start_landing_id: usize,
    end_landing_id: usize,
}

impl Tunnel {
    fn to_environment(&self, maze: &MazeDescriptor) -> TunnelEnvironment {
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

        type P = Point3<f64>;

        // This will hold "twisting frames",
        // rectangles that are twisted from the start to the end.
        let frames: Vec<(P, P, P, P)> = (0..=TUNNEL_SUBDIVISIONS)
            .map(|subdivision_i| {
                let percent = (subdivision_i as f64) / (TUNNEL_SUBDIVISIONS as f64);
                let up = start_landing
                    .up
                    .slerp(&end_landing.up, percent)
                    .into_inner();
                let right = -up.cross(&tunnel_dir);
                let frame_center = Point3::from(percent * inner_tunnel_vec + start_point.coords);
                let top_right =
                    frame_center + up * TUNNEL_HEIGHT / 2.0 + right * TUNNEL_WIDTH / 2.0;
                let bottom_right =
                    frame_center - up * TUNNEL_HEIGHT / 2.0 + right * TUNNEL_WIDTH / 2.0;
                let bottom_left =
                    frame_center - up * TUNNEL_HEIGHT / 2.0 - right * TUNNEL_WIDTH / 2.0;
                let top_left = frame_center + up * TUNNEL_HEIGHT / 2.0 - right * TUNNEL_WIDTH / 2.0;
                (top_right, bottom_right, bottom_left, top_left)
            })
            .collect();

        let faces = frames
            .windows(2)
            .flat_map(|frames| {
                let (front_top_right, front_bottom_right, front_bottom_left, front_top_left) =
                    frames[0];
                let (back_top_right, back_bottom_right, back_bottom_left, back_top_left) =
                    frames[1];

                [
                    // Right
                    Face::new(vec![front_top_right, back_top_right, back_bottom_right]),
                    Face::new(vec![back_bottom_right, front_bottom_right, front_top_right]),
                    // Bottom
                    Face::new(vec![
                        front_bottom_right,
                        back_bottom_right,
                        back_bottom_left,
                    ]),
                    Face::new(vec![
                        back_bottom_left,
                        front_bottom_left,
                        front_bottom_right,
                    ]),
                    // Left
                    Face::new(vec![front_bottom_left, back_bottom_left, back_top_left]),
                    Face::new(vec![back_top_left, front_top_left, front_bottom_left]),
                    // Top
                    Face::new(vec![front_top_left, back_top_left, back_top_right]),
                    Face::new(vec![back_top_right, front_top_right, front_top_left]),
                ]
            })
            .collect();

        let start_frame = frames.first().unwrap();
        let end_frame = frames.last().unwrap();

        let start_face = Face::new(vec![
            start_frame.0,
            start_frame.1,
            start_frame.2,
            start_frame.3,
        ]);
        let end_face = Face::new(vec![end_frame.0, end_frame.1, end_frame.2, end_frame.3]);

        let exit_faces = vec![
            (
                EnvironmentIdentifier::Landing(self.start_landing_id),
                start_face.clone(),
            ),
            (
                EnvironmentIdentifier::Landing(self.end_landing_id),
                end_face.clone(),
            ),
        ];

        TunnelEnvironment {
            faces,
            start_face,
            end_face,
            tunnel: self.clone(),
            exit_faces,
        }
    }
}

/// The "abstract" representation of a maze -- all the features are represented as single points
/// This is in contrast with the Maze struct, which is much higher-fidelity and includes faces.
struct MazeDescriptor {
    landings: Vec<Landing>,
    tunnels: Vec<Tunnel>,
}
impl MazeDescriptor {
    fn add_landing(&mut self, location: Point3<f64>, up: UnitVector3<f64>) -> usize {
        let id = self.landings.len();
        let landing = Landing::new(id, location, up);
        self.landings.push(landing);
        id
    }
    fn add_tunnel(&mut self, start_landing_id: usize, end_landing_id: usize) -> usize {
        let new_tunnel_id = self.tunnels.len();
        self.tunnels.push(Tunnel {
            start_landing_id,
            end_landing_id,
        });
        self.landings[start_landing_id]
            .tunnel_ids
            .push(new_tunnel_id);
        self.landings[end_landing_id].tunnel_ids.push(new_tunnel_id);
        new_tunnel_id
    }
}

#[wasm_bindgen]
pub struct Maze {
    landings: Vec<LandingEnvironment>,
    tunnels: Vec<TunnelEnvironment>,
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
        let mut maze = MazeDescriptor {
            landings: vec![],
            tunnels: vec![],
        };
        let landing_0 = maze.add_landing(
            point![0.0, 0.0, 0.0],
            Unit::new_normalize(vector![0.0, 1.0, 0.0]),
        );
        let landing_1 = maze.add_landing(
            point![20.0, 0.0, 0.0],
            Unit::new_normalize(vector![0.0, 1.0, 1.0]),
        );
        let landing_2 = maze.add_landing(
            point![0.0, 0.0, 20.0],
            Unit::new_normalize(vector![1.0, 1.0, 0.0]),
        );
        maze.add_tunnel(landing_0, landing_1);
        maze.add_tunnel(landing_0, landing_2);

        Self {
            landings: maze
                .landings
                .iter()
                .map(|landing| landing.to_environment(&maze))
                .collect(),
            tunnels: maze
                .tunnels
                .iter()
                .map(|tunnel| tunnel.to_environment(&maze))
                .collect(),
        }
    }
    #[inline]
    pub(crate) fn tunnels(&self) -> &[TunnelEnvironment] {
        &self.tunnels
    }
    #[inline]
    pub(crate) fn landings(&self) -> &[LandingEnvironment] {
        &self.landings
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
            .flat_map(|tunnel| tunnel.faces())
            .chain(self.landings.iter().flat_map(|landing| landing.faces()))
            .cloned()
            .collect()
    }
}
