use std::hash::BuildHasherDefault;

use im_rc::HashMap;
use rustc_hash::FxHasher;
use serde_json::Value;
use thiserror::Error;

use operation_result::OperationResult;
use things::thing::Thing;

use crate::vm::node::VmNode;
use crate::vm::state::VmState;

pub mod node;
pub mod operation_result;
pub mod state;
pub mod variables;

pub type ImHashMap<K, V> = HashMap<K, V, BuildHasherDefault<FxHasher>>;

#[derive(Debug, Clone, Error)]
pub enum VmErrors {
    #[error("Variable {} is not found", .0)]
    VariableNotFound(String),
}

#[derive(Debug)]
pub struct VmCode {
    nodes: Vec<VmNode>,
}

#[derive(Debug)]
pub struct VmRuntime<'a> {
    code: &'a VmCode,
    state: VmState<'a>,
    cur: usize,
}

impl<'a> VmRuntime<'a> {
    fn step(&mut self) -> Option<OperationResult> {
        let Some(node) = self.code.nodes.get(self.cur) else {
            return None;
        };
        Some(node.execute(&mut self.state))
    }
    /* Tracing
    fn step_tracing(&mut self) -> VmRunSnapshot<'a> {
        let snapshot = self.snapshot();
        let result = self.step();
        return VmRunSnapshot {
            runtime: snapshot,
            result: result,
        };
    }

    fn snapshot(&self) -> VmRuntime {
        VmRuntime {
            code: self.code,
            state: self.state.snapshot(),
            cur: self.cur,
        }
    }
    */
}

/* Tracing
#[derive(Debug)]
pub struct VmRunSnapshot<'a> {
    runtime: VmRuntime<'a>,
    result: Option<OperationResult>,
}

impl<'a> VmRunSnapshot<'a> {
    pub fn get_runtime(&self) -> &VmRuntime<'a> {
        &self.runtime
    }

    pub fn get_result(&self) -> &Option<OperationResult> {
        &self.result
    }
}
*/
