use nalgebra::{point, vector, Matrix4, Point3, Unit, UnitVector3, Vector2, Vector3, Vector4};
use wasm_bindgen::prelude::wasm_bindgen;

use rand::{distributions::Uniform, SeedableRng};
use rand::{Rng, RngCore};

use crate::console_log;
use crate::face::{Face, UVPair};

const TUNNEL_WIDTH: f64 = 3.0;
const TUNNEL_HEIGHT: f64 = 4.0;
const LANDING_RADIUS: f64 = 2.0;
const TUNNEL_SUBDIVISIONS: usize = 20;
const MIN_TUNNEL_LENGTH: f64 = 20.0;
const MAX_TUNNEL_LENGTH: f64 = 60.0;
const MIN_TWIST: f64 = -120.0;
const MAX_TWIST: f64 = 120.0;
const MAX_TRIES: usize = 100;

fn min_angle_between_tunnels() -> f64 {
    2.0 * f64::atan((TUNNEL_WIDTH / 2.0) / LANDING_RADIUS)
}

fn max_tunnel_nearness() -> f64 {
    (TUNNEL_WIDTH + TUNNEL_HEIGHT).sqrt()
}

struct Door {
    top_left: Point3<f64>,
    bottom_left: Point3<f64>,
    top_right: Point3<f64>,
    bottom_right: Point3<f64>,
}
impl Door {
    fn to_face(&self) -> Face<f64> {
        Face::new(vec![
            self.bottom_left,
            self.bottom_right,
            self.top_right,
            self.top_left,
        ])
    }
}
fn make_door(point: &Point3<f64>, up: &UnitVector3<f64>, forwards: &UnitVector3<f64>) -> Door {
    let up = up.into_inner();
    let right = forwards.cross(&up);
    let top_right = point + up * TUNNEL_HEIGHT / 2.0 + right * TUNNEL_WIDTH / 2.0;
    let bottom_right = point - up * TUNNEL_HEIGHT / 2.0 + right * TUNNEL_WIDTH / 2.0;
    let bottom_left = point - up * TUNNEL_HEIGHT / 2.0 - right * TUNNEL_WIDTH / 2.0;
    let top_left = point + up * TUNNEL_HEIGHT / 2.0 - right * TUNNEL_WIDTH / 2.0;
    Door {
        top_left,
        top_right,
        bottom_left,
        bottom_right,
    }
}

/// The kinds of thing you can be "in" - like a room.
/// The usize represents the corresponding id.
#[derive(Debug, Copy, Clone)]
pub(crate) enum EnvironmentIdentifier {
    Tunnel(usize),
    Landing(usize),
    DeadEnd(usize),
}

/// The usize represents the corresponding id.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(crate) enum ConnectorIdentifier {
    Landing(usize),
    DeadEnd(usize),
}

impl ConnectorIdentifier {
    fn environment_identifier(&self) -> EnvironmentIdentifier {
        match self {
            ConnectorIdentifier::Landing(id) => EnvironmentIdentifier::Landing(*id),
            ConnectorIdentifier::DeadEnd(id) => EnvironmentIdentifier::DeadEnd(*id),
        }
    }
}

struct Coupler {
    point: Point3<f64>,
    up: UnitVector3<f64>,
}

/// The kinds of things that can be attached to the ends of tunnels
trait Connector {
    fn point(&self) -> Point3<f64>;
    /// Where the tunnel should attach to
    /// (not necessarily `point()` which is the center of the object)
    /// other_tunnel_end is the `point()` of the other end of the tunnel
    /// (to be clear, *not* the coupling_point() of the other end of the tunnel)
    fn coupler(&self, _other_tunnel_end: &Point3<f64>) -> Coupler;
    fn add_tunnel(&mut self, tunnel_id: usize);
}

