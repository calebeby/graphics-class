use nalgebra::{Point, Point3};

use crate::{bounding_box::BoundingBox, face::Face, Number};

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
    fn bounding_box(&self) -> BoundingBox<T, DIM> {
        BoundingBox::from_points(&[self.start, self.end])
    }
    #[inline]
    fn invert(&self) -> Self {
        Self::new(self.end, self.start)
    }
}

impl<T: Number> Ray<T, 3> {
    pub(crate) fn intersection(&self, face: &Face<T>) -> Option<Point3<T>> {
        // If the bounding boxes do not intersect, the ray and the plane cannot intersect
        if !face.bounding_box().intersects_with(&self.bounding_box()) {
            return None;
        }

        let face_origin_absolute_coords = face.origin().coords;

        let start_offset = self.start.coords - face_origin_absolute_coords;
        let end_offset = self.end.coords - face_origin_absolute_coords;

        let start_in_relative_coords = Point::from(face.absolute_to_relative(start_offset));
        let end_in_relative_coords = Point3::from(face.absolute_to_relative(end_offset));

        // If the "distance from plane" coordinates for the start/end of the rays
        // have the same sign, they are on the same side of the face plane,
        // so the ray cannot intersect with the plane
        // (it is allowable for one of the ray ends to have a z-sign of 0,
        // meaning it just touches the plane: this counts as an intersection)
        if start_in_relative_coords.z.signum() == end_in_relative_coords.z.signum()
            && start_in_relative_coords.z.into().abs() >= std::f64::EPSILON
            && end_in_relative_coords.z.into().abs() >= std::f64::EPSILON
        {
            return None;
        }

        let intersection_relative_coords: Point3<T> = start_in_relative_coords
            + ((end_in_relative_coords - start_in_relative_coords)
                * (-start_in_relative_coords.z
                    / (end_in_relative_coords.z - start_in_relative_coords.z)));

        // We have gotten through all the "easy cases",
        // now we have to decide if the (x, y) intersection is within the 2D shape

        // This algorithm draws a 2D ray between a "known point" outside the 2D shape,
        // and checks the number of times the 2D ray intersects with the 2D shape.
        //
        // If the ray interects an even number of times,
        // the point we are testing is inside the shape.
        // Otherwise, the point is outside the shape.

        let intersection = Point3::from(
            face.x().scale(intersection_relative_coords.x)
                + face.y().scale(intersection_relative_coords.y)
                + face.normal().scale(intersection_relative_coords.z),
        );

        Some(intersection)
    }
}

#[cfg(test)]
mod tests {
    use nalgebra::point;

    use super::*;

    #[test]
    fn test_intersection() {
        let face = Face::new(vec![
            point!(0.0, 0.0, 0.0),
            point!(1.0, 0.0, 0.0),
            point!(1.0, 1.0, 0.0),
            point!(0.0, 1.0, 0.0),
        ]);

        // Through the middle of the face
        let ray = Ray::new(point!(0.5, 0.5, 1.0), point!(0.5, 0.5, -1.0));
        assert_eq!(ray.intersection(&face), Some(point!(0.5, 0.5, 0.0)));
        assert_eq!(
            ray.invert().intersection(&face),
            Some(point!(0.5, 0.5, 0.0))
        );

        let ray = Ray::new(point!(0.5, 0.5, 0.0), point!(0.5, 0.5, -1.0));
        assert_eq!(ray.intersection(&face), Some(point!(0.5, 0.5, 0.0)));
        assert_eq!(
            ray.invert().intersection(&face),
            Some(point!(0.5, 0.5, 0.0))
        );

        // Right on the edge of the face
        let ray = Ray::new(point!(1.0, 1.0, 1.0), point!(1.0, 1.0, -1.0));
        assert_eq!(ray.intersection(&face), Some(point!(1.0, 1.0, 0.0)));
        assert_eq!(
            ray.invert().intersection(&face),
            Some(point!(1.0, 1.0, 0.0))
        );

        // The ray doesn't reach far enough to go through the face
        let ray = Ray::new(point!(0.5, 0.5, 1.0), point!(0.5, 0.5, 0.9));
        assert_eq!(ray.intersection(&face), None);
        assert_eq!(ray.invert().intersection(&face), None);

        // The ray doesn't reach far enough to go through the face (other side)
        let ray = Ray::new(point!(0.5, 0.5, -1.0), point!(0.5, 0.5, -0.9));
        assert_eq!(ray.intersection(&face), None);
        assert_eq!(ray.invert().intersection(&face), None);

        // The ray is outside the shape of the face (easy since bounding boxes don't intersect)
        let ray = Ray::new(point!(1.5, 1.5, 1.0), point!(1.5, 1.5, -1.0));
        assert_eq!(ray.intersection(&face), None);
        assert_eq!(ray.invert().intersection(&face), None);

        let face = Face::new(vec![
            point!(0.0, 0.0, 0.0),
            point!(1.0, 0.0, 0.0),
            point!(1.0, 1.0, 0.0),
        ]);

        // // The ray is outside the shape of the face (trickier since the bounding boxes do intersect)
        // let ray = Ray::new(point!(0.0, 1.0, 1.0), point!(0.0, 1.0, -1.0));
        // assert_eq!(ray.intersection(&face), None);
        // assert_eq!(ray.invert().intersection(&face), None);
    }
}
