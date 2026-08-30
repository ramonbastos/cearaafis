/// Skeleton type enum — mirrors .NET SkeletonType.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkeletonType {
    Ridges,
    Valleys,
}

impl SkeletonType {
    pub fn is_ridges(&self) -> bool {
        matches!(self, SkeletonType::Ridges)
    }

    pub fn is_valleys(&self) -> bool {
        matches!(self, SkeletonType::Valleys)
    }
}
