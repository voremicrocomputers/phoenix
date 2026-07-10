use crate::compile::OuterFunction;
use crate::tree::{FElmVarDef, FType};
use crate::vm::{PCell, PhoenixVMState, RTOuterFunction};
use std::collections::BTreeMap;

pub mod compile;
pub mod phoenixarch;
pub mod tokenize;
pub mod tree;
pub mod vm;

fn main() {
    let input = std::fs::read_to_string("test1.fnx").expect("failed to read input file");
    let tokens = tokenize::tokenize(input).expect("failed to tokenize");
    let tree = tree::make_tree(tokens).expect("failed to make tree");
    let program = compile::compile(
        tree,
        vec![
            (
                "puts".to_string(),
                OuterFunction {
                    label: "puts".to_string(),
                    args: vec![FElmVarDef {
                        ftype: FType::Char,
                        ftype_ref: 1,
                        name: "string".to_string(),
                        assign: None,
                    }],
                    return_type: FType::Void,
                    return_type_ref: 0,
                },
            ),
            (
                "get_string".to_string(),
                OuterFunction {
                    label: "get_string".to_string(),
                    args: vec![],
                    return_type: FType::Char,
                    return_type_ref: 1,
                },
            ),
        ],
    )
    .expect("failed to compile program");
    println!("{:#?}", program);

    let mut puts_impl =
        RTOuterFunction {
            argument_count: 1,
            returns_value: false,
            f: Box::new(|args| {
                if let PCell::String(str) = &args[0] {
                    println!("{}", str);
                } else {
                    eprintln!("BAD ARGUMENT TO puts");
                }

                None
            }),
        };
    let mut get_string_impl =
        RTOuterFunction {
            argument_count: 0,
            returns_value: true,
            f: Box::new(|_| Some(PCell::String("test string".to_string()))),
        };

    let mut outer_functions = BTreeMap::new();
    outer_functions.insert(
        "puts".to_string(),
        &mut puts_impl,
    );
    outer_functions.insert(
        "get_string".to_string(),
        &mut get_string_impl,
    );

    let mut vm = PhoenixVMState::new(outer_functions, &program);

    vm.execute_function("main").expect("failed to execute main");
}
