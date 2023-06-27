use std::hash::BuildHasherDefault;

use egui::{Color32, RichText, Ui};
use im_rc::{HashMap, Vector};
use macroquad::prelude::*;
use rustc_hash::{FxHashMap, FxHasher};
use serde_json::Value;

use things::thing::primitives::PrimitiveRefinement;
use things::thing::Thing;

use crate::editors::*;

pub mod editors;
mod graph;
pub mod vm;

#[macroquad::main("egui with macroquad")]
async fn main() {
    // let mut nodes = Vec::new();
    let mut editors = FxHashMap::<&str, Box<Editor>>::default();

    editors.insert("number", Box::new(edit_number));
    editors.insert("string", Box::new(edit_string));
    editors.insert("boolean", Box::new(edit_bool));

    loop {
        clear_background(WHITE);

        // Process keys, mouse etc.

        egui_macroquad::ui(|egui_ctx| {
            egui::Window::new("egui ❤ macroquad").show(egui_ctx, |ui| {
                ui.label("Test");
                if ui.button("Add").clicked() {
                    // nodes.push(EditorNode(VmNode::Assign(
                    //     "Test".to_string(),
                    //     PrimitiveRefinement::Number,
                    //     Value::from(0),
                    // )))
                }
                // egui::ScrollArea::vertical()
                //     .auto_shrink([true; 2])
                //     .show(ui, |ui| {
                //         for (i, node) in nodes.iter_mut().enumerate() {
                //             ui.push_id(i, |ui| {
                //                 node.draw(ui, i);
                //             });
                //         }
                //     });
            });
        });

        // Draw things before egui

        egui_macroquad::draw();

        // Draw things after egui

        next_frame().await;
    }
}

// struct EditorNode(VmNode);
//
// impl EditorNode {
//     fn draw(&mut self, ui: &mut Ui, index: usize) {
//         egui::Frame::group(ui.style()).show(ui, |ui| {
//             egui::CollapsingHeader::new(self.0.get_title())
//                 .id_source("List Element")
//                 // .open(Some(!info.collapsed))
//                 .default_open(true)
//                 .show(ui, |ui| {
//                     let result = self.0.draw(ui, index);
//                     match result {
//                         EditingResult::Ok => {}
//                         EditingResult::Warn(warns) => {
//                             for warning in warns {
//                                 ui.label(
//                                     RichText::new(warning.to_string())
//                                         .color(Color32::from_rgb(255, 255, 0)),
//                                 );
//                             }
//                         }
//                         EditingResult::Err(err) => {
//                             ui.label(
//                                 RichText::new(err.to_string()).color(Color32::from_rgb(255, 0, 0)),
//                             );
//                         }
//                     }
//                 });
//         });
//     }
// }
