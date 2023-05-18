use bevy_ecs::system::Resource;
use lazy_static::lazy_static;
use macroquad::prelude::*;
use rustc_hash::FxHashMap;

use render::AssetStore;

pub mod io;
pub mod movement;

#[derive(Debug, Default, Resource)]
pub struct GlyphRegistry {
    glyphs: FxHashMap<String, GlyphRegistryData>,
}

impl GlyphRegistry {
    pub async fn register(&mut self, assets: &mut AssetStore, glyph: Glyph) {
        let texture = assets.load_texture(glyph.texture_path.as_str()).await;
        texture.set_filter(FilterMode::Nearest);
        self.glyphs
            .insert(glyph.name.clone(), GlyphRegistryData { texture, tile: glyph });
    }

    pub async fn register_default(&mut self, assets: &mut AssetStore) {
        for glyph in ALL.iter().cloned() {
            self.register(assets, glyph).await;
        }
    }

    pub fn get_texture(&self, name: &str) -> Option<Texture2D> {
        Some(self.glyphs.get(name)?.texture)
    }
}

#[derive(Debug)]
pub struct GlyphRegistryData {
    pub tile: Glyph,
    pub texture: Texture2D,
}

lazy_static! {
    pub static ref ALL: Vec<Glyph> = {
        Vec::new()
            .iter()
            .chain(io::ALL.iter())
            .chain(movement::ALL.iter())
            .cloned()
            .collect()
    };
}

#[derive(Debug, Clone)]
pub struct Glyph {
    pub texture_path: String,
    pub name: String,
}

impl Glyph {
    pub fn new(name: &str, texture_path: &str) -> Glyph {
        Glyph {
            name: name.to_owned(),
            texture_path: texture_path.to_owned(),
        }
    }
}
