pub mod feature_template;
pub mod persistent_template;

pub use feature_template::FeatureTemplate;
pub use persistent_template::PersistentTemplate;

#[cfg(test)]
mod tests {
    #[test]
    fn templates_module_exists() {}
}
