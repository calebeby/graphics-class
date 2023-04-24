use nalgebra::Point3;

use crate::face::Face;

pub(crate) fn load_obj(obj: &str) -> Vec<Face<f64>> {
    let lines = obj.split('\n');
    let mut vertices: Vec<Point3<f64>> = vec![];
    let mut faces: Vec<Face<f64>> = vec![];
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
            || command == "vt"
            || command == "s"
        {
            // ignore/no need to handle (at least for now)
        } else if command == "v" {
            vertices.push(Point3::new(
                chunks[1].parse().unwrap(),
                chunks[2].parse().unwrap(),
                chunks[3].parse().unwrap(),
            ));
        } else if command == "f" {
            let face_vertices = &chunks[1..];
            if face_vertices.len() != 3 {
                panic!("Only triangles are supported in loading obj files for now",);
            }
            let face_points: Vec<Point3<f64>> = face_vertices
                .iter()
                .map(|face_vertex| {
                    let vertex_id: usize = face_vertex.split('/').next().unwrap().parse().unwrap();
                    let vertex_id = vertex_id - 1;
                    vertices[vertex_id]
                })
                .collect();
            let new_face = Face::new(face_points);
            faces.push(new_face);
        } else {
            panic!("Unrecognized command on line {}: {}", line_num, command);
        }
    }
    faces
}
