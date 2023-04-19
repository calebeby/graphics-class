use nalgebra::{OVector, Point};

use crate::{bounding_box::BoundingBox, Number};

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
}

impl Ray<f64, 3> {
    #[inline]
    pub(crate) fn to_segment(&self) -> parry3d::shape::Segment {
        parry3d::shape::Segment {
            a: self.start,
            b: self.end,
        }
    }
}
