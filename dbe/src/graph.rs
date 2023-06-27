use egui::{Color32, Ui};
use egui_node_graph::{
    DataTypeTrait, Graph, GraphEditorState, NodeDataTrait, NodeId, NodeResponse, NodeTemplateIter,
    NodeTemplateTrait, UserResponseTrait, WidgetValueTrait,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::borrow::Cow;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

// region response
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DbeNodeResponse {}

impl UserResponseTrait for DbeNodeResponse {}
// endregion

// region node data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbeNodeData {
    template: DbeNodeTemplate,
}

impl NodeDataTrait for DbeNodeData {
    type Response = DbeNodeResponse;
    type UserState = DbeGraphState;
    type DataType = DbeDataType;
    type ValueType = DbeValue;

    fn bottom_ui(
        &self,
        ui: &mut Ui,
        node_id: NodeId,
        graph: &Graph<Self, Self::DataType, Self::ValueType>,
        user_state: &mut Self::UserState,
    ) -> Vec<NodeResponse<Self::Response, Self>> {
        vec![]
    }
}
// endregion

// region data type
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum DbeDataType {
    Null,
    String,
    Number,
    Boolean,
}

impl DataTypeTrait<DbeGraphState> for DbeDataType {
    fn data_type_color(&self, user_state: &mut DbeGraphState) -> Color32 {
        match self {
            DbeDataType::Null => Color32::from_rgb(0, 0, 0),
            DbeDataType::String => Color32::from_rgb(112, 178, 255),
            DbeDataType::Number => Color32::from_rgb(161, 161, 161),
            DbeDataType::Boolean => Color32::from_rgb(204, 166, 214),
        }
    }

    fn name(&self) -> Cow<str> {
        todo!()
    }
}
// endregion

// region node template
#[derive(Debug, Copy, Clone, Serialize, Deserialize, EnumIter)]
pub enum DbeNodeTemplate {
    Declare,
}

impl NodeTemplateTrait for DbeNodeTemplate {
    type NodeData = DbeNodeData;
    type DataType = DbeDataType;
    type ValueType = DbeValue;
    type UserState = DbeGraphState;
    type CategoryType = &'static str;

    fn node_finder_label(&self, user_state: &mut Self::UserState) -> Cow<str> {
        Cow::Borrowed(match self {
            DbeNodeTemplate::Declare => "Declare constant",
        })
    }

    fn node_graph_label(&self, user_state: &mut Self::UserState) -> String {
        self.node_finder_label(user_state).to_string()
    }

    fn user_data(&self, user_state: &mut Self::UserState) -> Self::NodeData {
        DbeNodeData { template: *self }
    }

    fn build_node(
        &self,
        graph: &mut Graph<Self::NodeData, Self::DataType, Self::ValueType>,
        user_state: &mut Self::UserState,
        node_id: NodeId,
    ) {
        todo!()
    }
}

pub struct AllMyNodeTemplates;

impl NodeTemplateIter for AllMyNodeTemplates {
    type Item = DbeNodeTemplate;

    fn all_kinds(&self) -> Vec<Self::Item> {
        DbeNodeTemplate::iter().collect()
    }
}
// endregion

// region value
#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct DbeValue(Value);

impl WidgetValueTrait for DbeValue {
    type Response = DbeNodeResponse;
    type UserState = DbeGraphState;
    type NodeData = DbeNodeData;

    fn value_widget(
        &mut self,
        param_name: &str,
        node_id: NodeId,
        ui: &mut Ui,
        user_state: &mut Self::UserState,
        node_data: &Self::NodeData,
    ) -> Vec<Self::Response> {
        todo!()
    }
}

// endregion

pub struct DbeGraphState;

pub type DbeGraph = Graph<DbeNodeData, DbeDataType, DbeValue>;
pub type DbeEditorState =
    GraphEditorState<DbeNodeData, DbeDataType, DbeValue, DbeNodeTemplate, DbeGraphState>;

fn draw_graph(ui: &mut Ui, state: &mut DbeEditorState) {
    let _ = state.draw_graph_editor(ui, AllMyNodeTemplates, &mut DbeGraphState);
}
