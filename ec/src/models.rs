mod titan_gt77_12uhs;

use std::fs;

use serde::{Deserialize, Serialize};
use strum::{Display, EnumDiscriminants, IntoStaticStr};

use crate::fw::{Addr, Bit};

#[derive(Debug, Copy, Clone)]
pub struct FanConfig {
    pub max_speed: u8,
    /// The highest number fan, i.e. how many fans total there are
    pub count: Fans,
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Fans {
    #[default]
    One,
    Two,
    Three,
    Four,
}

#[derive(Debug, Clone)]
pub struct Method {
    pub name: &'static str,
    pub method: &'static str,
    pub addr: Addr,
    pub ty: &'static [MethodTy],
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, EnumDiscriminants)]
#[strum_discriminants(name(MethodOp))]
#[strum_discriminants(derive(Serialize, Deserialize, IntoStaticStr, Display))]
pub enum MethodTy {
    // single bit read/write
    // requires addr to be Addr::Single
    ReadBit { bit: Bit, invert: bool },
    WriteBit { bit: Bit, invert: bool },

    // Single byte read/write
    Read,
    Write,

    // Range of bytes read/write
    ReadRange,
    WriteRange,
}

#[derive(Debug, Copy, Clone)]
pub struct ModelConfig {
    pub name: &'static str,
    #[cfg_attr(test, expect(unused))]
    pub has_dgpu: bool,
    pub fans: FanConfig,
    pub methods: &'static [Method],
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
