/// FeatureTemplate — mirrors .NET FeatureTemplate.cs.
use crate::features::Minutia;
use crate::primitives::short_point::ShortPoint;

#[derive(Debug, Clone)]
pub struct FeatureTemplate {
    pub size: ShortPoint,
    pub minutiae: Vec<Minutia>,
}

impl FeatureTemplate {
    pub fn new(size: ShortPoint, minutiae: Vec<Minutia>) -> Self {
        Self { size, minutiae }
    }

    pub fn count(&self) -> usize {
        self.minutiae.len()
    }
}
