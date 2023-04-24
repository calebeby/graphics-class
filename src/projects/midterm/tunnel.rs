use nalgebra::{Point3, UnitVector3, Vector2};

use crate::{
    face::{Face, UVPair},
    maze::{
        ConnectorIdentifier, Coupler, Environment, EnvironmentIdentifier, MazeSkeleton,
        TUNNEL_HEIGHT, TUNNEL_SUBDIVISIONS, TUNNEL_WIDTH,
    },
};

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
    pub(crate) start_connector: ConnectorIdentifier,
    pub(crate) end_connector: ConnectorIdentifier,
}

impl Tunnel {
    pub(crate) fn to_environment(&self, maze: &MazeSkeleton) -> TunnelEnvironment {
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
