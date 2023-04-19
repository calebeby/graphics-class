use nalgebra::{OVector, Point, Point2, Point3};

use crate::{bounding_box::BoundingBox, console_log, face::Face, Number};

#[derive(Debug, Clone)]
pub(crate) struct Ray<T: Number, const DIM: usize> {
    start: Point<T, DIM>,
    end: Point<T, DIM>,
}

impl<T: Number, const DIM: usize> Ray<T, { DIM }> {
    #[inline]
    pub(crate) fn new(start: Point<T, DIM>, end: Point<T, DIM>) -> Self {
        Self { start, end }
    }
    #[inline]
    pub(crate) fn bounding_box(&self) -> BoundingBox<T, DIM> {
        BoundingBox::from_points(&[self.start, self.end])
    }
    #[inline]
    pub(crate) fn invert(&self) -> Self {
        Self::new(self.end, self.start)
    }
    #[inline]
    pub(crate) fn to_vector(&self) -> OVector<T, nalgebra::Const<DIM>> {
        self.end - self.start
    }
    #[inline]
    pub(crate) fn start(&self) -> &Point<T, DIM> {
        &self.start
    }
    #[inline]
    pub(crate) fn end(&self) -> &Point<T, DIM> {
        &self.end
    }

    pub(crate) fn point_intersection(&self, point: &Point<T, DIM>) -> bool {
        let vec = self.end - self.start;
        if vec.magnitude() <= T::epsilon() {
            return (point - self.start).magnitude() <= T::epsilon()
                || (point - self.end).magnitude() <= T::epsilon();
        }

        let vec_to_point = point - self.start;

        let scale = vec_to_point[0] / vec[0];
        for dim in 0..DIM {
            if (scale * vec[dim] - vec_to_point[dim]).abs() >= T::epsilon() {
                return false;
            }
        }
        true
    }
}

impl<T: Number> Ray<T, 2> {
    pub(crate) fn ray_intersection(&self, other: &Ray<T, 2>) -> Option<Point<T, 2>> {
        let intersection: Point2<T> = {
            // 1 is self, 2 is other

            // y = m1 (x - x1) + y1
            // y = m2 (x - x2) + y2
            //
            // m1 x - m1 x1 + y1 = m2 x - m2 x2 + y2
            //
            // (m1 - m2) x = m1 x1 -m2 x2 + y2 - y1
            //
            // x = (m1 x1 - m2 x2 + y2 - y1) / (m1 - m2)

            let dy_1 = self.start.y - self.end.y;
            let dx_1 = self.start.x - self.end.x;
            let is_vertical_1 = dx_1.into().abs() <= f64::EPSILON;

            let dy_2 = other.start.y - other.end.y;
            let dx_2 = other.start.x - other.end.x;
            let is_vertical_2 = dx_2.into().abs() <= f64::EPSILON;

            let m_1 = if is_vertical_1 {
                T::infinity()
            } else {
                dy_1 / dx_1
            };

            let m_2 = if is_vertical_2 {
                T::infinity()
            } else {
                dy_2 / dx_2
            };

            match (m_1, m_2) {
                (inf1, inf2) if inf1 == T::infinity() && inf2 == T::infinity() => None,
                (inf, _) if inf == T::infinity() => {
                    // self is vertical, x of intersection must be x of self
                    Some(Point2::new(
                        self.start.x,
                        other.start.y + m_2 * (self.start.x - other.start.x),
                    ))
                }
                (_, inf) if inf == T::infinity() => {
                    // other is vertical, x of intersection must be x of other
                    Some(Point2::new(
                        other.start.x,
                        self.start.y + m_1 * (other.start.x - self.start.x),
                    ))
                }
                (_, _) => {
                    let x = (m_1 * self.start.x - m_2 * other.start.x + other.start.y
                        - self.start.y)
                        / (m_1 - m_2);
                    Some(Point2::new(x, self.start.y + m_1 * (x - self.start.x)))
                }
            }
        }?;

        if self.bounding_box().includes_point(&intersection)
            && other.bounding_box().includes_point(&intersection)
        {
            Some(intersection)
        } else {
            None
        }
    }
}

