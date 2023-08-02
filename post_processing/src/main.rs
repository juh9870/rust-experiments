use anyhow::bail;
use macroquad::miniquad::window::set_window_size;
use macroquad::miniquad::{BlendFactor, BlendState, BlendValue, Equation};
use macroquad::prelude::*;
use notify::{Config, EventKind, PollWatcher, RecursiveMode, Watcher};
use std::cmp::min;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[macroquad::main("Post processing")]
async fn main() {
    let mut storage = ShaderStorage::build().unwrap();
    let render_target = render_target(screen_width().ceil() as u32, screen_height().ceil() as u32);
    render_target.texture.set_filter(FilterMode::Nearest);
    set_window_size(800, 800);
    loop {
        let err = storage.update_materials();
        if err.is_err() {
            println!("{err:?}")
        }
        let material_bg = storage
            .get_material("./post_processing/assets/gamma.frag")
            .unwrap();
        let material = storage
            .get_material("./post_processing/assets/hdr.frag")
            .unwrap();
        let material_screen = storage
            .get_material("./post_processing/assets/screen.frag")
            .unwrap();

        material.set_texture("Target", render_target.texture.clone());
        material_bg.set_texture("Target", render_target.texture.clone());
        material_screen.set_texture("Target", render_target.texture.clone());
        // drawing to the texture

        material.set_uniform("ITime", get_time() as f32);
        material_bg.set_uniform("ITime", get_time() as f32);
        material_screen.set_uniform("ITime", get_time() as f32);

        let w = screen_width();
        let h = screen_height();
        // 0..100, 0..100 camera
        set_camera(&Camera2D {
            render_target: Some(render_target.clone()),
            ..Camera2D::from_display_rect(Rect::new(0., 0., w, h))
        });

        clear_background(BLACK);
        gl_use_material(&material_bg);
        draw_rectangle(0., 0., w, h, BLACK);

        gl_use_material(&material);
        // gl_use_default_material();

        // let mut y = 10.;
        // let space = -10.;
        // let mut t = 1.;
        // while y < h {
        //     draw_line(w * 0.4, y, w * 0.6, y, t, WHITE);
        //     draw_rectangle(w * 0.2, y - t / 2., w * 0.2, t, RED);
        //     t *= 1.25;
        //     y += t + space;
        // }

        let cx = w / 2.;
        let cy = h / 2.;
        let r = w.min(h) / 3.;
        let hr = r / 2.;
        let sr = 10.;

        draw_rectangle(cx - r - sr, cy - r - sr, r, r, RED);
        // gl_use_material(&material);
        draw_rectangle(cx - r - sr, cy, r, r, RED);
        // gl_use_material(&material);
        draw_rectangle(cx, cy, r, r, RED);
        // gl_use_material(&material);
        draw_rectangle(cx, cy - r - sr, r, r, RED);
        // draw_circle(cx - hr, cy - hr, r, RED);
        // draw_circle(cx - hr, cy + hr, r, GREEN);
        // draw_circle(cx + hr, cy + hr, r, BLUE);
        // draw_circle(cx + hr, cy - hr, r, YELLOW);

        gl_use_default_material();

        // drawing to the screen

        set_default_camera();

        gl_use_material(&material_screen);
        render_target.texture.set_filter(FilterMode::Linear);
        draw_texture_ex(
            &render_target.texture,
            0.,
            0.,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(screen_width(), screen_height())),
                flip_y: true,
                ..Default::default()
            },
        );
        gl_use_default_material();

        next_frame().await;
    }
}

struct ShaderStorage {
    pending_updates: Arc<Mutex<Vec<PathBuf>>>,
    watcher: PollWatcher,
    materials: HashMap<PathBuf, Material>,
}

