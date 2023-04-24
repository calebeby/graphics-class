use maze::ConnectorIdentifier;
use nalgebra::{Point3, Unit, UnitVector3};

use crate::{
    face::Face,
    maze::{
        make_door, min_angle_between_tunnels, Connector, Coupler, Environment,
        EnvironmentIdentifier, MazeSkeleton, LANDING_RADIUS, TUNNEL_HEIGHT,
    },
};
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
    pub(crate) fn new(id: usize, point: Point3<f64>, up: UnitVector3<f64>) -> Self {
        Self {
            id,
            point,
            up,
            tunnel_ids: vec![],
        }
    }
    pub(crate) fn to_environment(&self, maze: &MazeSkeleton) -> LandingEnvironment {
        assert!(self.tunnel_ids.len() >= 2);
        let mut exit_faces: Vec<(EnvironmentIdentifier, Face<f64>)> = vec![];

        let floor_to_ceiling = self.up.into_inner() * TUNNEL_HEIGHT;

        let tunnels: Vec<_> = self
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
        let forwards = tunnels[0].1;
        let right = self.up.cross(&forwards);
        let mut sorted_tunnels: Vec<_> = tunnels
            .into_iter()
            .map(|(tunnel_id, towards_tunnel)| {
                let angle = f64::atan2(towards_tunnel.dot(&forwards), towards_tunnel.dot(&right));
                (tunnel_id, towards_tunnel, angle)
            })
            .collect();

        sorted_tunnels.sort_by(
            |(_tunnel_id_1, _towards_tunnel_1, angle_1),
             (_tunnel_id_2, _towards_tunnel_2, angle_2)| {
                angle_1.partial_cmp(angle_2).unwrap()
            },
        );
        let min_angle_between_tunnels = min_angle_between_tunnels();

        for (first_tunnel_angle, second_tunnel_angle) in sorted_tunnels
            .windows(2)
            .map(|tunnel_pair| (tunnel_pair[0].2, tunnel_pair[1].2))
            .chain([(
                sorted_tunnels.last().unwrap().2,
                sorted_tunnels.first().unwrap().2,
            )])
        {
            // There are two ways to measure the angles between radii of a circle:
            // clockwise and counterclockwise. We find the smaller of the two
            let angle_between = (second_tunnel_angle - first_tunnel_angle)
                .abs()
                .min(2.0 * std::f64::consts::PI - (second_tunnel_angle - first_tunnel_angle).abs());
            assert!(
                angle_between > min_angle_between_tunnels,
                "Angle between tunnels too small: {}, (min: {})\n\
                 Angles: {:#?}",
                angle_between,
                min_angle_between_tunnels,
                sorted_tunnels
                    .iter()
                    .map(|(_id, _towards, angle)| *angle)
                    .collect::<Vec<f64>>()
            );
        }

        let floor_points: Vec<Point3<f64>> = sorted_tunnels
            .into_iter()
            .flat_map(|(tunnel_id, towards_tunnel, _angle)| {
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
