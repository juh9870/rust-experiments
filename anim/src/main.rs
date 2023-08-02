extern crate lyon;

use lyon::geom::Box2D;
use lyon::math::{point, Point};
use lyon::path::{Winding};
use lyon::path::builder::*;
use lyon::tessellation::*;
use lyon::tessellation::geometry_builder::simple_builder;
use macroquad::prelude::*;
use macroquad::prelude::Vertex;

fn window_conf() -> Conf {
    Conf {
        window_title: "Test".to_owned(),
        // platform: macroquad::miniquad::conf::Platform {
        //     swap_interval: Some(0),
        //     ..miniquad::conf::Platform::default()
        // },
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    // set_pc_assets_folder("game/assets");
    let w = 640;
    let h = 480;

    // let mut all_frames = vec![];

    let fps = 60;
    let duration = 10;

    let frames = fps * duration;

    let mut cur_frame = 0;
    loop {
        let progress = (cur_frame as f64) / (frames as f64);
        // cur_frame += 1;

        // let render_target = render_target(w, h);
        // let camera = Camera2D {
        //     render_target: Some(render_target),
        //     ..Camera2D::from_display_rect(Rect::new(0., 0., w as f32, h as f32))
        // };
        // set_camera(&camera);

        draw(progress);

        // set_default_camera();

        // all_frames.push(render_target.texture);

        next_frame().await
    }
}


fn draw(progress: f64) {
    let mut geometry: VertexBuffers<Point, u16> = VertexBuffers::new();
    let mut geometry_builder = simple_builder(&mut geometry);
    let options = FillOptions::tolerance(0.1);
    let mut tessellator = FillTessellator::new();

    let mut builder = tessellator.builder(
        &options,
        &mut geometry_builder,
    );

    builder.add_rounded_rectangle(
        &Box2D { min: point(50., 50.), max: point(150.0, 100.0) },
        &BorderRadii {
            top_left: 10.0,
            top_right: 5.0,
            bottom_left: 20.0,
            bottom_right: 25.0,
        },
        Winding::Positive,
    );

    builder.build().expect("Failed to render svg");

    let VertexBuffers { vertices, indices, .. } = geometry;

    draw_mesh(&Mesh {
        vertices: vertices.iter().map(vert_builder(RED)).collect(),
        indices,
        texture: None,
    })
}

fn vert_builder(color: Color) -> impl Fn(&Point) -> macroquad::models::Vertex {
    move |input: &Point| {
        vert(input, color)
    }
}

fn vert(input: &Point, color: Color) -> macroquad::models::Vertex {
    macroquad::models::Vertex {
        position: Vec3::new(input.x, input.y, 0.),
        uv: Vec2::ZERO,
        color,
    }
}