pub(crate) trait Environment {
    fn faces(&self) -> &[Face<f64>];
    /// Faces (not displayed) that when passed through,
    /// trigger a "handoff" of player control into the next environment
    fn exit_faces(&self) -> &[(EnvironmentIdentifier, Face<f64>)];
    fn up(&self, camera_position: Point3<f64>) -> UnitVector3<f64>;
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
    fn up(&self, _camera_position: Point3<f64>) -> UnitVector3<f64> {
        self.landing.up
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
        assert!(self.tunnel_ids.len() >= 2);
        let mut exit_faces: Vec<(EnvironmentIdentifier, Face<f64>)> = vec![];

        let floor_to_ceiling = self.up.into_inner() * TUNNEL_HEIGHT;
        let mut sorted_tunnels: Vec<_> = self
            .tunnel_ids
            .iter()
            .map(|&tunnel_id| {
                let tunnel = &maze.tunnels[tunnel_id];
                let towards_tunnel = Unit::new_normalize(
                    if ConnectorIdentifier::Landing(self.id) == tunnel.start_connector {
                        // self is start landing
                        maze.get_connector(tunnel.end_connector).point() - self.point
                    } else {
                        // self is end landing
                        maze.get_connector(tunnel.start_connector).point() - self.point
                    },
                );
                (tunnel_id, towards_tunnel)
            })
            .collect();

        // This is arbitrary, defined as the "zero angle" for the sake of comparison between tunnels
        // We need the tunnels to be in order so that when we display them
        // the entrances and walls don't criss-cross
        let forwards = sorted_tunnels[0].1;
        let right = self.up.cross(&forwards);
        sorted_tunnels.sort_by(
            |(_tunnel_id_1, towards_tunnel_1), (_tunnel_id_2, towards_tunnel_2)| {
                f64::atan2(
                    towards_tunnel_1.dot(&forwards),
                    towards_tunnel_1.dot(&right),
                )
                .partial_cmp(&f64::atan2(
                    towards_tunnel_2.dot(&forwards),
                    towards_tunnel_2.dot(&right),
                ))
                .unwrap()
            },
        );
        let min_angle_between_tunnels = min_angle_between_tunnels();

        for tunnel_pair in sorted_tunnels.windows(2) {
            // a dot b = |a| |b| cos(theta) => theta = ...
            let angle = f64::acos(
                tunnel_pair[0].1.dot(&tunnel_pair[1].1)
                    / (tunnel_pair[0].1.magnitude() * tunnel_pair[1].1.magnitude()),
            );
            assert!(angle > min_angle_between_tunnels);
        }

        let floor_points: Vec<Point3<f64>> = sorted_tunnels
            .into_iter()
            .flat_map(|(tunnel_id, towards_tunnel)| {
                let door = make_door(
                    &(self.point + towards_tunnel.into_inner() * LANDING_RADIUS),
                    &self.up,
                    &towards_tunnel,
                );
                exit_faces.push((EnvironmentIdentifier::Tunnel(tunnel_id), door.to_face()));
                [door.bottom_left, door.bottom_right]
            })
            .collect();
        // Fill in the spaces between the doors
        let walls: Vec<_> = floor_points[1..]
            .chunks_exact(2)
            .chain([[
                *floor_points.last().unwrap(),
                *floor_points.first().unwrap(),
            ]
            .as_slice()])
            .map(|floor_point_pair| {
                Face::new(vec![
                    floor_point_pair[0],
                    floor_point_pair[1],
                    floor_point_pair[1] + floor_to_ceiling,
                    floor_point_pair[0] + floor_to_ceiling,
                ])
            })
            .collect();
        let floor = Face::new(floor_points);
        let ceiling = Face::new(
            floor
                .points()
                .iter()
                .map(|point| point + floor_to_ceiling)
                .collect(),
        );
        let mut faces = vec![floor, ceiling];
        faces.extend_from_slice(&walls);

        LandingEnvironment {
            faces,
            exit_faces,
            landing: self.clone(),
        }
    }
}

impl Connector for Landing {
    fn point(&self) -> Point3<f64> {
        self.point
    }
    fn coupler(&self, other_tunnel_end: &Point3<f64>) -> Coupler {
        let tunnel_dir = (other_tunnel_end - self.point).normalize();
        let coupler_point = self.point + (tunnel_dir * LANDING_RADIUS);
        Coupler {
            point: coupler_point,
            up: self.up,
        }
    }
    fn add_tunnel(&mut self, tunnel_id: usize) {
        self.tunnel_ids.push(tunnel_id);
    }
}

