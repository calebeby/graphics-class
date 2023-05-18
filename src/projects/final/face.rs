use nalgebra::Point3;

use crate::Number;

#[derive(Debug, Clone)]
pub(crate) struct Face<T: Number> {
    points: Vec<Point3<T>>,
}

impl<T: Number> Face<T> {
    pub fn new(points: Vec<Point3<T>>) -> Self {
        Self { points }
    }
    /// Breaks a polygon into a bunch of triangle points (so they can be passed directly into webgl)
    pub(crate) fn break_into_triangles(&self) -> Vec<Point3<T>> {
        self.points[1..]
            .windows(2)
            .flat_map(|pair_of_points| vec![self.points[0], pair_of_points[0], pair_of_points[1]])
            .collect()
    }
}
