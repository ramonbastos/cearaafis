pub mod feature_template;
pub mod persistent_template;

pub use feature_template::*;
pub use persistent_template::*;

#[cfg(test)]
mod tests {
    #[test]
    fn templates_module_exists() {}
}
