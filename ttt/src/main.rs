use bevy_ecs::prelude::*;
use macroquad::prelude::*;
use nalgebra::Vector2;

use frame_lag::FrameData;
use render::{draw_bevy_ecs, AssetStore, RenderElement, TextureRenderer};

mod assets;

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
    set_pc_assets_folder("game/assets");
    let mut frame_data = FrameData::new();
    let mut world = World::default();
    let mut schedule = Schedule::default();
    let assets = AssetStore::new();
    world.insert_resource(assets);

    loop {
        frame_data.elapsed(get_frame_time());

        while frame_data.frame() {
            schedule.run(&mut world);
        }

        draw_bevy_ecs(&mut world);
        next_frame().await
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Position(Vector2<f32>);

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
struct Rotation(f32);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct TextureId(usize);
