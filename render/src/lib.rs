use std::cmp::Ordering;
use std::fmt::Debug;

#[cfg(feature = "bevy_ecs")]
use bevy_ecs::prelude::*;
use macroquad::prelude::*;
use nalgebra::{vector, Vector2};
use rustc_hash::FxHashMap;

pub fn render_element(element: &RenderElement) {
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
pub struct TextureRenderer {
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
    pub fn new(texture: Texture2D) -> Self {
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
#[cfg_attr(feature = "bevy_ecs", derive(Component))]
pub struct RenderElement {
    pub z_order: f32,
    pub kind: RenderElementKind,
}

impl RenderElement {
    pub fn new(kind: RenderElementKind, z_order: f32) -> Self {
        RenderElement { kind, z_order }
    }

    pub fn texture(texture: TextureRenderer, z_order: f32) -> Self {
        Self::new(RenderElementKind::Texture(texture), z_order)
    }
}

impl Eq for RenderElement {}

impl PartialOrd<Self> for RenderElement {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for RenderElement {
    fn cmp(&self, other: &Self) -> Ordering {
        let z_cmp = self.z_order.total_cmp(&other.z_order);
        if !z_cmp.is_eq() {
            return z_cmp;
        }
        self.kind.texture_ord().cmp(&other.kind.texture_ord())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum RenderElementKind {
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

#[derive(Debug, Default)]
#[cfg_attr(feature = "bevy_ecs", derive(Resource))]
pub struct AssetStore {
    loaded_textures: FxHashMap<String, Texture2D>,
}

impl AssetStore {
    pub fn new() -> Self {
        AssetStore::default()
    }

    pub async fn load_texture(&mut self, path: &str) -> Texture2D {
        self.try_load_texture(path)
            .await
            .unwrap_or_else(|err| panic!("Failed to load texture `{}`:\n{}", path, err))
    }

    pub async fn try_load_texture(&mut self, path: &str) -> Result<Texture2D, FileError> {
        if let Some(texture) = self.loaded_textures.get(path) {
            return Ok(*texture);
        }
        let texture = load_texture(path).await?;
        self.loaded_textures.insert(path.into(), texture);
        Ok(texture)
    }
}

#[cfg(feature = "bevy_ecs")]
pub fn draw_bevy_ecs(world: &mut World) {
    clear_background(DARKGRAY);
    // let store = world.resource::<&AssetStore>();

    let mut drawables = world
        .query::<&RenderElement>()
        .iter_mut(world)
        .collect::<Vec<_>>();
    drawables.sort();

    for item in drawables {
        render_element(item);
    }

    draw_text(
        format!("Fps: {}", get_fps()).as_str(),
        20.0,
        20.0,
        30.0,
        BLACK,
    );
}
