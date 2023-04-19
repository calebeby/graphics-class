use nalgebra::{Point3, Unit, UnitVector3};

use crate::{bounding_box::BoundingBox, Number};

#[derive(Debug, Clone)]
pub(crate) struct Face<T: Number> {
    points: Vec<Point3<T>>,
    bounding_box: BoundingBox<T, 3>,
    /// Unit vector in the "outwards" direction of the face
    normal: UnitVector3<T>,
}

impl<T: Number> Face<T> {
    pub(crate) fn new(points: Vec<Point3<T>>) -> Self {
        assert!(points.len() >= 3, "points must be 3 or more");
        let point_0 = points[0].coords;
        let point_1 = points[1].coords;
        let point_2 = points[2].coords;
        let x = Unit::new_normalize(point_1 - point_0);
        let normal = Unit::new_normalize((point_2 - point_1).cross(&x));

        Self {
            bounding_box: BoundingBox::from_points(&points),
            points,
            normal,
        }
    }

    /// Breaks a polygon into a bunch of triangle points (so they can be passed directly into webgl)
    pub(crate) fn break_into_triangles(&self) -> Vec<Point3<T>> {
        self.points[1..]
            .windows(2)
            .flat_map(|pair_of_points| vec![self.points[0], pair_of_points[0], pair_of_points[1]])
            .collect()
    }

    #[inline]
    pub(crate) fn bounding_box(&self) -> &BoundingBox<T, 3> {
        &self.bounding_box
    }

    #[inline]
    pub(crate) fn normal(&self) -> &UnitVector3<T> {
        &self.normal
    }
}

impl Face<f64> {
    pub(crate) fn to_convex_polyhedron(&self) -> parry3d::shape::ConvexPolyhedron {
        parry3d::shape::ConvexPolyhedron::from_convex_hull(&self.points).unwrap()
    }
}
