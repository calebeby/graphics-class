use nalgebra::{Point3, Unit, UnitVector3, Vector2};

use crate::{bounding_box::BoundingBox, Number};

#[derive(Debug, Clone)]
pub(crate) struct Face<T: Number> {
    points: Vec<Point3<T>>,
    uvs: Vec<Vector2<T>>,
    bounding_box: BoundingBox<T, 3>,
    /// Unit vector in the "outwards" direction of the face
    normal: UnitVector3<T>,
}

#[derive(Clone, Copy)]
pub(crate) struct UVPair<T: Number> {
    pub(crate) point: Point3<T>,
    pub(crate) uv: Vector2<T>,
}

impl<T: Number> Face<T> {
    pub(crate) fn from_uv_pairs(uvs_and_points: Vec<UVPair<T>>) -> Self {
        let (points, uvs): (Vec<Point3<T>>, Vec<Vector2<T>>) = uvs_and_points
            .into_iter()
            .map(|uv_pair| (uv_pair.point, uv_pair.uv))
            .unzip();
        Self::new_with_uvs(points, uvs)
    }
    pub fn new(points: Vec<Point3<T>>) -> Self {
        let uvs = points.iter().map(|_p| Vector2::<T>::zeros()).collect();
        Self::new_with_uvs(points, uvs)
    }
    pub fn new_with_uvs(points: Vec<Point3<T>>, uvs: Vec<Vector2<T>>) -> Self {
        assert!(points.len() >= 3, "points must be 3 or more");
        assert_eq!(points.len(), uvs.len());
        let point_0 = points[0].coords;
        let point_1 = points[1].coords;
        let point_2 = points[2].coords;
        let x = Unit::new_normalize(point_1 - point_0);
        let normal = Unit::new_normalize((point_2 - point_1).cross(&x));

        Self {
            bounding_box: BoundingBox::from_points(&points),
            points,
            normal,
            uvs,
        }
    }

    /// Breaks a polygon into a bunch of triangle points (so they can be passed directly into webgl)
    pub(crate) fn break_into_triangles(&self) -> Vec<Point3<T>> {
        self.points[1..]
            .windows(2)
            .flat_map(|pair_of_points| vec![self.points[0], pair_of_points[0], pair_of_points[1]])
            .collect()
    }

    /// Breaks a polygon into a bunch of triangle points, returning the corresponding UVs
    pub(crate) fn break_into_uv_triangles(&self) -> Vec<Vector2<T>> {
        self.uvs[1..]
            .windows(2)
            .flat_map(|pair_of_points| vec![self.uvs[0], pair_of_points[0], pair_of_points[1]])
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

    #[inline]
    pub(crate) fn points(&self) -> &[Point3<T>] {
        &self.points
    }
}

impl Face<f64> {
    pub(crate) fn to_convex_polyhedron(&self) -> parry3d::shape::ConvexPolyhedron {
        parry3d::shape::ConvexPolyhedron::from_convex_hull(&self.points).unwrap()
    }
}
