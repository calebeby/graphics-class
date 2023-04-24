use nalgebra::{Point3, Unit, UnitVector3};

use crate::{
    face::Face,
    maze::{
        make_door, Connector, ConnectorIdentifier, Coupler, Environment, EnvironmentIdentifier,
        MazeSkeleton,
    },
};

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
    pub(crate) fn new(id: usize, point: Point3<f64>, up: UnitVector3<f64>) -> Self {
        Self {
            id,
            point,
            up,
            tunnel_id: None,
        }
    }
    pub(crate) fn to_environment(&self, maze: &MazeSkeleton) -> DeadEndEnvironment {
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
