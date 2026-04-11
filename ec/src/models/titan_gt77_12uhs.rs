use super::{Fan, ModelConfig};

pub const CONFIG: ModelConfig = ModelConfig {
    name: "Titan GT77 12UHS",
    has_dgpu: true,
    fans: Fan::Four,
};
