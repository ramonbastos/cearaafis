/// PersistentTemplate — CBOR serialization format matching .NET PersistentTemplate.cs.
///
/// Mirrors the .NET serialization exactly:
/// - Version: "{package_version}-net" (e.g. "0.1.0-rust-net")
/// - Width/Height: short (i16) size of template
/// - PositionsX: i16[]
/// - PositionsY: i16[]
/// - Directions: f32[] (normalized angles)
/// - Types: string of 'B'/'E' chars (Bifurcation/Ending)
use crate::features::{Minutia, MinutiaType};
use crate::primitives::int_point::IntPoint;
use crate::primitives::short_point::ShortPoint;
use crate::templates::FeatureTemplate;
use serde::{Deserialize, Serialize};

/// Version suffix matching .NET FingerprintCompatibility.Version
const VERSION_SUFFIX: &str = "-net";

/// PersistentTemplate — mirrors .NET PersistentTemplate.cs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistentTemplate {
    pub version: String,
    pub width: i16,
    pub height: i16,
    pub positions_x: Vec<i16>,
    pub positions_y: Vec<i16>,
    pub directions: Vec<f32>,
    pub types: String,
}

impl PersistentTemplate {
    /// Encode a FeatureTemplate to PersistentTemplate.
    /// Mirrors .NET PersistentTemplate(FeatureTemplate).
    pub fn encode(template: &FeatureTemplate) -> Self {
        let count = template.count();
        let mut positions_x = Vec::with_capacity(count);
        let mut positions_y = Vec::with_capacity(count);
        let mut directions = Vec::with_capacity(count);
        let mut type_chars = Vec::with_capacity(count);

        for minutia in &template.minutiae {
            positions_x.push(minutia.position.x as i16);
            positions_y.push(minutia.position.y as i16);
            directions.push(minutia.angle as f32);
            if minutia.typ == MinutiaType::Bifurcation {
                type_chars.push('B');
            } else {
                type_chars.push('E');
            }
        }

        let types: String = type_chars.iter().collect();
        let version = format!("{}{}", "0.1.0", VERSION_SUFFIX);

        Self {
            version,
            width: template.size.x,
            height: template.size.y,
            positions_x,
            positions_y,
            directions,
            types,
        }
    }

    /// Decode a PersistentTemplate to FeatureTemplate.
    /// Mirrors .NET PersistentTemplate.Decode().
    pub fn decode(&self) -> FeatureTemplate {
        assert!(self.positions_x.len() == self.types.len());
        assert!(self.positions_y.len() == self.types.len());
        assert!(self.directions.len() == self.types.len());

        let count = self.types.len();
        let mut minutiae = Vec::with_capacity(count);

        for i in 0..count {
            let typ = match self.types.as_bytes()[i] {
                b'B' => MinutiaType::Bifurcation,
                _ => MinutiaType::Ending,
            };
            minutiae.push(Minutia::new(
                IntPoint::new(self.positions_x[i] as i32, self.positions_y[i] as i32),
                self.directions[i] as f64,
                typ,
            ));
        }

        FeatureTemplate::new(ShortPoint::new(self.width, self.height), minutiae)
    }

    /// Validate the template — mirrors .NET PersistentTemplate.Validate().
    pub fn validate(&self) -> Result<(), String> {
        if self.positions_x.is_empty() {
            return Err("Null/empty array of X positions.".to_string());
        }
        if self.positions_y.len() != self.positions_x.len() {
            return Err("Inconsistent lengths.".to_string());
        }
        if self.types.len() != self.positions_x.len() {
            return Err("Inconsistent lengths.".to_string());
        }

        for i in 0..self.positions_x.len() {
            if self.positions_x[i].abs() > 10_000 {
                return Err(format!("X position out of range at index {}", i));
            }
            if self.positions_y[i].abs() > 10_000 {
                return Err(format!("Y position out of range at index {}", i));
            }

            let direction = self.directions[i];
            if !(0.0..=2.0 * std::f32::consts::PI).contains(&direction) {
                return Err(format!("Denormalized direction at index {}", i));
            }

            match self.types.as_bytes()[i] {
                b'B' | b'E' => {}
                _ => return Err(format!("Unknown type at index {}", i)),
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode_roundtrip() {
        let minutiae = vec![
            Minutia::new(IntPoint::new(50, 50), 0.5, MinutiaType::Ending),
            Minutia::new(IntPoint::new(60, 60), 1.2, MinutiaType::Bifurcation),
            Minutia::new(IntPoint::new(70, 70), 2.1, MinutiaType::Ending),
        ];
        let tmpl = FeatureTemplate::new(ShortPoint::new(100, 100), minutiae);

        let persistent = PersistentTemplate::encode(&tmpl);
        let decoded = persistent.decode();

        assert_eq!(decoded.size.x, 100);
        assert_eq!(decoded.size.y, 100);
        assert_eq!(decoded.count(), 3);
        assert_eq!(decoded.minutiae[0].position.x, 50);
        assert_eq!(decoded.minutiae[1].position.y, 60);
        assert!(
            (decoded.minutiae[2].angle - 2.1_f32 as f64).abs() < 0.001,
            "Angle decoded: {:.5}",
            decoded.minutiae[2].angle
        );
    }

    #[test]
    fn test_encode_version_suffix() {
        let tmpl = FeatureTemplate::new(ShortPoint::new(100, 100), vec![]);
        let persistent = PersistentTemplate::encode(&tmpl);
        assert!(persistent.version.ends_with("-net"));
    }

    #[test]
    fn test_encode_types_b_and_e() {
        let minutiae = vec![
            Minutia::new(IntPoint::new(10, 10), 0.0, MinutiaType::Ending),
            Minutia::new(IntPoint::new(20, 20), 0.0, MinutiaType::Bifurcation),
        ];
        let tmpl = FeatureTemplate::new(ShortPoint::new(100, 100), minutiae);
        let persistent = PersistentTemplate::encode(&tmpl);
        assert_eq!(persistent.types.len(), 2);
        assert_eq!(persistent.types.as_bytes()[0], b'E');
        assert_eq!(persistent.types.as_bytes()[1], b'B');
    }

    #[test]
    fn test_validate_inconsistent_lengths() {
        let tmpl = PersistentTemplate {
            version: "test-net".to_string(),
            width: 100,
            height: 100,
            positions_x: vec![50],
            positions_y: vec![50, 60],
            directions: vec![0.5],
            types: "E".to_string(),
        };
        assert!(tmpl.validate().is_err());
    }

    #[test]
    fn test_validate_out_of_range_position() {
        let tmpl = PersistentTemplate {
            version: "test-net".to_string(),
            width: 100,
            height: 100,
            positions_x: vec![11000],
            positions_y: vec![50],
            directions: vec![0.5],
            types: "E".to_string(),
        };
        assert!(tmpl.validate().is_err());
    }

    #[test]
    fn test_validate_denormalized_direction() {
        let tmpl = PersistentTemplate {
            version: "test-net".to_string(),
            width: 100,
            height: 100,
            positions_x: vec![50],
            positions_y: vec![50],
            directions: vec![-1.0],
            types: "E".to_string(),
        };
        assert!(tmpl.validate().is_err());
    }
}