pub(crate) struct TunnelEnvironment {
    faces: Vec<Face<f64>>,
    tunnel: Tunnel,
    exit_faces: Vec<(EnvironmentIdentifier, Face<f64>)>,
    start_coupler: Coupler,
    end_coupler: Coupler,
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
    fn up(&self, camera_position: Point3<f64>) -> UnitVector3<f64> {
        let start_to_camera = camera_position - self.start_coupler.point;
        let tunnel_length_vector = self.end_coupler.point - self.start_coupler.point;
        let percent =
            start_to_camera.dot(&tunnel_length_vector) / tunnel_length_vector.magnitude_squared();
        self.start_coupler.up.slerp(&self.end_coupler.up, percent)
    }
}

#[derive(Clone)]
pub(crate) struct Tunnel {
    start_connector: ConnectorIdentifier,
    end_connector: ConnectorIdentifier,
}

impl Tunnel {
    fn to_environment(&self, maze: &MazeDescriptor) -> TunnelEnvironment {
        let start_connector = &maze.get_connector(self.start_connector);
        let end_connector = &maze.get_connector(self.end_connector);
        let tunnel_vec = end_connector.point().coords - start_connector.point().coords;
        let start_coupler = start_connector.coupler(&end_connector.point());
        let end_coupler = end_connector.coupler(&start_connector.point());
        // Dot products should be zero,
        // showing that the "direction" of the tunnel should be perpendicular
        // to the "up"s of the start and end landings
        assert!(
            start_coupler.up.dot(&tunnel_vec).abs() < 1e-6,
            "start_coupler was not perpendicular to up vector, dot product was {}",
            start_coupler.up.dot(&tunnel_vec).abs()
        );
        assert!(
            end_coupler.up.dot(&tunnel_vec).abs() < 1e-6,
            "end_coupler was not perpendicular to up vector, dot product was {}",
            end_coupler.up.dot(&tunnel_vec).abs()
        );

        let inner_tunnel_vec = end_coupler.point - start_coupler.point;
        let tunnel_dir = (end_connector.point() - start_connector.point()).normalize();

        type P = UVPair<f64>;

        // This will hold "twisting frames",
        // rectangles that are twisted from the start to the end.
        // Why are there 5 points instead of 4? Read the comment about top_right_repeat
        let frames: Vec<(P, P, P, P, P)> = (0..=TUNNEL_SUBDIVISIONS)
            .map(|subdivision_i| {
                let percent = (subdivision_i as f64) / (TUNNEL_SUBDIVISIONS as f64);
                let up = start_coupler
                    .up
                    .slerp(&end_coupler.up, percent)
                    .into_inner();
                let right = -up.cross(&tunnel_dir);
                let frame_center = start_coupler.point + percent * inner_tunnel_vec;
                const UV_SCALE: f64 = 0.2;
                let uv_x = percent * UV_SCALE * inner_tunnel_vec.magnitude();
                let top_right = UVPair {
                    point: frame_center + up * TUNNEL_HEIGHT / 2.0 + right * TUNNEL_WIDTH / 2.0,
                    uv: Vector2::new(uv_x, 0.0 * UV_SCALE),
                };
                let bottom_right = UVPair {
                    point: frame_center - up * TUNNEL_HEIGHT / 2.0 + right * TUNNEL_WIDTH / 2.0,
                    uv: Vector2::new(uv_x, TUNNEL_HEIGHT * UV_SCALE),
                };
                let bottom_left = UVPair {
                    point: frame_center - up * TUNNEL_HEIGHT / 2.0 - right * TUNNEL_WIDTH / 2.0,
                    uv: Vector2::new(uv_x, (TUNNEL_WIDTH + TUNNEL_HEIGHT) * UV_SCALE),
                };
                let top_left = UVPair {
                    point: frame_center + up * TUNNEL_HEIGHT / 2.0 - right * TUNNEL_WIDTH / 2.0,
                    uv: Vector2::new(uv_x, (TUNNEL_WIDTH + 2.0 * TUNNEL_HEIGHT) * UV_SCALE),
                };
                // Why is this repeated?
                // Because we need a different UV point for when we "wrap back around".
                // So this point has the same point value but a different UV than top_right
                let top_right_repeat = UVPair {
                    point: frame_center + up * TUNNEL_HEIGHT / 2.0 + right * TUNNEL_WIDTH / 2.0,
                    uv: Vector2::new(uv_x, (2.0 * TUNNEL_WIDTH + 2.0 * TUNNEL_HEIGHT) * UV_SCALE),
                };
                (
                    top_right,
                    bottom_right,
                    bottom_left,
                    top_left,
                    top_right_repeat,
                )
            })
            .collect();

        let faces = frames
            .windows(2)
            .flat_map(|frames| {
                let (
                    front_top_right,
                    front_bottom_right,
                    front_bottom_left,
                    front_top_left,
                    front_top_right_repeat,
                ) = frames[0];
                let (
                    back_top_right,
                    back_bottom_right,
                    back_bottom_left,
                    back_top_left,
                    back_top_right_repeat,
                ) = frames[1];

                [
                    // Right
                    Face::from_uv_pairs(vec![front_top_right, back_top_right, back_bottom_right]),
                    Face::from_uv_pairs(vec![
                        back_bottom_right,
                        front_bottom_right,
                        front_top_right,
                    ]),
                    // Bottom
                    Face::from_uv_pairs(vec![
                        front_bottom_right,
                        back_bottom_right,
                        back_bottom_left,
                    ]),
                    Face::from_uv_pairs(vec![
                        back_bottom_left,
                        front_bottom_left,
                        front_bottom_right,
                    ]),
                    // Left
                    Face::from_uv_pairs(vec![front_bottom_left, back_bottom_left, back_top_left]),
                    Face::from_uv_pairs(vec![back_top_left, front_top_left, front_bottom_left]),
                    // Top
                    Face::from_uv_pairs(vec![front_top_left, back_top_left, back_top_right_repeat]),
                    Face::from_uv_pairs(vec![
                        back_top_right_repeat,
                        front_top_right_repeat,
                        front_top_left,
                    ]),
                ]
            })
            .collect();

        let start_frame = frames.first().unwrap();
        let end_frame = frames.last().unwrap();

        let start_face = Face::new(vec![
            start_frame.0.point,
            start_frame.1.point,
            start_frame.2.point,
            start_frame.3.point,
        ]);
        let end_face = Face::new(vec![
            end_frame.0.point,
            end_frame.1.point,
            end_frame.2.point,
            end_frame.3.point,
        ]);

        let exit_faces = vec![
            (self.start_connector.environment_identifier(), start_face),
            (self.end_connector.environment_identifier(), end_face),
        ];

        TunnelEnvironment {
            faces,
            tunnel: self.clone(),
            exit_faces,
            start_coupler,
            end_coupler,
        }
    }
}