impl Ray<f64, 3> {
    pub(crate) fn face_intersection(&self, face: &Face<f64>) -> bool {
        let polygon = parry3d::shape::ConvexPolyhedron::from_convex_hull(face.points()).unwrap();
        parry3d::query::intersection_test(
            &nalgebra::Isometry::identity(),
            &polygon,
            &nalgebra::Isometry::identity(),
            &parry3d::shape::Segment {
                a: self.start,
                b: self.end,
            },
        )
        .unwrap()

        // // If the bounding boxes do not intersect, the ray and the plane cannot intersect
        // if !face.bounding_box().intersects_with(&self.bounding_box()) {
        //     return None;
        // }

        // let face_origin_absolute_coords = face.origin().coords;

        // let start_offset = self.start.coords - face_origin_absolute_coords;
        // let end_offset = self.end.coords - face_origin_absolute_coords;

        // let start_in_relative_coords = Point::from(face.absolute_to_relative(start_offset));
        // let end_in_relative_coords = Point::from(face.absolute_to_relative(end_offset));
        // console_log!(
        //     "check int,\n{:#?}\n{:?}",
        //     face.points()
        //         .iter()
        //         .map(|v| format!("{:?}", v))
        //         .collect::<Vec<_>>(),
        //     self
        // );

        // // If the "distance from plane" coordinates for the start/end of the rays
        // // have the same sign, they are on the same side of the face plane,
        // // so the ray cannot intersect with the plane
        // // (it is allowable for one of the ray ends to have a z-sign of 0,
        // // meaning it just touches the plane: this counts as an intersection)
        // if start_in_relative_coords.z.signum() == end_in_relative_coords.z.signum()
        //     && start_in_relative_coords.z.abs() >= std::f64::EPSILON
        //     && end_in_relative_coords.z.abs() >= std::f64::EPSILON
        // {
        //     return None;
        // }

        // let intersection_relative_coords: Point3<f64> =
        //     if end_in_relative_coords.z == start_in_relative_coords.z {
        //         Point3::from(
        //             (start_in_relative_coords.coords + end_in_relative_coords.coords).scale(0.5),
        //         )
        //     } else {
        //         start_in_relative_coords
        //             + ((end_in_relative_coords - start_in_relative_coords)
        //                 * (-start_in_relative_coords.z
        //                     / (end_in_relative_coords.z - start_in_relative_coords.z)))
        //     };

        // // We have gotten through all the "easy cases",
        // // now we have to decide if the (x, y) intersection is within the 2D shape

        // // This algorithm draws a 2D ray between a "known point" outside the 2D shape,
        // // and checks the number of times the 2D ray intersects with the 2D shape.

        // // If the ray crosses an odd number of times,
        // // the point we are testing is inside the shape.
        // // Otherwise, the point is outside the shape.

        // let polygon =
        //     parry2d::shape::ConvexPolygon::from_convex_hull(&face.points_relative()).unwrap();

        // let point_outside_face: Point2<f64> = Point2::from(
        //     BoundingBox::from_points(face.points_relative())
        //         .max_pt
        //         .coords
        //         + Point2::new(1.0, 1.0).coords,
        // );

        // let ray_to_point_outside: Ray<f64, 2> = Ray::new(
        //     Point2::new(
        //         intersection_relative_coords.x,
        //         intersection_relative_coords.y,
        //     ),
        //     point_outside_face,
        // );

        // let intersects = parry2d::query::intersection_test(
        //     &nalgebra::Isometry::identity(),
        //     &polygon,
        //     &nalgebra::Isometry::identity(),
        //     &parry2d::shape::Segment {
        //         a: ray_to_point_outside.start,
        //         b: ray_to_point_outside.start,
        //     },
        // )
        // .unwrap();

        // let face_intersection = Point3::from(
        //     face.x().scale(intersection_relative_coords.x)
        //         + face.y().scale(intersection_relative_coords.y)
        //         + face.normal().scale(intersection_relative_coords.z),
        // );

        // if intersects {
        //     Some(face_intersection)
        // } else {
        //     None
        // }

        // let num_edge_crossings = face
        //     .points_relative()
        //     .windows(2)
        //     // Chain one more vertex pair to make it wrap around to the first vertex again
        //     .chain(std::iter::once(
        //         [
        //             *face.points_relative().last().unwrap(),
        //             face.points_relative()[0],
        //         ]
        //         .as_slice(),
        //     ))
        //     .filter(|vertex_pair| {
        //         let edge_ray: Ray<f64, 2> = Ray::new(vertex_pair[0], vertex_pair[1]);
        //         edge_ray.ray_intersection(&ray_to_point_outside).is_some()
        //     })
        //     .count()
        //     // Subtract the vertices that intersect with ray_to_point_outside
        //     // So they aren't double-counted by the two edges they are incident with
        //     - face
        //         .points_relative()
        //         .iter()
        //         .filter(|face_point| ray_to_point_outside.point_intersection(face_point))
        //         .count();

        // // If the ray crosses an odd number of times,
        // // the point we are testing is inside the shape.
        // // Otherwise, the point is outside the shape.
        // if num_edge_crossings % 2 == 0 {
        //     // Even number of crossings => outside
        //     return None;
        // }
    }
}

#[cfg(test)]
mod tests {
    use nalgebra::point;

    use super::*;

