use anyhow::anyhow;
use im_rc::{HashMap, Vector};
use serde_json::Value;

use things::thing::Thing;
use things::thing::ThingLike;

use crate::op_result;
use crate::vm::{ImHashMap, OperationResult, VmErrors};

#[derive(Debug)]
pub struct VmState<'a> {
    outer: Option<&'a mut VmState<'a>>,
    variable_names: ImHashMap<String, usize>,
    values: Vector<Value>,
    types: Vector<Thing>,
}

// #[derive(Debug, Copy, Clone)]
// pub struct VmVariableIndex(usize);

impl Default for VmState<'_> {
    fn default() -> Self {
        VmState {
            outer: None,
            variable_names: HashMap::default(),
            values: Vector::default(),
            types: Vector::default(),
        }
    }
}

impl<'a> VmState<'a> {
    // pub fn snapshot(&self) -> VmState {
    //     VmState {
    //         outer: self
    //             .outer
    //             .as_ref()
    //             .map(|state| &mut state.shallow_snapshot()),
    //         ..self.shallow_snapshot()
    //     }
    // }
    //
    // fn shallow_snapshot(&self) -> VmState {
    //     VmState {
    //         outer: None,
    //         variable_names: self.variable_names.clone(),
    //         types: self.types.clone(),
    //         values: self.values.clone(),
    //     }
    // }

    pub fn fork(&'a mut self) -> Self {
        VmState {
            outer: Some(self),
            ..VmState::default()
        }
    }

    // pub fn variable(&mut self, name: &str) -> VmVariableAccessor<'_> {
    //     if let Some(index) = self.variable_names.get(name) {
    //         VmVariableAccessor::Existing(ExistingVmVariableAccessor {
    //             state: self,
    //             index: *index,
    //         })
    //     } else {
    //         if let Some(outer) = self.outer {
    //             if let Some(index) = outer.variable_names.get(name) {
    //                 return VmVariableAccessor::Existing(ExistingVmVariableAccessor {
    //                     state: outer,
    //                     index: *index,
    //                 });
    //             }
    //         }
    //         VmVariableAccessor::Missing(MissingVmVariableAccessor {
    //             state: self,
    //             name: name.to_owned(),
    //         })
    //     }
    // }

    pub fn new_variable(&mut self, name: &str, value: Value, thing: Thing)
    /*-> ExistingVmVariableAccessor<'a>*/
    {
        self.values.push_back(value);
        self.types.push_back(thing);
        let index = self.values.len() - 1;
        self.variable_names.insert(name.to_owned(), index);
        // ExistingVmVariableAccessor {
        //     state: self,
        //     index: index,
        // }
    }

    pub fn set(&mut self, name: &str, mut value: Value) -> OperationResult {
        let Some(index) = self.variable_names.get(name) else {
            return VmErrors::VariableNotFound(name.to_owned()).into();
        };
        self.set_indexed(*index, value)
    }

    pub fn exists(&self, name: &str) -> bool {
        self.variable_names.contains_key(name)
    }

    pub fn set_or_else(
        &mut self,
        name: &str,
        value: Value,
        otherwise: impl FnOnce(&mut VmState<'a>, Value) -> OperationResult,
    ) -> OperationResult {
        let Some(index) = self.variable_names.get(name) else {
            return otherwise(self, value);
        };
        self.set_indexed(*index, value)
    }

    pub fn get_value(&self, name: &str) -> Option<&Value> {
        self.values.get(*self.variable_names.get(name)?)
    }

    pub fn get_type(&self, name: &str) -> Option<Option<&Thing>> {
        Some(self.types.get(*self.variable_names.get(name)?))
    }

    fn set_indexed(&mut self, index: usize, mut value: Value) -> OperationResult {
        let thing = &self.types[index];
        let result = op_result!(thing.apply(&mut value));
        self.values[index] = value;
        result.into()
    }
}

// pub enum VmVariableAccessor<'a> {
//     Missing(MissingVmVariableAccessor<'a>),
//     Existing(ExistingVmVariableAccessor<'a>),
// }
//
// impl<'a> VmVariableAccessor<'a> {
//     pub fn or_create(self, value: Value, thing: Thing) -> ExistingVmVariableAccessor<'a> {
//         match self {
//             VmVariableAccessor::Existing(data) => data,
//             VmVariableAccessor::Missing(missing) => {
//                 missing.state.new_variable(missing.name, value, thing)
//             }
//         }
//     }
//     pub fn or_create_with(
//         self,
//         source: impl FnOnce() -> (Value, Thing),
//     ) -> ExistingVmVariableAccessor<'a> {
//         match self {
//             VmVariableAccessor::Existing(data) => data,
//             VmVariableAccessor::Missing(missing) => {
//                 let (value, thing) = source();
//                 missing.state.new_variable(missing.name, value, thing)
//             }
//         }
//     }
//
//     pub fn existing(self) -> Option<ExistingVmVariableAccessor<'a>> {
//         match self {
//             VmVariableAccessor::Missing(_) => None,
//             VmVariableAccessor::Existing(data) => Some(data),
//         }
//     }
//
//     pub fn missing(self) -> Option<MissingVmVariableAccessor<'a>> {
//         match self {
//             VmVariableAccessor::Missing(data) => Some(data),
//             VmVariableAccessor::Existing(_) => None,
//         }
//     }
// }
//
// pub struct ExistingVmVariableAccessor<'a> {
//     state: &'a mut VmState<'a>,
//     index: usize,
// }
//
// impl<'a> ExistingVmVariableAccessor<'a> {
//     pub fn get_value(&mut self) -> &mut Value {
//         &mut self.state.values[self.index]
//     }
//
//     pub fn get_type(&mut self) -> &mut Thing {
//         &mut self.state.types[self.index]
//     }
//
//     pub fn set(&mut self, value: Value) -> OperationResult {
//         self.state.set(self.index, value)
//     }
// }
//
// pub struct MissingVmVariableAccessor<'a> {
//     state: &'a mut VmState<'a>,
//     name: String,
// }
//
// impl<'a> MissingVmVariableAccessor<'a> {
//     pub fn create(self, value: Value, thing: Thing) {
//         self.state.new_variable(self.name, value, thing);
//     }
// }
