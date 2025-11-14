use std::collections::HashMap;

use fruits_math::Vec2;

use crate::{asset::AssetHandle, render::StandardTexture};

pub struct Font {
    pub texture: AssetHandle<StandardTexture>,
    pub characters_uv: HashMap<char, [Vec2<f32>; 2]>,
    pub missing_character_uv: [Vec2<f32>; 2],
    pub character_ratio: f32,
}