#[derive(Clone)]
pub(crate) struct DeadEnd {
    pub(crate) id: usize,
    pub(crate) up: UnitVector3<f64>,
    pub(crate) point: Point3<f64>,
    tunnel_id: Option<usize>,
}

impl Connector for DeadEnd {
    fn point(&self) -> Point3<f64> {
        self.point
    }
    fn coupler(&self, _other_tunnel_end: &Point3<f64>) -> Coupler {
        Coupler {
            point: self.point,
            up: self.up,
        }
    }
    fn add_tunnel(&mut self, tunnel_id: usize) {
        assert!(self.tunnel_id.is_none());
        self.tunnel_id = Some(tunnel_id);
    }
}

impl DeadEnd {
    fn new(id: usize, point: Point3<f64>, up: UnitVector3<f64>) -> Self {
        Self {
            id,
            point,
            up,
            tunnel_id: None,
        }
    }
    fn to_environment(&self, maze: &MazeDescriptor) -> DeadEndEnvironment {
        assert!(self.tunnel_id.is_some());
        let tunnel_id = self.tunnel_id.unwrap();
        let tunnel = &maze.tunnels[tunnel_id];
        let door = make_door(
            &self.point,
            &self.up,
            &Unit::new_normalize(
                if tunnel.start_connector == ConnectorIdentifier::DeadEnd(self.id) {
                    maze.get_connector(tunnel.end_connector).point()
                        - maze.get_connector(tunnel.start_connector).point()
                } else {
                    maze.get_connector(tunnel.start_connector).point()
                        - maze.get_connector(tunnel.end_connector).point()
                },
            ),
        );
        DeadEndEnvironment {
            faces: vec![door.to_face()],
            exit_faces: vec![(EnvironmentIdentifier::Tunnel(tunnel_id), door.to_face())],
            dead_end: self.clone(),
        }
    }
}

