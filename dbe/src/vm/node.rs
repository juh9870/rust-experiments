use things::thing::primitives::PrimitiveRefinement;
use things::thing::Thing;

use crate::op_result;
use crate::vm::state::VmState;
use crate::vm::variables::{TypedVariable, ValueSource, ValueTarget};
use crate::vm::OperationResult;

#[derive(Debug, Clone)]
pub enum VmNode {
    Assign(ValueTarget, Thing, ValueSource),
}

impl VmNode {
    // fn draw(&mut self, ui: &mut Ui, index: usize) -> EditingResult {
    //     match self {
    //         VmNode::Assign(name, field_type, value) => {
    //             egui::Grid::new(format!("Node_{index}"))
    //                 .num_columns(2)
    //                 .striped(false)
    //                 .show(ui, |ui| {
    //                     ui.label("Field name");
    //                     ui.text_edit_singleline(name);
    //                     ui.end_row();
    //                     ui.label("Field type");
    //                     egui::ComboBox::from_label("")
    //                         .selected_text(format!("{:?}", field_type))
    //                         .show_ui(ui, |ui| {
    //                             const SUPPORTED_VALUES: [PrimitiveRefinement; 4] = [
    //                                 PrimitiveRefinement::Number,
    //                                 PrimitiveRefinement::String,
    //                                 PrimitiveRefinement::Bool,
    //                                 PrimitiveRefinement::Null,
    //                             ];
    //
    //                             for primitive in SUPPORTED_VALUES {
    //                                 ui.selectable_value(
    //                                     field_type,
    //                                     primitive,
    //                                     primitive.to_string(),
    //                                 );
    //                             }
    //                         });
    //                     ui.end_row();
    //                     ui.label("Value");
    //                     let result = edit_auto(ui, value, Thing::from(*field_type));
    //                     result
    //                 })
    //                 .inner
    //         }
    //     }
    // }
    // fn get_title(&self) -> String {
    //     match self {
    //         VmNode::Assign(name, _, _) => format!("Define variable: {name}"),
    //     }
    // }

    pub fn execute(&self, state: &mut VmState) -> OperationResult {
        match self {
            VmNode::Assign(target, refinement, source) => {
                let value = op_result!(source.get_value(state));
                target.set_value(state, value, refinement)
            }
        }
    }

    pub fn get_inputs(&self) -> impl IntoIterator<Item = TypedVariable<'_>> {
        match self {
            VmNode::Assign(_, thing, source) => match source {
                ValueSource::Const(_) => vec![],
                ValueSource::Variable(name) => vec![TypedVariable::new(name, thing)],
            },
        }
    }

    pub fn get_outputs(&self) -> impl IntoIterator<Item = TypedVariable<'_>> {
        match self {
            VmNode::Assign(target, thing, _) => match target {
                ValueTarget::Ignored => vec![],
                ValueTarget::Variable(name) => vec![TypedVariable::new(name, thing)],
                ValueTarget::NewVariable(name) => vec![TypedVariable::new(name, thing)],
            },
        }
    }
}
