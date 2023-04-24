use nalgebra::{point, vector, Matrix4, Point3, Unit, UnitVector3, Vector3, Vector4};
use wasm_bindgen::prelude::wasm_bindgen;

use rand::{distributions::Uniform, SeedableRng};
use rand::{Rng, RngCore};

use crate::console_log;
use crate::dead_end::{DeadEnd, DeadEndEnvironment};
use crate::face::Face;
use crate::landing::{Landing, LandingEnvironment};
use crate::tunnel::{Tunnel, TunnelEnvironment};

pub(crate) const TUNNEL_WIDTH: f64 = 3.0;
pub(crate) const TUNNEL_HEIGHT: f64 = 4.0;
pub(crate) const LANDING_RADIUS: f64 = 2.0;
pub(crate) const TUNNEL_SUBDIVISIONS: usize = 20;
pub(crate) const MIN_TUNNEL_LENGTH: f64 = 20.0;
pub(crate) const MAX_TUNNEL_LENGTH: f64 = 60.0;
pub(crate) const MIN_TWIST: f64 = -120.0;
pub(crate) const MAX_TWIST: f64 = 120.0;
pub(crate) const TARGET_LANDING_COUNT: usize = 100;

pub(crate) fn min_angle_between_tunnels() -> f64 {
    2.0 * f64::atan((TUNNEL_WIDTH / 2.0) / LANDING_RADIUS)
}

pub(crate) struct Door {
    pub(crate) top_left: Point3<f64>,
    pub(crate) bottom_left: Point3<f64>,
    pub(crate) top_right: Point3<f64>,
    pub(crate) bottom_right: Point3<f64>,
}
impl Door {
    pub(crate) fn to_face(&self) -> Face<f64> {
        Face::new(vec![
            self.bottom_left,
            self.bottom_right,
            self.top_right,
            self.top_left,
        ])
    }
}
pub(crate) fn make_door(
    point: &Point3<f64>,
    up: &UnitVector3<f64>,
    forwards: &UnitVector3<f64>,
) -> Door {
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
    pub(crate) fn environment_identifier(&self) -> EnvironmentIdentifier {
        match self {
            ConnectorIdentifier::Landing(id) => EnvironmentIdentifier::Landing(*id),
            ConnectorIdentifier::DeadEnd(id) => EnvironmentIdentifier::DeadEnd(*id),
        }
    }
}

pub(crate) struct Coupler {
    pub(crate) point: Point3<f64>,
    pub(crate) up: UnitVector3<f64>,
}

/// The kinds of things that can be attached to the ends of tunnels
pub(crate) trait Connector {
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

/// The "abstract" representation of a maze -- all the features are represented as single points
/// This is in contrast with the Maze struct, which is much higher-fidelity and includes faces.
pub(crate) struct MazeSkeleton {
    pub(crate) landings: Vec<Landing>,
    pub(crate) dead_ends: Vec<DeadEnd>,
    pub(crate) tunnels: Vec<Tunnel>,
}
impl MazeSkeleton {
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
    pub(crate) fn get_connector(
        &self,
        connector_identifier: ConnectorIdentifier,
    ) -> &dyn Connector {
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
    pub fn generate(rng_seed: u32) -> Self {
        fn rotate(axisangle: Vector3<f64>, vector: &Vector3<f64>) -> Vector3<f64> {
            let result =
                Matrix4::new_rotation(axisangle) * Vector4::new(vector.x, vector.y, vector.z, 0.0);
            Vector3::new(result.x, result.y, result.z)
        }

        let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(rng_seed as u64);
        let mut maze = MazeSkeleton {
            landings: vec![],
            tunnels: vec![],
            dead_ends: vec![],
        };

        fn get_landing_tunnel_angles<Rng: RngCore>(rng: &mut Rng) -> Vec<f64> {
            let landing_rotation_range: Uniform<f64> = Uniform::new(
                min_angle_between_tunnels(),
                1.5 * min_angle_between_tunnels(),
            );
            let angle_1 = rng.sample(landing_rotation_range);
            let angle_2 = angle_1 + rng.sample(landing_rotation_range);
            vec![angle_1, angle_2]
        }

        /// Assuming a landing already has a tunnel coming into it,
        /// generates tunnels going out of it (recursively)
        fn extend_landing<Rng: RngCore>(
            start_landing: ConnectorIdentifier,
            input_direction: &UnitVector3<f64>,
            angle: f64,
            rng: &mut Rng,
            maze: &mut MazeSkeleton,
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
            let make_dead_end = rng.gen_bool(if maze.landings.len() > TARGET_LANDING_COUNT {
                0.9
            } else {
                0.1
            });
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
                let new_input_direction = Unit::new_normalize(-connector_direction);
                for angle in get_landing_tunnel_angles(rng) {
                    extend_landing(new_landing_id, &new_input_direction, angle, rng, maze);
                }
            }
        }

        let start_point = point![0.0, 0.0, 0.0];
        let start_up = Unit::new_normalize(vector![0.0, 1.0, 0.0]);
        let start_landing = maze.add_landing(start_point, start_up);

        let input_direction = Unit::new_normalize(vector![1.0, 0.0, 0.0]);

        extend_landing(start_landing, &input_direction, 0.0, &mut rng, &mut maze);

        for angle in get_landing_tunnel_angles(&mut rng) {
            extend_landing(start_landing, &input_direction, angle, &mut rng, &mut maze);
        }

        console_log!("random seed: {}", rng_seed);
        console_log!("tunnels: {}", maze.tunnels.len());
        console_log!("landings: {}", maze.landings.len());
        console_log!("dead ends: {}", maze.dead_ends.len());

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generation() {
        let mut rng = rand::thread_rng();
        for _ in 0..100 {
            Maze::generate(rng.next_u32());
        }
    }
}
