use lazy_static::lazy_static;
use macroquad::prelude::*;
use rustc_hash::FxHashMap;

use render::AssetStore;

use crate::assets;

pub mod io;
pub mod movement;

macro_rules! tile {
    ($self:ident, $assets:ident, $path:tt) => {
        $self
            .register_auto($assets, names::$path, assets::symbols::$path)
            .await;
    };
}

#[derive(Debug, Default)]
pub struct TileRegistry {
    tiles: FxHashMap<String, TileRegistryData>,
}

impl TileRegistry {
    pub async fn register(&mut self, assets: &mut AssetStore, tile: Tile) {
        let texture = assets.load_texture(tile.texture_path.as_str()).await;
        texture.set_filter(FilterMode::Nearest);
        self.tiles
            .insert(tile.name.clone(), TileRegistryData { texture, tile });
    }

    pub async fn register_default(&mut self, assets: &mut AssetStore) {
        for tile in ALL_TILES.iter().cloned() {
            self.register(assets, tile).await;
        }
    }

    pub fn get_texture(&self, name: &str) -> Option<Texture2D> {
        Some(self.tiles.get(name)?.texture)
    }
}

#[derive(Debug)]
pub struct TileRegistryData {
    pub tile: Tile,
    pub texture: Texture2D,
}

lazy_static! {
    pub static ref ALL_TILES: Vec<Tile> = {
        Vec::new()
            .iter()
            .chain(io::ALL.iter())
            .chain(movement::ALL.iter())
            .cloned()
            .collect()
    };
}

#[derive(Debug, Clone)]
pub struct Tile {
    pub texture_path: String,
    pub name: String,
}

impl Tile {
    pub fn new(name: &str, texture_path: &str) -> Tile {
        Tile {
            name: name.to_owned(),
            texture_path: texture_path.to_owned(),
        }
    }
}
