pub mod titan_gt77_12uhs;

use std::fs;

use dmidecode::Structure;

/// A registry of model specific configs..
#[rustfmt::skip]
pub static MODEL_REGISTRY: ModelRegistry<'static> = ModelRegistry(&[
    titan_gt77_12uhs::CONFIG
]);

pub struct ModelRegistry<'a>(&'a [ModelConfig]);

impl ModelRegistry<'_> {
    pub fn get(&self) -> Option<ModelConfig> {
        // Get the SMBIOS header and DMI table from sysfs.
        let buf = fs::read("/sys/firmware/dmi/tables/smbios_entry_point").ok()?;
        let dmi = fs::read("/sys/firmware/dmi/tables/DMI").ok()?;
        let entry = dmidecode::EntryPoint::search(&buf).ok()?;

        for table in entry.structures(&dmi) {
            let Ok(table) = table else {
                log::error!("DMI tables contain malformed structure: {table:?}");
                continue;
            };

            match table {
                Structure::System(system) => {
                    #[rustfmt::skip]
                    return MODEL_REGISTRY
                        .0
                        .iter()
                        .find(|i| i.name == system.product)
                        .copied();
                }

                _ => continue,
            }
        }

        None
    }
}

#[derive(Debug, Copy, Clone, Default)]
pub enum Fan {
    #[default]
    One,
    Two,
    Three,
    Four,
}

impl Fan {
    pub fn as_num(&self) -> u8 {
        match self {
            Fan::One => 1,
            Fan::Two => 2,
            Fan::Three => 3,
            Fan::Four => 4,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct ModelConfig {
    pub name: &'static str,
    pub has_dgpu: bool,
    /// The highest number fan, i.e. how many fans total there are
    pub fans: Fan,
}
