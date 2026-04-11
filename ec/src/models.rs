pub mod titan_gt77_12uhs;

use std::fs;

/// A registry of model specific configs..
#[rustfmt::skip]
pub static MODEL_REGISTRY: ModelRegistry<'static> = ModelRegistry(&[
    titan_gt77_12uhs::CONFIG
]);

pub struct ModelRegistry<'a>(&'a [ModelConfig]);

impl ModelRegistry<'_> {
    /// Find the ModelConfig for this computer model as read from BIOS
    pub fn find(&self) -> Option<ModelConfig> {
        let buf = fs::read("/sys/class/dmi/id/product_name").ok()?;
        let s = String::from_utf8(buf).ok()?;
        MODEL_REGISTRY.get_from_name(s.trim())
    }

    pub fn get_from_name(&self, name: &str) -> Option<ModelConfig> {
        MODEL_REGISTRY.0.iter().find(|i| i.name == name).copied()
    }
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum Fan {
    #[default]
    One,
    Two,
    Three,
    Four,
}

#[derive(Debug, Copy, Clone)]
pub struct ModelConfig {
    pub name: &'static str,
    pub has_dgpu: bool,
    /// The highest number fan, i.e. how many fans total there are
    pub fans: Fan,
}
