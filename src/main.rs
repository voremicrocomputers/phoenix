use phoenix::compile::OuterFunction;
use phoenix::tree::{FElmVarDef, FType};
use phoenix::vm::{PCell, PhoenixVMState, RTOuterFunction};
use std::collections::BTreeMap;
use phoenix::{compile, tokenize, tree, ImplementedOuterFunctionBuilder, OuterFunctionArgument};
use phoenix::phoenixarch::Program;


fn main() {
    let input = std::fs::read_to_string("test1.fnx").expect("failed to read input file");
    let tokens = tokenize::tokenize(input).expect("failed to tokenize");
    let tree = tree::make_tree(tokens).expect("failed to make tree");
    
    let mut state = 0;

    let (compile_of, vm_of) = ImplementedOuterFunctionBuilder::default()
        .add(
            "puts",
            &[OuterFunctionArgument::new("string", FType::String, 0)],
            None,
            Box::new(
                |args, state| {
                    if let Some(PCell::String(str)) = args.get(0) {
                        println!("{}", str);
                    } else {
                        eprintln!("BAD ARGS TO puts");
                    }
                    None
                }
            )
        )
        .add(
            "get_string",
            &[],
            Some((FType::String, 0)),
            Box::new(
                |_, _| {
                    Some(PCell::String("test string".to_string()))
                }
            )
        )
        .add(
            "state_test",
            &[],
            None,
            Box::new(
                |_, state: &mut i32| {
                    println!("state has been called {} times previously", *state);
                    *state += 1;
                    
                    None
                }
            )
        )
        .build();

    let program = compile::compile(
        tree,
        compile_of
    )
    .expect("failed to compile program");
    println!("{:#?}", program);

    let program = program.bytecode();
    println!("program is {} bytes long", program.len());
    let program = Program::from_bytecode(&program).expect("failed to deserialize program bytecode");

    let mut vm = PhoenixVMState::new(vm_of, &program);

    vm.execute_function("main", &mut state).expect("failed to execute main");
}
