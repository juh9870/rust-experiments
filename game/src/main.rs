use std::cmp::Ordering;

use hecs::Query;
use hecs::{Entity, World};
use macroquad::miniquad::gl::GLuint;
use macroquad::miniquad::graphics::Texture;
use macroquad::prelude::*;
use rapier2d::na::{vector, Vector1, Vector2};
use rapier2d::parry::utils::hashmap::HashMap;
use rustc_hash::FxHashMap;

use frame_lag::FrameData;

fn window_conf() -> Conf {
    Conf {
        window_title: "Test".to_owned(),
        platform: macroquad::miniquad::conf::Platform {
            swap_interval: Some(0),
            ..miniquad::conf::Platform::default()
        },
        ..Default::default()
    }
}

async fn spawn_sprites(world: &mut World, store: &mut AssetStore) {
    let texture = store.load_texture("logo.png").await;
    let spawned = (0..1).map(|n| {
        let mut sprite = TextureRenderer::new(texture);
        let n = n as f32;
        sprite.pos.x = 200.0 + 5.0 * n;
        sprite.pos.y = 200.0;
        (RenderElement::texture(sprite, -n), 0)
    });
    world.spawn_batch(spawned);
}

#[warn(clippy::disallowed_types)]
#[macroquad::main(window_conf)]
async fn main() {
    set_pc_assets_folder("game/assets");
    let mut frame_data = FrameData::new();
    let mut world = World::new();
    let mut assets = AssetStore::new();
    spawn_sprites(&mut world, &mut assets).await;

    loop {
        frame_data.elapsed(get_frame_time());

        while frame_data.frame() {
            frame(&mut world, &mut assets);
        }
        draw(&mut world, &assets);
        next_frame().await
    }
}

fn frame(world: &mut World, store: &mut AssetStore) {
    for (_, render) in world.query_mut::<(&mut RenderElement)>() {}
}

fn draw(world: &mut World, store: &AssetStore) {
    clear_background(DARKGRAY);

    let mut drawables = world
        .query_mut::<&RenderElement>()
        .into_iter()
        .map(RenderEntity)
        .collect::<Vec<_>>();
    drawables.sort();

    for RenderEntity((_, drawable)) in drawables {
        render_element(drawable);
    }

    draw_text(
        format!("Fps: {}", get_fps()).as_str(),
        20.0,
        20.0,
        30.0,
        BLACK,
    );
}

fn render_element(element: &RenderElement) {
    match &element.kind {
        RenderElementKind::Texture(texture) => {
            let texture2d = texture.texture;
            let x = texture.pos.x - texture2d.width() * texture.pivot.x;
            let y = texture.pos.y - texture2d.height() * texture.pivot.y;
            draw_texture_ex(
                texture2d,
                x,
                y,
                texture.color,
                DrawTextureParams {
                    dest_size: None,
                    flip_x: texture.flip_x,
                    flip_y: texture.flip_y,
                    rotation: texture.rotation,
                    source: texture.source,
                    pivot: None,
                },
            );
        }
    };
}

#[derive(Clone, Debug, PartialEq)]
struct TextureRenderer {
    pub texture: Texture2D,
    pub pos: Vector2<f32>,
    pub color: Color,
    pub source: Option<Rect>,
    pub rotation: f32,
    pub flip_x: bool,
    pub flip_y: bool,
    pub pivot: Vector2<f32>,
}

impl TextureRenderer {
    fn new(texture: Texture2D) -> Self {
        TextureRenderer {
            texture,
            pos: vector![0.0, 0.0],
            color: WHITE,
            pivot: vector![0.5, 0.5],
            flip_y: false,
            flip_x: false,
            rotation: 0.0,
            source: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct RenderElement {
    z_order: f32,
    kind: RenderElementKind,
}

impl RenderElement {
    fn new(kind: RenderElementKind, z_order: f32) -> Self {
        RenderElement {
            kind,
            z_order: z_order,
        }
    }

    fn texture(texture: TextureRenderer, z_order: f32) -> Self {
        return Self::new(RenderElementKind::Texture(texture), z_order);
    }
}

#[derive(Clone, Debug, PartialEq)]
enum RenderElementKind {
    Texture(TextureRenderer),
}

impl RenderElementKind {
    fn texture_ord(&self) -> impl Ord {
        match self {
            RenderElementKind::Texture(texture) => texture
                .texture
                .raw_miniquad_texture_handle()
                .gl_internal_id(),
        }
    }
}

/// Dummy wrapper used for sorting
#[derive(Clone, Debug)]
struct RenderEntity<'a>((Entity, &'a RenderElement));

impl<'a> PartialEq for RenderEntity<'a> {
    fn eq(&self, other: &Self) -> bool {
        return self.0 == other.0;
    }
}

impl<'a> Eq for RenderEntity<'a> {}

impl<'a> Ord for RenderEntity<'a> {
    fn cmp(&self, other: &Self) -> Ordering {
        let z_cmp = self.0 .1.z_order.total_cmp(&other.0 .1.z_order);
        if !z_cmp.is_eq() {
            return z_cmp;
        }
        let texture_cmp = self
            .0
             .1
            .kind
            .texture_ord()
            .cmp(&other.0 .1.kind.texture_ord());
        if !texture_cmp.is_eq() {
            return texture_cmp;
        }
        return self.0 .0.cmp(&other.0 .0);
    }
}

impl<'a> PartialOrd for RenderEntity<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Position(Vector2<f32>);

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
struct Rotation(f32);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct TextureId(usize);

#[derive(Debug)]
struct AssetStore {
    loaded_textures: HashMap<String, Texture2D>,
}

impl AssetStore {
    fn new() -> Self {
        AssetStore {
            loaded_textures: Default::default(),
        }
    }

    async fn load_texture(&mut self, path: &str) -> Texture2D {
        return self
            .try_load_texture(path)
            .await
            .unwrap_or_else(|err| panic!("Failed to load texture `{}`:\n{}", path, err));
    }

    async fn try_load_texture(&mut self, path: &str) -> Result<Texture2D, FileError> {
        if let Some(texture) = self.loaded_textures.get(path) {
            return Ok(*texture);
        }
        let texture = load_texture(path).await?;
        self.loaded_textures.insert(path.into(), texture);
        return Ok(texture);
    }
}
