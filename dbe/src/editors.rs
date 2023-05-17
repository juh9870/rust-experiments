use anyhow::{anyhow, Error};
use egui::{Ui, Widget};
use serde_json::Value;

use things::thing::bounds::BoundsRefinement;
use things::thing::primitives::PrimitiveRefinement;
use things::thing::{Thing, ThingLike};

use crate::vm::OperationResult;

macro_rules! some {
    ($body:expr) => {
        (|| Some($body))()
    };
}

fn set_value(
    value: &mut Value,
    thing: &impl ThingLike,
    new_value: impl Into<Value>,
) -> anyhow::Result<Vec<anyhow::Error>> {
    let mut new_value = new_value.into();
    let warns = thing.apply(&mut new_value)?;
    *value = new_value;
    Ok(warns)
}

pub type Editor = dyn Fn(&mut Ui, &mut Value, Thing) -> OperationResult;

pub fn edit_auto(ui: &mut Ui, value: &mut Value, thing: Thing) -> OperationResult {
    let field_type = thing.get_refinement::<PrimitiveRefinement>();
    match field_type {
        Some(PrimitiveRefinement::Number) => edit_number(ui, value, thing),
        Some(PrimitiveRefinement::String) => edit_string(ui, value, thing),
        Some(PrimitiveRefinement::Bool) => edit_bool(ui, value, thing),
        Some(PrimitiveRefinement::Null) => {
            ui.label("Null");
            OperationResult::Ok
        }
        None => OperationResult::Err(anyhow!("Field type can't be determined")),
        _ => edit_unsupported(ui, value, thing),
    }
}

pub fn edit_number(ui: &mut Ui, value: &mut Value, thing: Thing) -> OperationResult {
    let mut num = value.as_f64().unwrap_or(0.0);
    let bounds = thing.get_refinement::<BoundsRefinement>();
    let bounds = some! { bounds?.get_min()?..=bounds?.get_max()? };
    let mut drag = egui::DragValue::new(&mut num);
    if let Some(bounds) = bounds {
        drag = drag.clamp_range(bounds);
    }
    if drag.ui(ui).changed() {
        set_value(value, &thing, num).into()
    } else {
        OperationResult::Ok
    }
}

pub fn edit_string(ui: &mut Ui, value: &mut Value, thing: Thing) -> OperationResult {
    let mut data = value.as_str().unwrap_or("").to_string();
    if ui.text_edit_singleline(&mut data).changed() {
        set_value(value, &thing, data).into()
    } else {
        OperationResult::Ok
    }
}

pub fn edit_bool(ui: &mut Ui, value: &mut Value, thing: Thing) -> OperationResult {
    let mut data = value.as_bool().unwrap_or(false);
    if ui.checkbox(&mut data, "").changed() {
        set_value(value, &thing, data).into()
    } else {
        OperationResult::Ok
    }
}

pub fn edit_unsupported(ui: &mut Ui, value: &mut Value, thing: Thing) -> OperationResult {
    ui.label("Unsupported");
    OperationResult::Ok
}
