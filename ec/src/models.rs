pub mod titan_gt77_12uhs;

use std::fs;

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

impl ModelConfig {
    /// Find the ModelConfig for this computer model as read from BIOS
    pub fn find() -> Option<ModelConfig> {
        let buf = fs::read("/sys/class/dmi/id/product_name").ok()?;
        let s = String::from_utf8(buf).ok()?;
        Self::from_name(s.trim())
    }

    pub fn from_name(name: &str) -> Option<ModelConfig> {
        #[rustfmt::skip]
        static MODEL_REGISTRY: &[ModelConfig] = &[
            titan_gt77_12uhs::CONFIG
        ];

        MODEL_REGISTRY.iter().find(|i| i.name == name).copied()
    }
}