    #[test]
    fn test_face_intersection() {
        // let face = Face::new(vec![
        //     point!(0.0, 0.0, 0.0),
        //     point!(1.0, 0.0, 0.0),
        //     point!(1.0, 1.0, 0.0),
        //     point!(0.0, 1.0, 0.0),
        // ]);

        // // Through the middle of the face
        // let ray = Ray::new(point!(0.5, 0.5, 1.0), point!(0.5, 0.5, -1.0));
        // assert_eq!(ray.face_intersection(&face), Some(point!(0.5, 0.5, 0.0)));
        // assert_eq!(
        //     ray.invert().face_intersection(&face),
        //     Some(point!(0.5, 0.5, 0.0))
        // );

        // let ray = Ray::new(point!(0.5, 0.5, 0.0), point!(0.5, 0.5, -1.0));
        // assert_eq!(ray.face_intersection(&face), Some(point!(0.5, 0.5, 0.0)));
        // assert_eq!(
        //     ray.invert().face_intersection(&face),
        //     Some(point!(0.5, 0.5, 0.0))
        // );

        // // Right on the edge of the face
        // let ray = Ray::new(point!(1.0, 1.0, 1.0), point!(1.0, 1.0, -1.0));
        // assert_eq!(ray.face_intersection(&face), Some(point!(1.0, 1.0, 0.0)));
        // assert_eq!(
        //     ray.invert().face_intersection(&face),
        //     Some(point!(1.0, 1.0, 0.0))
        // );

        // // The ray doesn't reach far enough to go through the face
        // let ray = Ray::new(point!(0.5, 0.5, 1.0), point!(0.5, 0.5, 0.9));
        // assert_eq!(ray.face_intersection(&face), None);
        // assert_eq!(ray.invert().face_intersection(&face), None);

        // // The ray doesn't reach far enough to go through the face (other side)
        // let ray = Ray::new(point!(0.5, 0.5, -1.0), point!(0.5, 0.5, -0.9));
        // assert_eq!(ray.face_intersection(&face), None);
        // assert_eq!(ray.invert().face_intersection(&face), None);

        // // The ray is outside the shape of the face (easy since bounding boxes don't intersect)
        // let ray = Ray::new(point!(1.5, 1.5, 1.0), point!(1.5, 1.5, -1.0));
        // assert_eq!(ray.face_intersection(&face), None);
        // assert_eq!(ray.invert().face_intersection(&face), None);

        // let face = Face::new(vec![
        //     point!(0.0, 0.0, 0.0),
        //     point!(1.0, 0.0, 0.0),
        //     point!(1.0, 1.0, 0.0),
        // ]);

        // // The ray is outside the shape of the face (trickier since the bounding boxes do intersect)
        // let ray = Ray::new(point!(0.0, 1.0, 1.0), point!(0.0, 1.0, -1.0));
        // assert_eq!(ray.face_intersection(&face), None);
        // assert_eq!(ray.invert().face_intersection(&face), None);
    }

    #[test]
    fn test_2d_ray_intersection() {
        let vert_ray = Ray::new(point!(0.0, 0.0), point!(0.0, 1.0));
        let horiz_ray = Ray::new(point!(-0.5, 0.5), point!(0.5, 0.5));

        macro_rules! assert_intersection {
            ($ray_a: expr, $ray_b: expr, $expected: expr) => {
                let ray_a = &$ray_a;
                let ray_b = &$ray_b;
                let expected = $expected;
                assert_eq!(
                    ray_a.ray_intersection(ray_b),
                    expected,
                    "Self: {:?}, other: {:?}",
                    ray_a,
                    ray_b,
                );
                assert_eq!(
                    ray_b.ray_intersection(ray_a),
                    expected,
                    "Self: {:?}, other: {:?}",
                    ray_b,
                    ray_a,
                );
                assert_eq!(
                    ray_a.invert().ray_intersection(ray_b),
                    expected,
                    "Self: {:?}, other: {:?}",
                    ray_a.invert(),
                    ray_b,
                );
                assert_eq!(
                    ray_b.invert().ray_intersection(ray_a),
                    expected,
                    "Self: {:?}, other: {:?}",
                    ray_b.invert(),
                    ray_a,
                );
                assert_eq!(
                    ray_a.ray_intersection(&ray_b.invert()),
                    expected,
                    "Self: {:?}, other: {:?}",
                    ray_a,
                    ray_b.invert(),
                );
                assert_eq!(
                    ray_b.ray_intersection(&ray_a.invert()),
                    expected,
                    "Self: {:?}, other: {:?}",
                    ray_b,
                    ray_a.invert(),
                );
                assert_eq!(
                    ray_a.invert().ray_intersection(&ray_b.invert()),
                    expected,
                    "Self: {:?}, other: {:?}",
                    ray_a.invert(),
                    ray_b.invert(),
                );
                assert_eq!(
                    ray_b.invert().ray_intersection(&ray_a.invert()),
                    expected,
                    "Self: {:?}, other: {:?}",
                    ray_b.invert(),
                    ray_a.invert(),
                );
            };
        }
        assert_intersection!(vert_ray, horiz_ray, Some(point!(0.0, 0.5)));
        let diagonal_ray_1 = Ray::new(point!(-0.5, 0.0), point!(0.5, 1.0));
        assert_intersection!(vert_ray, diagonal_ray_1, Some(point!(0.0, 0.5)));
        assert_intersection!(horiz_ray, diagonal_ray_1, Some(point!(0.0, 0.5)));
        let diagonal_ray_2 = Ray::new(point!(0.5, 0.0), point!(-0.5, 1.0));
        assert_intersection!(horiz_ray, diagonal_ray_2, Some(point!(0.0, 0.5)));

        assert_intersection!(
            Ray::new(point!(0.0, 0.0), point!(0.0, 0.5)),
            Ray::new(point!(0.1, 0.1), point!(0.5, 0.5)),
            None
        );
    }
}
