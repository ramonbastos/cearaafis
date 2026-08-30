/// FeatureTemplate — mirrors .NET FeatureTemplate.
use crate::primitives::DoublePoint;

pub struct FeatureTemplate {
    pub size: DoublePoint,
    pub minutiae: Vec<crate::features::Minutia>,
}
