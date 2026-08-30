//! Unit tests for Skeleton feature type.
//! Mirrors SourceAFIS tests for Skeleton

#[cfg(test)]
mod tests {
    use cearaafis::features::{Skeleton, SkeletonMinutia, SkeletonType, SkeletonTypeExt};

    #[test]
    fn test_skeleton_new() {
        let skeleton = Skeleton::new(SkeletonType::Ridges, &cearaafis::primitives::IntPoint::new(100, 100));
        assert_eq!(skeleton.type_, SkeletonType::Ridges);
        assert_eq!(skeleton.size.x, 100);
        assert_eq!(skeleton.size.y, 100);
    }

    #[test]
    fn test_skeleton_default() {
        let skeleton = Skeleton::default();
        assert_eq!(skeleton.type_, SkeletonType::Ridges); // or whatever Default impl is
    }

    #[test]
    fn test_skeleton_add_minutia() {
        let mut skeleton = Skeleton::new(SkeletonType::Ridges, &cearaafis::primitives::IntPoint::new(10, 10));
        let minutia = SkeletonMinutia::new(cearaafis::primitives::IntPoint::new(5, 5));
        skeleton.add_minutia(minutia);
        assert_eq!(skeleton.minutiae.len(), 1);
    }

    #[test]
    fn test_skeleton_shadow() {
        let skeleton = Skeleton::new(SkeletonType::Ridges, &cearaafis::primitives::IntPoint::new(10, 10));
        let shadow = skeleton.shadow();
        assert_eq!(shadow.width(), 10);
        assert_eq!(shadow.height(), 10);
    }

    #[test]
    fn test_skeleton_type_prefix() {
        let ridge = SkeletonType::Ridges;
        assert_eq!(ridge.prefix(), "ridges-");

        let valley = SkeletonType::Valleys;
        assert_eq!(valley.prefix(), "valleys-");

        let inner = SkeletonType::Ridges;
        assert_eq!(inner.prefix(), "ridges-");

        let outer = SkeletonType::Valleys;
        assert_eq!(outer.prefix(), "valleys-");
    }
}
