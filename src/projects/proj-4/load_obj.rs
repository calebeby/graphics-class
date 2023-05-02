use nalgebra::{Point3, Vector2};

use crate::face::{Face, UVPair};

pub(crate) fn load_obj(obj: &str) -> Vec<Face<f64>> {
    let lines = obj.split('\n');
    let mut vertices: Vec<Point3<f64>> = vec![];
    let mut faces: Vec<Face<f64>> = vec![];
    let mut uvs: Vec<Vector2<f64>> = vec![];
    for (i, line) in lines.enumerate() {
        let line_num = i + 1;
        if line.starts_with('#') || line.trim() == "" {
            continue;
        };
        let chunks: Vec<&str> = line.split_whitespace().collect();
        let command = chunks[0];
        if command == "mtllib"
            || command == "o"
            || command == "vn"
            || command == "s"
            || command == "g"
            || command == "usemtl"
        {
            // ignore/no need to handle (at least for now)
        } else if command == "v" {
            vertices.push(Point3::new(
                chunks[1].parse().unwrap(),
                chunks[2].parse().unwrap(),
                chunks[3].parse().unwrap(),
            ));
        } else if command == "vt" {
            uvs.push(Vector2::new(
                chunks[1].parse().unwrap(),
                chunks[2].parse().unwrap(),
            ));
        } else if command == "f" {
            let face_vertices = &chunks[1..];
            if face_vertices.len() != 3 {
                panic!("Only triangles are supported in loading obj files for now",);
            }
            let face_points: Vec<UVPair<f64>> = face_vertices
                .iter()
                .map(|face_vertex| {
                    let ids: Vec<Option<usize>> =
                        face_vertex.split('/').map(|id| id.parse().ok()).collect();
                    let vertex_id: usize = ids[0].unwrap() - 1;
                    let uv = if let Some(uv_id) = ids[1] {
                        uvs[uv_id - 1]
                    } else {
                        Vector2::zeros()
                    };
                    UVPair {
                        point: vertices[vertex_id],
                        uv,
                    }
                })
                .collect();
            let new_face = Face::from_uv_pairs(face_points);
            faces.push(new_face);
        } else {
            panic!("Unrecognized command on line {}: {}", line_num, command);
        }
    }
    faces
}
