//! CearáAFIS — Rust re-implementation of SourceAFIS fingerprint recognition engine.

#[cfg(test)]
mod tests {
    #[test]
    fn hello_world() {
        assert_eq!(1 + 1, 2);
    }
}

pub mod extractor;
pub mod extractor_minutiae;
pub mod extractor_skeletons;
pub mod features;
pub mod matcher;
pub mod parameters;
pub mod primitives;
pub mod root;
pub mod templates;
pub mod transparency;

pub use features::*;
pub use parameters::*;
pub use primitives::*;
pub use root::*;
pub use templates::*;
