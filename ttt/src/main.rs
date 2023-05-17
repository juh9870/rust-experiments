use std::collections::HashMap;

use bevy_ecs::prelude::*;
use grid::Grid;
use macroquad::prelude::*;
use nalgebra::{vector, Vector2};
use rustc_hash::FxHashMap;

use frame_lag::FrameData;
use render::{draw_bevy_ecs, AssetStore, RenderElement, TextureRenderer};

use crate::tiles::{Tile, TileRegistry};

mod assets;
mod tiles;

fn window_conf() -> Conf {
    Conf {
        window_title: "TTT".to_owned(),
        // platform: miniquad::conf::Platform {
        //     swap_interval: Some(0),
        //     ..miniquad::conf::Platform::default()
        // },
        ..Default::default()
    }
}

#[warn(clippy::disallowed_types)]
#[macroquad::main(window_conf)]
async fn main() {
    set_pc_assets_folder("ttt/assets");
    let mut frame_data = FrameData::new();
    let mut assets = AssetStore::new();

    let tile_bg = assets.load_texture(assets::platforms::TILE).await;
    tile_bg.set_filter(FilterMode::Nearest);

    let mut registry = TileRegistry::default();
    registry.register_default(&mut assets).await;

    let board = Board::new(
        vec!["   x   ", "  x x  ", " x x x ", "x x x x"],
        [
            (' ', tiles::movement::EMPTY.name.as_str()),
            ('x', tiles::io::DISCARD.name.as_str()),
        ],
    );

    loop {
        frame_data.elapsed(get_frame_time());

        while frame_data.frame() {}

        board.draw(vector![0.0, 0.0], 64.0, tile_bg, &registry);
        next_frame().await
    }
}

#[derive(Debug, Clone)]
struct Board {
    grid: Grid<usize>,
    palette: Vec<String>,
}

impl Default for Board {
    fn default() -> Self {
        Board {
            grid: Grid::new(0, 0),
            palette: Vec::new(),
        }
    }
}

impl Board {
    pub fn new<'a>(
        data: Vec<&str>,
        palette_source: impl IntoIterator<Item = (char, &'a str)>,
    ) -> Board {
        let palette = FxHashMap::<char, &'a str>::from_iter(palette_source.into_iter());
        let mut resulting_palette = Vec::new();
        let mut board = Vec::new();
        let mut palette_mappings = FxHashMap::<char, usize>::default();
        for x in data.iter().flat_map(|e| e.chars()) {
            let mapped = palette[&x];
            let idx = palette_mappings.entry(x).or_insert_with(|| {
                resulting_palette.push(mapped.to_owned());
                resulting_palette.len() - 1
            });
            board.push(*idx);
        }
        dbg!(Board {
            grid: Grid::from_vec(board, data[0].len()),
            palette: resulting_palette,
        })
    }
}

impl Board {
    fn draw(&self, offset: Vector2<f32>, tile_size: f32, bg: Texture2D, registry: &TileRegistry) {
        let atlas = self
            .palette
            .iter()
            .map(|name| {
                registry
                    .get_texture(name)
                    .unwrap_or_else(|| panic!("Texture for tile {} is not registered", name))
            })
            .collect::<Vec<_>>();

        for (i, _) in self.grid.iter().enumerate() {
            let x = i % self.grid.cols();
            let y = i / self.grid.cols();
            draw_texture_ex(
                bg,
                offset.x + (x as f32) * tile_size,
                offset.y + (y as f32) * tile_size,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(Vec2::new(tile_size, tile_size)),
                    ..DrawTextureParams::default()
                },
            )
        }

        for (i, &id) in self.grid.iter().enumerate() {
            let x = i % self.grid.cols();
            let y = i / self.grid.cols();
            let texture = atlas[id];
            draw_texture_ex(
                texture,
                offset.x + (x as f32) * tile_size,
                offset.y + (y as f32) * tile_size,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(Vec2::new(tile_size, tile_size)),
                    ..DrawTextureParams::default()
                },
            )
        }
    }
}
