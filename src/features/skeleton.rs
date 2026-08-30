/// Skeleton: skeleton structure with type, size, and minutiae — mirrors .NET Skeleton.cs.
use crate::primitives::{bool_matrix::BooleanMatrix, int_point::IntPoint};
use super::skeleton_minutia::SkeletonMinutia;
use super::skeleton_type::SkeletonType;

pub struct Skeleton {
    pub type_: SkeletonType,
    pub size: IntPoint,
    pub minutiae: Vec<SkeletonMinutia>,
}

impl Skeleton {
    pub fn new(type_: SkeletonType, size: &IntPoint) -> Self {
        Self {
            type_,
            size: *size,
            minutiae: Vec::new(),
        }
    }

    pub fn add_minutia(&mut self, minutia: SkeletonMinutia) {
        self.minutiae.push(minutia);
    }

    pub fn remove_minutia(&mut self, minutia: &SkeletonMinutia) {
        self.minutiae.retain(|m| m.position.x != minutia.position.x || m.position.y != minutia.position.y);
    }

    pub fn shadow(&self) -> BooleanMatrix {
        let shadow = BooleanMatrix::new(self.size.x() as usize, self.size.y() as usize);
        shadow
    }
}

impl Default for Skeleton {
    fn default() -> Self {
        Self::new(SkeletonType::Ridges, &IntPoint::new(0, 0))
    }
}
