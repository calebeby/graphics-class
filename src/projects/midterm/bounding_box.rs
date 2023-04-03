use crate::Number;
use nalgebra::Point;

#[derive(Debug)]
pub(crate) struct BoundingBox<T: Number, const DIM: usize> {
    min_pt: Point<T, { DIM }>,
    max_pt: Point<T, { DIM }>,
}

impl<T: Number, const DIM: usize> BoundingBox<T, { DIM }> {
    pub(crate) fn from_points(points: &[Point<T, DIM>]) -> Self {
        assert!(!points.is_empty());
        let mut min_pt = points[0];
        let mut max_pt = points[0];
        for point in points {
            for dim in 0..DIM {
                if point[dim] < min_pt[dim] {
                    min_pt[dim] = point[dim];
                }
                if point[dim] > max_pt[dim] {
                    max_pt[dim] = point[dim];
                }
            }
        }
        Self { min_pt, max_pt }
    }

    pub(crate) fn intersects_with(&self, other: &BoundingBox<T, { DIM }>) -> bool {
        /// Check whether two (min, max) ranges overlap with each other (inclusive)
        #[inline]
        fn range_overlaps<T: PartialOrd + Copy>(range_1: (T, T), range_2: (T, T)) -> bool {
            if range_1.1 > range_2.1 {
                // range 1 has a higher max
                range_2.1 >= range_1.0
            } else {
                // range 2 has a higher max
                range_1.1 >= range_2.0
            }
        }

        let mut overlaps = true;
        // Every range (x, y, z) must overlap for the two bounding boxes
        for dim in 0..DIM {
            overlaps &= range_overlaps(
                (self.min_pt[dim], self.max_pt[dim]),
                (other.min_pt[dim], other.max_pt[dim]),
            );
        }
        overlaps
    }
}

#[cfg(test)]
mod tests {
    use nalgebra::Point3;

    use super::*;

    #[test]
    fn test_construction() {
        let bb = BoundingBox::from_points(&[Point3::new(0.0, 0.0, 0.0)]);
        assert_eq!(bb.min_pt, Point3::new(0.0, 0.0, 0.0));
        assert_eq!(bb.max_pt, Point3::new(0.0, 0.0, 0.0));

        let bb =
            BoundingBox::from_points(&[Point3::new(-1.0, 0.0, 0.0), Point3::new(1.0, 1.0, 1.0)]);
        assert_eq!(bb.min_pt, Point3::new(-1.0, 0.0, 0.0));
        assert_eq!(bb.max_pt, Point3::new(1.0, 1.0, 1.0));

        let bb = BoundingBox::from_points(&[
            Point3::new(-1.0, 0.0, 1.0),
            Point3::new(1.0, -1.0, 0.0),
            Point3::new(0.0, 1.0, -1.0),
        ]);
        assert_eq!(bb.min_pt, Point3::new(-1.0, -1.0, -1.0));
        assert_eq!(bb.max_pt, Point3::new(1.0, 1.0, 1.0));
    }

    #[test]
    fn test_intersection() {
        // The corners of these cubes touch
        let bb1 = BoundingBox {
            min_pt: Point3::new(0.0, 0.0, 0.0),
            max_pt: Point3::new(1.0, 1.0, 1.0),
        };
        let bb2 = BoundingBox {
            min_pt: Point3::new(1.0, 1.0, 1.0),
            max_pt: Point3::new(2.0, 2.0, 2.0),
        };
        assert!(bb1.intersects_with(&bb2));
        assert!(bb2.intersects_with(&bb1));

        // X and Y ranges overlap, but not z
        let bb1 = BoundingBox {
            min_pt: Point3::new(0.0, 0.0, 0.0),
            max_pt: Point3::new(1.0, 1.0, 1.0),
        };
        let bb2 = BoundingBox {
            min_pt: Point3::new(0.0, 0.0, 2.0),
            max_pt: Point3::new(1.0, 1.0, 3.0),
        };
        assert!(!bb1.intersects_with(&bb2));
        assert!(!bb2.intersects_with(&bb1));

        // X range is fully within other x range
        let bb1 = BoundingBox {
            min_pt: Point3::new(0.0, 0.0, 0.0),
            max_pt: Point3::new(1.0, 1.0, 1.0),
        };
        let bb2 = BoundingBox {
            min_pt: Point3::new(0.5, 0.5, 0.0),
            max_pt: Point3::new(0.5, 0.5, 1.0),
        };
        assert!(bb1.intersects_with(&bb2));
        assert!(bb2.intersects_with(&bb1));
    }
}
