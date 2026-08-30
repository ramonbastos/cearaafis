//! CearáAFIS — Rust re-implementation of SourceAFIS fingerprint recognition engine.

#[cfg(test)]
mod tests {
    #[test]
    fn hello_world() {
        assert_eq!(1 + 1, 2);
    }
}

pub mod primitives;
pub mod parameters;
pub mod features;
pub mod templates;
pub mod extractor;
pub mod extractor_skeletons;
pub mod extractor_minutiae;
pub mod matcher;
pub mod root;
pub mod transparency;

pub use primitives::*;
pub use parameters::*;
pub use features::*;
pub use templates::*;
pub use root::*;