pub(crate) struct DeadEndEnvironment {
    faces: Vec<Face<f64>>,
    exit_faces: Vec<(EnvironmentIdentifier, Face<f64>)>,
    dead_end: DeadEnd,
}

impl DeadEndEnvironment {
    pub(crate) fn dead_end(&self) -> &DeadEnd {
        &self.dead_end
    }
}
impl Environment for DeadEndEnvironment {
    fn faces(&self) -> &[Face<f64>] {
        &self.faces
    }
    fn exit_faces(&self) -> &[(EnvironmentIdentifier, Face<f64>)] {
        &self.exit_faces
    }
    fn up(&self, _camera_position: Point3<f64>) -> UnitVector3<f64> {
        self.dead_end.up
    }
}

/// The "abstract" representation of a maze -- all the features are represented as single points
/// This is in contrast with the Maze struct, which is much higher-fidelity and includes faces.
struct MazeDescriptor {
    landings: Vec<Landing>,
    dead_ends: Vec<DeadEnd>,
    tunnels: Vec<Tunnel>,
}
impl MazeDescriptor {
    fn add_landing(&mut self, location: Point3<f64>, up: UnitVector3<f64>) -> ConnectorIdentifier {
        let id = self.landings.len();
        let landing = Landing::new(id, location, up);
        self.landings.push(landing);
        ConnectorIdentifier::Landing(id)
    }
    fn add_dead_end(&mut self, location: Point3<f64>, up: UnitVector3<f64>) -> ConnectorIdentifier {
        let id = self.dead_ends.len();
        let dead_end = DeadEnd::new(id, location, up);
        self.dead_ends.push(dead_end);
        ConnectorIdentifier::DeadEnd(id)
    }
    fn add_tunnel(
        &mut self,
        start_connector: ConnectorIdentifier,
        end_connector: ConnectorIdentifier,
    ) -> usize {
        let new_tunnel_id = self.tunnels.len();
        self.tunnels.push(Tunnel {
            start_connector,
            end_connector,
        });
        self.get_connector_mut(start_connector)
            .add_tunnel(new_tunnel_id);
        self.get_connector_mut(end_connector)
            .add_tunnel(new_tunnel_id);
        new_tunnel_id
    }
    #[inline]
    fn get_connector(&self, connector_identifier: ConnectorIdentifier) -> &dyn Connector {
        match connector_identifier {
            ConnectorIdentifier::Landing(id) => &self.landings[id],
            ConnectorIdentifier::DeadEnd(id) => &self.dead_ends[id],
        }
    }
    #[inline]
    fn get_connector_mut(
        &mut self,
        connector_identifier: ConnectorIdentifier,
    ) -> &mut dyn Connector {
        match connector_identifier {
            ConnectorIdentifier::Landing(id) => &mut self.landings[id],
            ConnectorIdentifier::DeadEnd(id) => &mut self.dead_ends[id],
        }
    }
}

#[wasm_bindgen]
pub struct Maze {
    landings: Vec<LandingEnvironment>,
    tunnels: Vec<TunnelEnvironment>,
    dead_ends: Vec<DeadEndEnvironment>,
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
        fn rotate(axisangle: Vector3<f64>, vector: &Vector3<f64>) -> Vector3<f64> {
            let result =
                Matrix4::new_rotation(axisangle) * Vector4::new(vector.x, vector.y, vector.z, 0.0);
            Vector3::new(result.x, result.y, result.z)
        }

        let mut rng = rand::thread_rng();
        // let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(1);
        let mut maze = MazeDescriptor {
            landings: vec![],
            tunnels: vec![],
            dead_ends: vec![],
        };

        // let mut space_claims = vec![];

