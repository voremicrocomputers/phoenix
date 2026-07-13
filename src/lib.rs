use std::collections::BTreeMap;
use crate::compile::OuterFunction;
use crate::tree::{FElmVarDef, FType};
use crate::vm::{PCell, RTOuterFunction};

pub mod compile;
pub mod phoenixarch;
pub mod tokenize;
pub mod tree;
pub mod vm;

#[derive(Clone)]
pub struct OuterFunctionArgument {
    pub ftype: FType,
    pub ftype_ref: usize,
    pub name: String,
}

impl OuterFunctionArgument {
    pub fn new(name: &str, ftype: FType, ftype_ref: usize) -> Self {
        Self {
            ftype,
            ftype_ref,
            name: name.to_string(),
        }
    }
}

pub struct ImplementedOuterFunction<S> {
    pub name: String,
    pub arguments: Vec<OuterFunctionArgument>,
    pub return_type: FType,
    pub return_type_ref: usize,
    pub func: Box<dyn FnMut(Vec<PCell>, &mut S) -> Option<PCell>>,
}

#[derive(Default)]
pub struct ImplementedOuterFunctionBuilder<S> {
    pub functions: Vec<ImplementedOuterFunction<S>>,
}

impl<S> ImplementedOuterFunctionBuilder<S> {
    pub fn add(self, name: &str, arguments: &[OuterFunctionArgument], return_type: Option<(FType, usize)>, func: Box<dyn FnMut(Vec<PCell>, &mut S) -> Option<PCell>>) -> Self {
        let mut functions = self.functions;
        functions.push(ImplementedOuterFunction {
            name: name.to_string(),
            arguments: arguments.to_vec(),
            return_type: if let Some((rettype, _)) = &return_type { *rettype } else { FType::Void },
            return_type_ref: if let Some((_, refcount)) = &return_type { *refcount } else { 0 },
            func,
        });
        Self {
            functions,
        }
    }
    
    pub fn build(self) -> (Vec<(String, OuterFunction)>, BTreeMap<String, RTOuterFunction<S>>) {
        let compile_of = self.functions.iter().map(|v| (v.name.clone(), OuterFunction {
            label: v.name.clone(),
            args: v.arguments.iter().map(|v| {
                FElmVarDef {
                    ftype: v.ftype,
                    ftype_ref: v.ftype_ref,
                    name: v.name.clone(),
                    assign: None,
                }
            }).collect(),
            return_type: v.return_type,
            return_type_ref: v.return_type_ref,
        })).collect();
        let vm_of = self.functions.into_iter().map(|v| (v.name, RTOuterFunction {
            argument_count: v.arguments.len(),
            returns_value: if v.return_type == FType::Void && v.return_type_ref == 0 { false } else { true },
            f: v.func,
        })).collect();

        (compile_of, vm_of)
    }
}