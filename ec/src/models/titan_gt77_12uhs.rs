use super::*;

pub const CONFIG: ModelConfig = ModelConfig {
    name: "Titan GT77 12UHS",
    has_dgpu: true,
    fans: FanConfig {
        max_speed: 150,
        count: Fans::Four,
    },
};
