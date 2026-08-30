use super::minutia_type::MinutiaType;
/// Minutia: a single minutia point in the fingerprint — mirrors .NET Minutia.cs.
use crate::primitives::int_point::IntPoint;

#[derive(Debug, Clone, PartialEq)]
pub struct Minutia {
    pub position: IntPoint,
    pub angle: f64,
    pub typ: MinutiaType,
}

impl Minutia {
    pub fn new(position: IntPoint, angle: f64, typ: MinutiaType) -> Self {
        Self {
            position,
            angle,
            typ,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let m = Minutia::new(IntPoint::new(10, 20), 0.5, MinutiaType::Ending);
        assert_eq!(m.position.x(), 10);
        assert_eq!(m.position.y(), 20);
        assert_eq!(m.typ, MinutiaType::Ending);
    }
}