impl ShaderStorage {
    pub fn build() -> anyhow::Result<Self> {
        let pending = Arc::new(Mutex::new(vec![]));
        let pending_inner = pending.clone();
        let watcher = notify::PollWatcher::new(
            move |evt: notify::Result<notify::Event>| {
                match evt {
                    Ok(mut evt) => {
                        let mut data = pending_inner
                            .lock()
                            .expect("Failed to obtain pending updates lock");
                        data.append(&mut evt.paths)
                    }
                    Err(err) => {
                        println!("Shader file watch error: {}", err);
                    }
                };
            },
            notify::Config::default().with_poll_interval(Duration::from_millis(1000)),
        )?;
        Ok(ShaderStorage {
            pending_updates: pending,
            watcher,
            materials: Default::default(),
        })
    }

    pub fn get_material(&mut self, path: impl AsRef<Path>) -> anyhow::Result<Material> {
        let path = path.as_ref();
        match self.materials.get(path) {
            None => {
                let entry = self.load_material(path)?;
                let mat = entry.clone();
                self.materials.insert(path.to_owned(), entry);
                Ok(mat)
            }
            Some(material) => Ok(material.clone()),
        }
    }

    pub fn update_materials(&mut self) -> anyhow::Result<()> {
        let items = {
            let mut pending = self
                .pending_updates
                .lock()
                .expect("Failed to obtain pending updates lock");
            pending.drain(..).collect::<Vec<_>>()
        };
        let items: HashSet<PathBuf> = HashSet::from_iter(items.into_iter());
        let results = items
            .into_iter()
            .map(|p| -> anyhow::Result<()> {
                let material = self.load_material(&p)?;
                println!("Reloaded {:?}", p);
                self.materials.insert(p, material);
                Ok(())
            })
            .filter_map(Result::err)
            .map(|err| err.to_string())
            .collect::<Vec<_>>()
            .join("\n");

        if !results.is_empty() {
            bail!("Got errors while loading shaders:\n{}", results)
        }

        Ok(())
    }

    fn build_material(frag: &str, vertex: &str) -> anyhow::Result<Material> {
        Ok(load_material(
            ShaderSource {
                glsl_vertex: Some(vertex),
                glsl_fragment: Some(frag),
                metal_shader: None,
            },
            MaterialParams {
                pipeline_params: PipelineParams {
                    color_blend: Some(BlendState::new(
                        Equation::Add,
                        BlendFactor::Value(BlendValue::SourceAlpha),
                        BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
                        // BlendFactor::One,
                        // BlendFactor::One,
                    )),
                    ..Default::default()
                },
                uniforms: vec![("ITime".to_owned(), UniformType::Float1)],
                textures: vec!["Target".to_owned()],
                ..Default::default()
            },
        )?)
    }

    fn preprocess(path: &Path, dependencies: &mut Vec<PathBuf>) -> anyhow::Result<String> {
        let source = fs::read_to_string(&path)?;
        let mut result = String::new();
        for line in source.lines() {
            result += "\n";
            match line.strip_prefix("#include \"") {
                None => {
                    result += line;
                }
                Some(rest) => {
                    let end = rest.find('"');
                    let Some(end) = end else {
                        bail!("Invalid #include statement")
                    };
                    let include_name: &Path = rest[..end].as_ref();
                    if include_name.is_absolute() {
                        bail!("Attempted to include absolute file path")
                    }
                    let mut new_path = path.to_owned();
                    new_path.pop();
                    new_path.push(include_name);
                    let processed_included_source = Self::preprocess(&new_path, dependencies)
                        .map_err(|err| err.context(format!("While including {:?}", new_path)))?;
                    result.push_str(&processed_included_source);
                    dependencies.push(new_path);
                }
            }
        }
        Ok(result)
    }

    fn load_material(&mut self, path: &Path) -> anyhow::Result<Material> {
        let source = Self::preprocess(path, &mut vec![])?;
        let mat = Self::build_material(&source, VERTEX_SHADER)?;

        self.watcher.watch(path, RecursiveMode::NonRecursive)?;

        return Ok(mat);
    }
}

const CRT_FRAGMENT_SHADER: &str = include_str!("../assets/hdr.frag");
const BG_FRAGMENT_SHADER: &str = include_str!("../assets/gamma.frag");

const VERTEX_SHADER: &str = include_str!("../assets/main.vert");