        /// Assuming a landing already has a tunnel coming into it,
        /// generates tunnels going out of it (recursively)
        fn extend_landing<Rng: RngCore>(
            start_landing: ConnectorIdentifier,
            input_direction: &UnitVector3<f64>,
            angle: f64,
            rng: &mut Rng,
            maze: &mut MazeDescriptor,
        ) {
            let tunnel_length_range: Uniform<f64> =
                Uniform::new(MIN_TUNNEL_LENGTH, MAX_TUNNEL_LENGTH);
            let tunnel_twist_range: Uniform<f64> = Uniform::new(MIN_TWIST, MAX_TWIST);
            let start_landing_id = match start_landing {
                ConnectorIdentifier::Landing(id) => id,
                _ => panic!(
                    "extend_landing must be passed a Landing, received {:?}",
                    start_landing
                ),
            };
            let landing = &maze.landings[start_landing_id];

            let connector_direction = rotate(*landing.up * angle, input_direction);
            let connector_vec = connector_direction * rng.sample(tunnel_length_range);
            let connector_point = landing.point + connector_vec;
            let connector_up = Unit::new_normalize(rotate(
                connector_vec.normalize() * rng.sample(tunnel_twist_range),
                &landing.up,
            ));
            let segment = parry3d::shape::Segment::new(landing.point, connector_point);
            let make_dead_end = rng.gen_bool(0.5);
            console_log!("make dead end {}", make_dead_end);
            if make_dead_end {
                let dead_end = maze.add_dead_end(connector_point, connector_up);
                maze.add_tunnel(ConnectorIdentifier::Landing(start_landing_id), dead_end);
            } else {
                // making a landing
                let new_landing_id = maze.add_landing(connector_point, connector_up);
                maze.add_tunnel(
                    ConnectorIdentifier::Landing(start_landing_id),
                    new_landing_id,
                );
                let landing_rotation_range: Uniform<f64> = Uniform::new(
                    min_angle_between_tunnels(),
                    1.5 * min_angle_between_tunnels(),
                );
                let angle_1 = rng.sample(landing_rotation_range);
                let angle_2 = angle_1 + rng.sample(landing_rotation_range);
                extend_landing(
                    new_landing_id,
                    &Unit::new_normalize(-connector_direction),
                    angle_1,
                    rng,
                    maze,
                );
                extend_landing(
                    new_landing_id,
                    &Unit::new_normalize(-connector_direction),
                    angle_2,
                    rng,
                    maze,
                );
            }
        }

        let start_point = point![0.0, 0.0, 0.0];
        let start_up = Unit::new_normalize(vector![0.0, 1.0, 0.0]);
        let start_landing = maze.add_landing(start_point, start_up);

        let input_direction = Unit::new_normalize(vector![1.0, 0.0, 0.0]);

        let landing_rotation_range: Uniform<f64> = Uniform::new(
            min_angle_between_tunnels(),
            2.0 * min_angle_between_tunnels(),
        );
        let angle_0 = 0.0;
        let angle_1 = rng.sample(landing_rotation_range);
        let angle_2 = angle_1 + rng.sample(landing_rotation_range);

        extend_landing(
            start_landing,
            &input_direction,
            angle_0,
            &mut rng,
            &mut maze,
        );
        extend_landing(
            start_landing,
            &input_direction,
            angle_1,
            &mut rng,
            &mut maze,
        );
        extend_landing(
            start_landing,
            &input_direction,
            angle_2,
            &mut rng,
            &mut maze,
        );

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
            dead_ends: maze
                .dead_ends
                .iter()
                .map(|dead_end| dead_end.to_environment(&maze))
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
    #[inline]
    pub(crate) fn dead_ends(&self) -> &[DeadEndEnvironment] {
        &self.dead_ends
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

    #[wasm_bindgen]
    pub fn uvs_to_float32array(&self) -> Vec<f32> {
        self.faces()
            .iter()
            .flat_map(|face| {
                face.break_into_uv_triangles()
                    .iter()
                    .flat_map(|triangle| [triangle.x as f32, triangle.y as f32])
                    .collect::<Vec<_>>()
            })
            .collect()
    }

    #[inline]
    pub(crate) fn faces(&self) -> Vec<Face<f64>> {
        self.tunnels
            .iter()
            .flat_map(|tunnel| tunnel.faces())
            .chain(self.landings.iter().flat_map(|landing| landing.faces()))
            .chain(self.dead_ends.iter().flat_map(|dead_end| dead_end.faces()))
            .cloned()
            .collect()
    }
}
