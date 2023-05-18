use bevy_ecs::prelude::*;
use grid::Grid;
use macroquad::prelude::*;
use nalgebra::{vector, Vector2};
use rustc_hash::FxHashMap;

use render::{
    draw_bevy_ecs, render_on_target, AssetStore, RenderBundle, RenderElement, RenderPosition,
    RenderScale, TargetRenderOptions, TextureRenderer,
};

use crate::glyphs::{Glyph, GlyphRegistry};

mod assets;
mod glyphs;

const TILE_Z: f32 = -2.0;
const GLYPH_Z: f32 = TILE_Z + 1.0;

const TILE_SIZE: f32 = 16.0;

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
    let w = 920;
    let h = 640;

    request_new_screen_size(w as f32, h as f32);

    let render_target = render_target(w, h);
    render_target.texture.set_filter(FilterMode::Nearest);

    let mut world = World::default();
    let mut schedule = Schedule::default();

    schedule.add_system(sprite_alignment_system);

    let board_render_target = setup_world(&mut world).await;
    board_render_target.texture.set_filter(FilterMode::Nearest);

    loop {
        schedule.run(&mut world);

        clear_background(BLACK);
        render_on_target(
            TargetRenderOptions {
                target: render_target,
                force_integer_scaling: false,
                bounds: vector![screen_width(), screen_height()],
                parent_camera: None,
            },
            |size, c| {
                clear_background(LIGHTGRAY);
                render_on_target(
                    TargetRenderOptions {
                        target: board_render_target,
                        force_integer_scaling: true,
                        bounds: vector![size.x / 2., size.y],
                        parent_camera: Some(c),
                    },
                    |_, _| {
                        draw_bevy_ecs(&mut world);
                    },
                );
            },
        );
        draw_text(
            format!("Fps: {}", get_fps()).as_str(),
            20.0,
            20.0,
            30.0,
            BLACK,
        );
        next_frame().await
    }
}

async fn setup_world(world: &mut World) -> RenderTarget {
    // let mut frame_data = FrameData::new();
    let mut assets = AssetStore::new();

    let tile_bg = assets.load_texture(assets::platforms::TILE).await;
    tile_bg.set_filter(FilterMode::Nearest);

    let mut registry = GlyphRegistry::default();
    registry.register_default(&mut assets).await;

    let board = Board::new(
        vec![
            "xxxxxxxxxxxx",
            "xxxxxxxxxxxx",
            "xxxxxxxxxxxx",
            "xxxxxxxxxxxx",
            "xxxxxxxxxxxx",
            "xxxxxxxxxxxx",
            "xxxxxxxxxxxx",
            "xxxxxxxxxxxx",
            "xxxxxxxxxxxx",
            "xxxxxxxxxxxx",
            "xxxxxxxxxxxx",
            "xxxxxxxxxxxx",
        ],
        [
            (' ', glyphs::movement::EMPTY.name.as_str()),
            ('x', glyphs::io::DISCARD.name.as_str()),
        ],
    );

    world.insert_resource(TileSprites { bg: tile_bg });
    world.insert_resource(assets);
    world.insert_resource(registry);

    spawn_board(world, board).expect("Failed to spawn board")
}

fn spawn_board(world: &mut World, board: Board) -> Option<RenderTarget> {
    let Board {
        palette,
        grid: glyphs,
    } = board;
    world.insert_resource(GlyphPalette(palette));
    world.insert_resource(BoardInfo {
        rows: glyphs.rows(),
        cols: glyphs.cols(),
    });
    world.clear_entities();

    let tiles_bg = world.resource::<TileSprites>().bg;
    world.spawn_batch(glyphs.iter().enumerate().map(|(i, _)| GridRenderBundle {
        render: RenderBundle::new(RenderElement::texture(
            TextureRenderer::with_pivot(tiles_bg, vector![0.0, 0.0]),
            TILE_Z,
        )),
        position: GridPosition::from_index(i, glyphs.cols()),
    }));

    let palette = world.resource::<GlyphPalette>();
    let registry = world.resource::<GlyphRegistry>();
    world.spawn_batch(
        glyphs
            .iter()
            .enumerate()
            .map(|(i, palette_id)| {
                Some(GlyphBundle {
                    id: GlyphId(*palette_id),
                    grid: GridRenderBundle {
                        render: RenderBundle::new(RenderElement::texture(
                            TextureRenderer::with_pivot(
                                registry.get_texture(palette.0.get(*palette_id)?)?,
                                vector![0.0, 0.0],
                            ),
                            GLYPH_Z,
                        )),
                        position: GridPosition::from_index(i, glyphs.cols()),
                    },
                })
            })
            .collect::<Option<Vec<_>>>()?,
    );
    Some(render_target(
        (glyphs.rows() as f32 * TILE_SIZE) as u32,
        (glyphs.cols() as f32 * TILE_SIZE) as u32,
    ))
}

#[derive(Debug, Resource)]
struct TileSprites {
    bg: Texture2D,
}

#[derive(Debug, Resource)]
struct BoardInfo {
    pub rows: usize,
    pub cols: usize,
}

#[derive(Debug, Clone, Component)]
struct GridPosition {
    pub row: usize,
    pub column: usize,
}

impl GridPosition {
    pub fn from_index(index: usize, columns: usize) -> Self {
        GridPosition {
            row: index / columns,
            column: index % columns,
        }
    }
}

#[derive(Debug, Clone, Component)]
struct GridRenderOffset(Vector2<f32>);

#[derive(Debug, Clone, Component)]
struct GlyphId(usize);

#[derive(Debug, Clone, Resource)]
struct GlyphPalette(Vec<String>);

#[derive(Debug, Bundle)]
struct GlyphBundle {
    id: GlyphId,
    grid: GridRenderBundle,
}

#[derive(Debug, Bundle)]
struct GridRenderBundle {
    render: RenderBundle,
    position: GridPosition,
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

fn sprite_alignment_system(
    mut query: Query<
        (
            &GridPosition,
            Option<&GridRenderOffset>,
            &mut RenderPosition,
        ),
        Or<(
            Changed<GridPosition>,
            Added<GridPosition>,
            Changed<GridRenderOffset>,
            Added<GridRenderOffset>,
        )>,
    >,
) {
    for (grid_pos, offset, mut render_pos) in query.iter_mut() {
        render_pos.0.x = grid_pos.column as f32 * TILE_SIZE;
        render_pos.0.y = grid_pos.row as f32 * TILE_SIZE;
        if let Some(offset) = offset {
            render_pos.0 += offset.0;
        }
    }
}
