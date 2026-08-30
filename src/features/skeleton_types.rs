/// Skeleton type extension methods — mirrors .NET SkeletonTypes.
use super::skeleton_type::SkeletonType;

pub trait SkeletonTypeExt {
    fn prefix(&self) -> &'static str;
}

impl SkeletonTypeExt for SkeletonType {
    fn prefix(&self) -> &'static str {
        match self {
            SkeletonType::Ridges => "ridges-",
            SkeletonType::Valleys => "valleys-",
        }
    }
}
