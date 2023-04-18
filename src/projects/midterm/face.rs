use nalgebra::{Matrix3, Point2, Point3, Unit, UnitVector3, Vector3};

use crate::{bounding_box::BoundingBox, Number};

#[derive(Debug, Clone)]
pub(crate) struct Face<T: Number> {
    points: Vec<Point3<T>>,
    bounding_box: BoundingBox<T, 3>,
    /// Unit vector in the "outwards" direction of the face
    normal: UnitVector3<T>,
    /// Unit vector in the direction of the axis between points 0 and 1
    x: UnitVector3<T>,
    /// Unit vector in the direction orthogonal to the x axis, on the face plane
    y: UnitVector3<T>,
    /// Matrix that converts a vector from absolute to relative (face-coordinates-based)
    absolute_to_relative: Matrix3<T>,
    /// The coordinates of each point, in terms of the face-plane-defined x and y axis
    points_relative: Vec<Point2<T>>,
}

impl<T: Number> Face<T> {
    pub(crate) fn new(points: Vec<Point3<T>>) -> Self {
        assert!(points.len() >= 3, "points must be 3 or more");
        let point_0 = points[0].coords;
        let point_1 = points[1].coords;
        let point_2 = points[2].coords;
        let x = Unit::new_normalize(point_1 - point_0);
        let normal = Unit::new_normalize((point_2 - point_1).cross(&x));
        let y: UnitVector3<T> = Unit::new_normalize(normal.cross(&x));
        let absolute_to_relative =
            Matrix3::from_columns(&[x.into_inner(), y.into_inner(), normal.into_inner()]);

        let points_relative = points
            .iter()
            .map(|point| {
                let p = absolute_to_relative * (point - point_0);
                Point2::new(p.x, p.y)
            })
            .collect();

        Self {
            bounding_box: BoundingBox::from_points(&points),
            points,
            normal,
            x,
            y,
            points_relative,
            absolute_to_relative,
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
    pub(crate) fn absolute_to_relative(&self, point: Vector3<T>) -> Vector3<T> {
        self.absolute_to_relative * point
    }

    #[inline]
    pub(crate) fn bounding_box(&self) -> &BoundingBox<T, 3> {
        &self.bounding_box
    }

    #[inline]
    pub(crate) fn points(&self) -> &[Point3<T>] {
        &self.points
    }

    #[inline]
    pub(crate) fn points_relative(&self) -> &[Point2<T>] {
        &self.points_relative
    }

    #[inline]
    pub(crate) fn origin(&self) -> &Point3<T> {
        &self.points[0]
    }

    #[inline]
    pub(crate) fn normal(&self) -> &UnitVector3<T> {
        &self.normal
    }

    #[inline]
    pub(crate) fn x(&self) -> &UnitVector3<T> {
        &self.x
    }

    #[inline]
    pub(crate) fn y(&self) -> &UnitVector3<T> {
        &self.y
    }
}
