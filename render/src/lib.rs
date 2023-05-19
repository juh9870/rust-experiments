use std::cmp::Ordering;
use std::fmt::Debug;

#[cfg(feature = "bevy_ecs")]
use bevy_ecs::prelude::*;
use macroquad::prelude::*;
use nalgebra::{vector, Vector2};
use rustc_hash::FxHashMap;

pub fn render_element(element: &RenderElement, position: Vector2<f32>, scale: Vector2<f32>) {
    match &element.kind {
        RenderElementKind::Texture(texture) => {
            let texture2d = texture.texture;
            let x = position.x - texture2d.width() * texture.pivot.x;
            let y = position.y - texture2d.height() * texture.pivot.y;

            let w = texture
                .dest_size
                .map(|e| e.x)
                .unwrap_or_else(|| texture2d.width())
                * scale.x;
            let h = texture
                .dest_size
                .map(|e| e.y)
                .unwrap_or_else(|| texture2d.height())
                * scale.y;

            draw_texture_ex(
                texture2d,
                x,
                y,
                texture.color,
                DrawTextureParams {
                    dest_size: Some(Vec2::new(w, h)),
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
    pub dest_size: Option<Vector2<f32>>,
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
            dest_size: None,
            color: WHITE,
            pivot: vector![0.5, 0.5],
            flip_y: false,
            flip_x: false,
            rotation: 0.0,
            source: None,
        }
    }

    pub fn with_pivot(texture: Texture2D, pivot: Vector2<f32>) -> Self {
        TextureRenderer {
            pivot,
            ..Self::new(texture)
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

#[derive(Debug, Copy, Clone, Default)]
#[cfg_attr(feature = "bevy_ecs", derive(Component))]
pub struct RenderPosition(pub Vector2<f32>);

#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "bevy_ecs", derive(Component))]
pub struct RenderScale(pub Vector2<f32>);

impl Default for RenderScale {
    fn default() -> Self {
        RenderScale(vector![1.0, 1.0])
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "bevy_ecs", derive(Bundle))]
pub struct RenderBundle {
    pub pos: RenderPosition,
    pub scale: RenderScale,
    pub element: RenderElement,
}

impl RenderBundle {
    pub fn new(element: RenderElement) -> Self {
        RenderBundle {
            pos: Default::default(),
            scale: Default::default(),
            element,
        }
    }
}

#[cfg(feature = "bevy_ecs")]
pub fn draw_bevy_ecs(world: &mut World) {
    clear_background(DARKGRAY);
    // let store = world.resource::<&AssetStore>();

    let mut drawables = world
        .query::<(&RenderElement, &RenderPosition, Option<&RenderScale>)>()
        .iter_mut(world)
        .collect::<Vec<_>>();
    drawables.sort_by_key(|e| e.0);

    for item in drawables {
        render_element(
            item.0,
            item.1 .0,
            item.2.map(|e| e.0).unwrap_or_else(|| vector![0.0, 0.0]),
        );
    }
}

pub fn render_on_target<F: FnOnce(Vector2<f32>, &Camera2D)>(
    target: RenderTarget,
    parent_camera: Option<&Camera2D>,
    render: F,
) {
    let w = target.texture.width();
    let h = target.texture.height();
    let camera = Camera2D {
        render_target: Some(target),
        ..Camera2D::from_display_rect(Rect::new(0., 0., w, h))
    };
    set_camera(&camera);

    render(vector![w, h], &camera);

    match parent_camera {
        None => set_default_camera(),
        Some(camera) => set_camera(camera),
    }
}

pub fn draw_texture_fit(
    texture: Texture2D,
    bounds: Vector2<f32>,
    force_integer_scaling: bool,
    pivot: Vector2<f32>,
) {
    let fit = fit_into(
        vector![texture.width(), texture.height()],
        vector![bounds.x, bounds.y],
        force_integer_scaling,
        pivot,
    );

    draw_texture_ex(
        texture,
        fit.x,
        fit.y,
        WHITE,
        DrawTextureParams {
            flip_y: true,
            dest_size: Some(vec2(fit.w, fit.h)),
            ..Default::default()
        },
    );
}

pub fn fit_into(
    size: Vector2<f32>,
    bounds: Vector2<f32>,
    force_integer_scaling: bool,
    pivot: Vector2<f32>,
) -> Rect {
    let mut scale = (bounds.x / size.x).min(bounds.y / size.y);
    if force_integer_scaling {
        scale = scale.floor().max(1.);
    }
    let w = scale * size.x;
    let h = scale * size.y;
    Rect {
        x: (bounds.x - w) * pivot.x,
        y: (bounds.y - h) * pivot.y,
        w,
        h,
    }
}

#[derive(Debug, Copy, Clone)]
pub struct AnimationData {
    pub texture: Texture2D,
    pub frame_size: Vector2<u32>,
    pub gap: Vector2<u32>,
    pub start_frame: u32,
    pub end_frame: u32,
    pub fps: f32,
    pub repeat: AnimationRepeat,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum AnimationRepeat {
    None,
    Loop,
    Bounce,
}

#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "bevy_ecs", derive(Component))]
pub struct AnimationState {
    pub animation: AnimationData,
    pub start_time: u32,
}
