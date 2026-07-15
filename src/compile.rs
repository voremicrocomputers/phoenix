use std::collections::BTreeMap;
use crate::phoenixarch::{Instruction, Program};
use crate::tree::{FElm, FElmCall, FElmClosure, FElmData, FElmFunction, FElmIfStatement, FElmVarAssign, FElmVarDef, FType, Operator};

pub struct VariableState {
    pub ftype: FType,
    pub ftype_ref: usize,
    pub stack_idx: usize,
    // at any given time, argument stack position is TOP - (state.stack_idx - var.stack_idx)
    pub is_argument: bool,
}

pub struct CompiledFunction {
    pub label: String,
    pub id: u64,
    pub toplevel: bool,
    pub args: Vec<FElmVarDef>, // doesn't use assigns
    pub return_type: FType,
    pub return_type_ref: usize,
    pub instructions: Vec<Instruction>,
}

pub struct OuterFunction {
    pub label: String,
    pub args: Vec<FElmVarDef>, // doesn't use assigns
    pub return_type: FType,
    pub return_type_ref: usize,
}

pub struct CompilerState {
    pub line: usize,
    pub character: usize,

    pub string_table: Vec<String>,
    pub outer_function_table: Vec<(String, OuterFunction)>,
    pub unique_function_lookup: BTreeMap<usize, String>,
    pub scope_functions: Vec<(String, usize, (FType, usize))>,
    pub variables: Vec<(String, VariableState)>,
    pub stack_idx: usize,
}

#[derive(Debug)]
pub enum CompileErrorType {
    TopLevelTreeElementIsNotCorrect,
    InternalCompilerStackCorruption,
    UnexpectedElementInClosure,
    UnsupportedElementInExpression,
    UnsupportedElementInIfStatement,
    InternalCompilerStringTableError,
    VariableNotFound(String),
    ExpressionIsNotOfExpectedType((FType, usize), (FType, usize)), // (expected, actual)
    IncorrectArgumentCount(usize, usize), // (expected, actual)
    InvalidUnaryOperator,
    InvalidBinaryOperator,
    InvalidExpressionInBinaryOperation,
    BinaryExpressionsAreNotOfEqualType,
    TooManyDereferences,
    PointerArithmeticNotCurrentlySupported,
    NumberOutOfTypeRange,
}

#[derive(Debug)]
pub struct CompileError {
    pub error_type: CompileErrorType,
    pub line: usize,
    pub character: usize,
}

fn unique_function_name(label: &str, id: usize) -> String {
    format!("{}_{}", label, id)
}

fn evaluate_constant_strings(elm: &FElm) -> Result<Vec<String>, CompileError> {
    match &elm.data {
        FElmData::Function(func) => {
            evaluate_constant_strings(&func.body)
        }
        FElmData::Closure(closure) => {
            let mut strings = vec![];
            for instruction in &closure.instructions {
                strings.extend(evaluate_constant_strings(instruction)?);
            }
            Ok(strings)
        }
        FElmData::Call(call) => {
            let mut strings = vec![];
            for arg in &call.args {
                strings.extend(evaluate_constant_strings(arg)?);
            }
            Ok(strings)
        }
        FElmData::BooleanLiteral(_) => {
            Ok(vec![])
        }
        FElmData::NumberLiteral(_) => {
            Ok(vec![])
        }
        FElmData::StringLiteral(str) => {
            Ok(vec![str.value.clone()])
        }
        FElmData::CharLiteral(_) => {
            Ok(vec![])
        }
        FElmData::ExternFunction(_) => {
            Ok(vec![])
        }
        FElmData::VarRef(_) => {
            Ok(vec![])
        }
        FElmData::VarDef(def) => {
            if let Some(assign) = &def.assign {
                evaluate_constant_strings(assign)
            } else {
                Ok(vec![])
            }
        }
        FElmData::UnaryExpression(unary) => {
            evaluate_constant_strings(&unary.alpha)
        }
        FElmData::BinaryExpression(binary) => {
            let mut strings = vec![];
            strings.extend(evaluate_constant_strings(&binary.alpha)?);
            strings.extend(evaluate_constant_strings(&binary.beta)?);
            Ok(strings)
        }
        FElmData::IfStatement(ifst) => {
            let mut strings = vec![];

            strings.extend(evaluate_constant_strings(&ifst.condition)?);
            strings.extend(evaluate_constant_strings(&ifst.body)?);
            if let Some(otherwise) = &ifst.otherwise {
                strings.extend(evaluate_constant_strings(otherwise)?);
            }

            Ok(strings)
        }
        FElmData::VarAssign(vas) => {
            evaluate_constant_strings(&vas.assign)
        }
    }
}

fn evaluate_type(state: &mut CompilerState, elm: &FElm) -> Option<(FType, usize)> {
    match &elm.data {
        FElmData::Call(call) => {
            if let Some((_, _, ftype)) = state.scope_functions.iter().rfind(|(name, _, _)| name == &call.label) {
                Some(*ftype)
            } else if let Some((_, (_, outer_func))) = state.outer_function_table.iter().enumerate().find(|(_, (name, _))| name == &call.label) {
                Some((outer_func.return_type, outer_func.return_type_ref))
            } else {
                None
            }
        }
        FElmData::BooleanLiteral(_) => {
            Some((FType::Boolean, 0))
        }
        FElmData::NumberLiteral(_) => {
            Some((FType::U32, 0))
        }
        FElmData::StringLiteral(_) => {
            Some((FType::String, 0))
        }
        FElmData::CharLiteral(_) => {
            Some((FType::Char, 0))
        }
        FElmData::VarRef(var) => {
            let variable_idx = state.variables.iter().rposition(|v| v.0 == var.value)?;
            let (_, variable) = state.variables.get(variable_idx).unwrap();
            Some((variable.ftype, variable.ftype_ref + var.ref_count - var.deref_count))
        }
        FElmData::UnaryExpression(felm) => {
            evaluate_type(state, &felm.alpha)
        }
        FElmData::BinaryExpression(felm) => {
            evaluate_type(state, &felm.alpha)
        }

        _ => None,
    }
}

/// THIS SHOULD ALWAYS LEAVE STACK IDX AS ORIGINAL STACK IDX + 1
fn compile_expression(
    state: &mut CompilerState,
    functions: &mut Vec<(String, CompiledFunction)>,
    elm: &FElm,
    expected_type: FType,
    expected_typeref: usize,
) -> Result<Vec<Instruction>, CompileError> {
    let mut instructions = vec![];
    state.line = elm.line;
    state.character = elm.character;

    let original_stack_idx = state.stack_idx;

    match &elm.data {
        FElmData::BooleanLiteral(v) => {
            if !(expected_type == FType::Boolean && expected_typeref == 0) {
                return Err(CompileError {
                    error_type: CompileErrorType::ExpressionIsNotOfExpectedType((expected_type, expected_typeref), (FType::Boolean, 0)),
                    line: state.line,
                    character: state.character,
                })
            }

            instructions.push(Instruction::ConstBoolean(v.value));
            state.stack_idx += 1;
        }
        FElmData::NumberLiteral(num) => {
            if expected_typeref > 0 {
                // supposed to be a pointer i guess? let's not support this for now
                return Err(CompileError {
                    error_type: CompileErrorType::PointerArithmeticNotCurrentlySupported,
                    line: state.line,
                    character: state.character,
                });
            }
            match expected_type {
                FType::U32 => {
                    let num = if let Some(num) = u32::try_from(num.value).ok() {
                        num
                    } else {
                        return Err(CompileError {
                            error_type: CompileErrorType::NumberOutOfTypeRange,
                            line: state.line,
                            character: state.character,
                        });
                    };
                    instructions.push(Instruction::ConstU32(num));
                    state.stack_idx += 1;
                }
                _ => {
                    return Err(CompileError {
                        error_type: CompileErrorType::ExpressionIsNotOfExpectedType((expected_type, expected_typeref), (FType::U32, 0)),
                        line: state.line,
                        character: state.character,
                    });
                }
            }
        }
        FElmData::StringLiteral(str) => {
            if !(expected_type == FType::String && expected_typeref == 0) {
                return Err(CompileError {
                    error_type: CompileErrorType::ExpressionIsNotOfExpectedType((expected_type, expected_typeref), (FType::String, 0)),
                    line: state.line,
                    character: state.character,
                })
            }
            let idx = state.string_table.iter().position(|v| v == &str.value)
                .ok_or(CompileError {
                    error_type: CompileErrorType::InternalCompilerStringTableError,
                    line: state.line,
                    character: state.character,
                })?;
            instructions.push(Instruction::ConstString(idx as u16));
            state.stack_idx += 1;
        }
        FElmData::CharLiteral(_) => { todo!() }
        FElmData::VarRef(var) => {
            let variable_idx = state.variables.iter().rposition(|v| v.0 == var.value)
                .ok_or(CompileError {
                    error_type: CompileErrorType::VariableNotFound(var.value.clone()),
                    line: state.line,
                    character: state.character,
                })?;
            let (_, variable) = state.variables.get(variable_idx).unwrap();
            if !(expected_type == variable.ftype && expected_typeref == variable.ftype_ref + var.ref_count - var.deref_count) {
                return Err(CompileError {
                    error_type: CompileErrorType::ExpressionIsNotOfExpectedType((expected_type, expected_typeref), (variable.ftype, variable.ftype_ref)),
                    line: state.line,
                    character: state.character,
                });
            }

            // get the stack pointer at the beginning of the context
            instructions.push(Instruction::PushStackPointer);
            instructions.push(Instruction::ConstU32(state.stack_idx as u32));
            state.stack_idx += 2;
            instructions.push(Instruction::SubU32); // (current stack pointer - how much we've increased the stack) = original stack pointer
            state.stack_idx -= 1;

            // get the variable pointer in "local" pointerspace
            instructions.push(Instruction::ConstU32(variable.stack_idx as u32));
            state.stack_idx += 1;

            // add the two to get the pointer in "global" pointerspace
            instructions.push(Instruction::AddU32);
            state.stack_idx -= 1;

            // if we don't want to reference this, then bring the value
            if var.ref_count == 0 {
                instructions.push(Instruction::RotateDynamic);
            }

            // if we want to dereference, then rotate (maybe again)
            if var.deref_count > 0 {
                instructions.push(Instruction::RotateDynamic);
            }
        }
        FElmData::Call(call) => {
            let (ins, has_return_value) = compile_call(state, functions, call, Some((expected_type, expected_typeref)))?;
            assert!(has_return_value);
            instructions.extend(ins);
        }
        FElmData::UnaryExpression(una) => {
            if !(expected_type == FType::Boolean && expected_typeref == 0) {
                return Err(CompileError {
                    error_type: CompileErrorType::ExpressionIsNotOfExpectedType((expected_type, expected_typeref), (FType::Boolean, 0)),
                    line: state.line,
                    character: state.character,
                })
            }

            if una.operator == Operator::Not {
                instructions.extend(compile_expression(state, functions, &una.alpha, expected_type, expected_typeref)?);
                instructions.push(Instruction::BooleanNot);
            } else {
                return Err(CompileError {
                    error_type: CompileErrorType::InvalidUnaryOperator,
                    line: state.line,
                    character: state.character,
                })
            }
        }
        FElmData::BinaryExpression(bina) => {
            let first_type = evaluate_type(state, &bina.alpha).ok_or(CompileError {
                error_type: CompileErrorType::InvalidExpressionInBinaryOperation,
                line: state.line,
                character: state.character,
            })?;
            let second_type = evaluate_type(state, &bina.beta).ok_or(CompileError {
                error_type: CompileErrorType::InvalidExpressionInBinaryOperation,
                line: state.line,
                character: state.character,
            })?;
            if first_type != second_type {
                return Err(CompileError {
                    error_type: CompileErrorType::BinaryExpressionsAreNotOfEqualType,
                    line: state.line,
                    character: state.character,
                });
            }
            match bina.operator {
                Operator::EQ | Operator::NotEQ => {
                    instructions.extend(compile_expression(state, functions, &bina.alpha, first_type.0, first_type.1)?);
                    instructions.extend(compile_expression(state, functions, &bina.beta, first_type.0, first_type.1)?);
                    instructions.push(Instruction::CompareEqual);
                    state.stack_idx -= 2;
                    state.stack_idx += 1;
                    if bina.operator == Operator::NotEQ {
                        instructions.push(Instruction::BooleanNot);
                    }
                }
                Operator::Add => {
                    match first_type.0 {
                        FType::U32 => {
                            instructions.extend(compile_expression(state, functions, &bina.alpha, first_type.0, first_type.1)?);
                            instructions.extend(compile_expression(state, functions, &bina.beta, first_type.0, first_type.1)?);
                            instructions.push(Instruction::AddU32);
                            state.stack_idx -= 2;
                            state.stack_idx += 1;
                        }
                        _ => {
                            return Err(CompileError {
                                error_type: CompileErrorType::ExpressionIsNotOfExpectedType((expected_type, expected_typeref), (FType::U32, 0)),
                                line: state.line,
                                character: state.character,
                            });
                        }
                    }
                }
                Operator::Sub => {
                    match first_type.0 {
                        FType::U32 => {
                            instructions.extend(compile_expression(state, functions, &bina.alpha, first_type.0, first_type.1)?);
                            instructions.extend(compile_expression(state, functions, &bina.beta, first_type.0, first_type.1)?);
                            instructions.push(Instruction::SubU32);
                            state.stack_idx -= 2;
                            state.stack_idx += 1;
                        }
                        _ => {
                            return Err(CompileError {
                                error_type: CompileErrorType::ExpressionIsNotOfExpectedType((expected_type, expected_typeref), (FType::U32, 0)),
                                line: state.line,
                                character: state.character,
                            });
                        }
                    }
                }
                /*
                Operator::Mul => {}
                Operator::Div => {}
                Operator::Mod => {}
                Operator::BitwiseAnd => {}
                Operator::BooleanAnd => {}
                Operator::BitwiseOr => {}
                Operator::BooleanOr => {}
                Operator::BitwiseXor => {}
                Operator::GreaterThan => {}
                Operator::LessThan => {}
                Operator::GreaterThanEqual => {}
                Operator::LessThanEqual => {}
                 */

                Operator::Not => {
                    return Err(CompileError {
                        error_type: CompileErrorType::InvalidBinaryOperator,
                        line: state.line,
                        character: state.character,
                    })
                }

                _ => { todo!() }
            }
        }

        _ => {
            return Err(CompileError {
                error_type: CompileErrorType::UnsupportedElementInExpression,
                line: state.line,
                character: state.character,
            });
        }
    }

    assert_eq!(state.stack_idx, original_stack_idx + 1);

    Ok(instructions)
}

fn compile_if_statement(
    state: &mut CompilerState,
    functions: &mut Vec<(String, CompiledFunction)>,
    elm: &FElmIfStatement,
) -> Result<Vec<Instruction>, CompileError> {
    let mut instructions = vec![];

    let osi = state.stack_idx;

    instructions.extend(compile_expression(state, functions, &elm.condition, FType::Boolean, 0)?);

    let add_one_to_branch = if let Some(otherwise) = &elm.otherwise {
        matches!(&otherwise.data, FElmData::Closure(_) | FElmData::IfStatement(_))
    } else {
        false
    };

    if let FElmData::Closure(clos) = &elm.body.data {
        // if the boolean above is true, we want to run this code, otherwise we want to skip forward
        state.stack_idx -= 1; // needs to be done here since the closure needs to know that the stack went down from the relbranch
        let closure_instructions = compile_closure(state, functions, clos)?;
        instructions.push(Instruction::RelativeBranch(closure_instructions.len() as i32 + if add_one_to_branch { 1 } else { 0 })); // skips forward if boolean is false
        instructions.extend(closure_instructions);
    } else {
        return Err(CompileError {
            error_type: CompileErrorType::UnsupportedElementInIfStatement,
            line: state.line,
            character: state.character,
        });
    }

    if let Some(otherwise) = &elm.otherwise {
        match &otherwise.data {
            FElmData::IfStatement(ifst) => {
                let otherwise_instructions = compile_if_statement(state, functions, ifst)?;
                instructions.push(Instruction::Jump(otherwise_instructions.len() as i32));
                // this is now where we will land after the first relativebranch, if the if statement was false
                instructions.extend(otherwise_instructions);
            }
            FElmData::Closure(clos) => {
                // only run this code if the original code didn't execute
                // we can assert that add_one_to_branch is true, and as such,
                // if the original closure ran, we will end up before the 1 additional instruction
                // we will use that extra instruction to jump past the else code
                let otherwise_instructions = compile_closure(state, functions, clos)?;
                instructions.push(Instruction::Jump(otherwise_instructions.len() as i32));
                // this is now where we will land after the first relativebranch, if the if statement was false
                instructions.extend(otherwise_instructions);
            }
            _ => {
                return Err(CompileError {
                    error_type: CompileErrorType::UnsupportedElementInIfStatement,
                    line: state.line,
                    character: state.character,
                });
            }
        }
    }

    assert_eq!(state.stack_idx, osi);

    Ok(instructions)
}

fn compile_vardef(
    state: &mut CompilerState,
    functions: &mut Vec<(String, CompiledFunction)>,
    elm: &FElmVarDef,
) -> Result<Vec<Instruction>, CompileError> {
    let mut instructions = vec![];

    // add the variable to the state
    state.variables.push((elm.name.clone(), VariableState {
        ftype: elm.ftype,
        ftype_ref: elm.ftype_ref,
        stack_idx: state.stack_idx,
        is_argument: false,
    }));

    if let Some(assign) = &elm.assign {
        instructions.extend(compile_expression(state, functions, assign, elm.ftype, elm.ftype_ref)?);
    } else {
        instructions.push(Instruction::PushEmpty);
        state.stack_idx += 1;
    }

    Ok(instructions)
}

fn compile_varassign(
    state: &mut CompilerState,
    functions: &mut Vec<(String, CompiledFunction)>,
    elm: &FElmVarAssign,
) -> Result<Vec<Instruction>, CompileError> {
    let mut instructions = vec![];

    let osi = state.stack_idx;

    // find the variable
    let (_, var) = state.variables.iter_mut().rfind(|(v, _)| v == &elm.name)
        .ok_or(CompileError {
            error_type: CompileErrorType::VariableNotFound(elm.name.clone()),
            line: state.line,
            character: state.character,
        })?;
    if elm.deref_count > var.ftype_ref {
        return Err(CompileError {
            error_type: CompileErrorType::TooManyDereferences,
            line: state.line,
            character: state.character,
        });
    }
    let ftype = var.ftype;
    let ftype_ref = var.ftype_ref - elm.deref_count;
    let stack_idx = var.stack_idx;

    instructions.extend(compile_expression(state, functions, &elm.assign, ftype, ftype_ref)?);

    // get the context pointer
    instructions.push(Instruction::PushStackPointer);
    instructions.push(Instruction::ConstU32(state.stack_idx as u32));
    state.stack_idx += 2;
    instructions.push(Instruction::SubU32);
    state.stack_idx -= 1;

    // get the variable pointer in "local" pointerspace
    instructions.push(Instruction::ConstU32(stack_idx as u32)); // bring the variable to the top
    state.stack_idx += 1;

    // add the two to get the pointer in "global" pointerspace
    instructions.push(Instruction::AddU32);
    state.stack_idx -= 1;

    // we need to rotate to get the actual pointer
    if elm.deref_count > 0 {
        instructions.push(Instruction::RotateDynamic); // copy the original pointer
    }
    instructions.push(Instruction::ExchangeDynamic);
    state.stack_idx -= 2;

    assert_eq!(osi, state.stack_idx);

    Ok(instructions)
}

/// boolean indicates if this call returns something
fn compile_call(
    state: &mut CompilerState,
    functions: &mut Vec<(String, CompiledFunction)>,
    elm: &FElmCall,
    expected_type: Option<(FType, usize)>,
) -> Result<(Vec<Instruction>, bool), CompileError> {
    let mut instructions = vec![];
    let mut has_return_value = false;
    let original_stack_idx = state.stack_idx;

    if let Some((label, elm_id, ftype)) = state.scope_functions.iter().rfind(|(name, _, _)| name == &elm.label) {
        // get true label
        let unique_label = unique_function_name(label, *elm_id);
        let (idx, (_, func)) = functions.iter().enumerate().find(|(_, (v, _))| v == &unique_label).ok_or(CompileError {
            error_type: CompileErrorType::VariableNotFound(unique_label.clone()),
            line: state.line,
            character: state.character,
        })?;
        // verify arguments
        if let Some((expected_type, expected_typeref)) = expected_type {
            if !(expected_type == ftype.0 && expected_typeref == ftype.1) {
                return Err(CompileError {
                    error_type: CompileErrorType::ExpressionIsNotOfExpectedType((expected_type, expected_typeref), (ftype.0, ftype.1)),
                    line: state.line,
                    character: state.character,
                })
            }
        }
        if ftype.0 != FType::Void {
            has_return_value = true;
        }
        let types = func.args.iter().map(|v| (v.ftype, v.ftype_ref)).collect::<Vec<_>>();
        for i in 0..elm.args.len() {
            let ftype = types[i].0;
            let ftype_ref = types[i].1;
            let alpha = &elm.args[i];
            instructions.extend(compile_expression(state, functions, alpha, ftype, ftype_ref)?);
        }
        assert_eq!(state.stack_idx, original_stack_idx + elm.args.len());
        instructions.push(Instruction::ConstU32(idx as u32));
        instructions.push(Instruction::Call);
        if has_return_value {
            // one extra stack element for return value
            state.stack_idx += 1;
            for _ in 0..elm.args.len() {
                instructions.push(Instruction::Swap);
                instructions.push(Instruction::Drop);
                state.stack_idx -= 1;
            }
        } else {
            for _ in 0..elm.args.len() {
                instructions.push(Instruction::Drop);
                state.stack_idx -= 1;
            }
        }

    } else if let Some((idx, (_, outer_func))) = state.outer_function_table.iter().enumerate().find(|(_, (name, _))| name == &elm.label) {
        // verify type
        if let Some((expected_type, expected_typeref)) = expected_type {
            if !(expected_type == outer_func.return_type && expected_typeref == outer_func.return_type_ref) {
                return Err(CompileError {
                    error_type: CompileErrorType::ExpressionIsNotOfExpectedType((expected_type, expected_typeref), (outer_func.return_type, outer_func.return_type_ref)),
                    line: state.line,
                    character: state.character,
                })
            }
        }
        // verify arguments
        if elm.args.len() != outer_func.args.len() {
            return Err(CompileError {
                error_type: CompileErrorType::IncorrectArgumentCount(outer_func.args.len(), elm.args.len()),
                line: state.line,
                character: state.character,
            });
        }
        if outer_func.return_type != FType::Void {
            has_return_value = true;
        }
        let types = outer_func.args.iter().map(|v| (v.ftype, v.ftype_ref)).collect::<Vec<_>>();
        for i in 0..elm.args.len() {
            let ftype = types[i].0;
            let ftype_ref = types[i].1;
            let alpha = &elm.args[i];
            instructions.extend(compile_expression(state, functions, alpha, ftype, ftype_ref)?);
        }
        assert_eq!(state.stack_idx, original_stack_idx + elm.args.len());
        instructions.push(Instruction::CallOuter(idx as u16));
        state.stack_idx -= elm.args.len(); // all arguments were consumed
        if has_return_value {
            // one extra stack element for return value
            state.stack_idx += 1;
        }
    }

    Ok((instructions, has_return_value))
}

fn compile_closure(
    state: &mut CompilerState,
    functions: &mut Vec<(String, CompiledFunction)>,
    elm: &FElmClosure,
) -> Result<Vec<Instruction>, CompileError> {
    let mut instructions = vec![];

    let original_stack_idx = state.stack_idx;

    // evaluate scope functions
    let scope_function_index = state.scope_functions.len();
    for elm in &elm.instructions {
        if let FElmData::Function(func) = &elm.data {
            state.scope_functions.push((func.label.clone(), elm.id, (func.return_type, func.return_type_ref)));
            compile_function(state, functions, func, elm.id, false)?;
        }
    }

    let variable_state_index = state.variables.len();

    for elm in &elm.instructions {
        state.line = elm.line;
        state.character = elm.character;
        match &elm.data {
            FElmData::Function(_) => {
                // already done
            }
            FElmData::Closure(clos) => {
                instructions.extend(compile_closure(state, functions, clos,)?);
            }
            FElmData::VarDef(vardef) => {
                instructions.extend(compile_vardef(state, functions, vardef)?);
            }
            FElmData::Call(call) => {
                let osi = state.stack_idx;
                let (ins, has_return_value) = compile_call(state, functions, call, None)?;
                instructions.extend(ins);
                if has_return_value {
                    assert_eq!(state.stack_idx, osi + 1);
                    instructions.push(Instruction::Drop); // we don't want the return value
                    state.stack_idx -= 1;
                } else {
                    assert_eq!(state.stack_idx, osi);
                }
            }
            FElmData::IfStatement(ifst) => {
                instructions.extend(compile_if_statement(state, functions, ifst)?);
            }
            FElmData::VarAssign(vas) => {
                instructions.extend(compile_varassign(state, functions, vas)?);
            }
            /*
            FElmData::NumberLiteral(_) => {}
            FElmData::StringLiteral(_) => {}
            FElmData::CharLiteral(_) => {}
            FElmData::ExternFunction(_) => {}
            FElmData::VarRef(_) => {}
            FElmData::UnaryExpression(_) => {}
            FElmData::BinaryExpression(_) => {}
             */
            _ => {
                return Err(CompileError {
                    error_type: CompileErrorType::UnexpectedElementInClosure,
                    line: state.line,
                    character: state.character,
                });
            }
        }
    }

    // remove scope functions & variables
    let _ = state.scope_functions.split_off(scope_function_index);
    let _ = state.variables.split_off(variable_state_index);
    // drop all added stack elements
    // todo: this will need to be changed to support function returns
    if state.stack_idx > original_stack_idx {
        for _ in 0..(state.stack_idx - original_stack_idx) {
            instructions.push(Instruction::Drop);
            state.stack_idx -= 1;
        }
    } else if state.stack_idx < original_stack_idx {
        return Err(CompileError {
            error_type: CompileErrorType::InternalCompilerStackCorruption,
            line: state.line,
            character: state.character,
        });
    }

    assert_eq!(state.stack_idx, original_stack_idx);

    Ok(instructions)
}

fn compile_function(
    state: &mut CompilerState,
    functions: &mut Vec<(String, CompiledFunction)>,
    elm: &FElmFunction,
    elm_id: usize,
    toplevel: bool,
) -> Result<(), CompileError> {
    let unique_label = unique_function_name(&elm.label, elm_id);

    let osi = state.stack_idx;
    let variable_state_index = state.variables.len();
    state.stack_idx = 0;

    let mut prelude_instructions = vec![];

    for arg in elm.args.iter() {
        state.variables.push((arg.name.clone(), VariableState {
            ftype: arg.ftype,
            ftype_ref: arg.ftype_ref,
            stack_idx: state.stack_idx,
            is_argument: true,
        }));
        state.stack_idx += 1;
    }

    let func = CompiledFunction {
        label: elm.label.clone(),
        id: elm_id as u64,
        toplevel,
        args: elm.args.iter().map(|v| FElmVarDef {
            ftype: v.ftype,
            ftype_ref: v.ftype_ref,
            name: v.name.clone(),
            assign: None,
        }).collect(),
        return_type: elm.return_type,
        return_type_ref: elm.return_type_ref,
        instructions: if let FElmData::Closure(clos) = &elm.body.data {
            state.line = elm.body.line;
            state.character = elm.body.character;
            let instructions = compile_closure(state, functions, clos)?;
            prelude_instructions.extend(instructions);
            prelude_instructions
        } else {
            return Err(CompileError {
                error_type: CompileErrorType::UnexpectedElementInClosure,
                line: state.line,
                character: state.character,
            });
        },
    };
    functions.push((unique_label, func));

    let _ = state.variables.split_off(variable_state_index);

    state.stack_idx = osi;

    Ok(())
}

pub fn compile(tree: FElm, implemented_outer_functions: Vec<(String, OuterFunction)>) -> Result<Program, CompileError> {
    let mut state = CompilerState {
        line: 0,
        character: 0,
        string_table: evaluate_constant_strings(&tree)?,
        outer_function_table: implemented_outer_functions,
        unique_function_lookup: Default::default(),
        scope_functions: vec![],
        variables: Default::default(),
        stack_idx: 0,
    };

    let mut functions: Vec<(String, CompiledFunction)> = Vec::new();

    match &tree.data {
        FElmData::Closure(closure) => {
            // evaluate scope functions
            for elm in &closure.instructions {
                if let FElmData::Function(func) = &elm.data {
                    state.scope_functions.push((func.label.clone(), elm.id, (func.return_type, func.return_type_ref)));
                }
            }

            // add closure elements
            for elm in &closure.instructions {
                match &elm.data {
                    FElmData::Function(func) => {
                        compile_function(&mut state, &mut functions, func, elm.id, true)?;
                    }
                    _ => {
                        return Err(CompileError {
                            error_type: CompileErrorType::TopLevelTreeElementIsNotCorrect,
                            line: elm.line,
                            character: elm.character,
                        });
                    }
                }
            }
        }
        _ => {
            return Err(CompileError {
                error_type: CompileErrorType::TopLevelTreeElementIsNotCorrect,
                line: 0,
                character: 0,
            });
        }
    }

    let functions = functions.into_iter().collect::<Vec<_>>();

    Ok(Program {
        string_table: state.string_table,
        outer_function_table: state.outer_function_table.iter().map(|v| v.0.clone()).collect(),
        toplevel_function_table: functions.iter().enumerate().filter_map(|v| if v.1.1.toplevel { Some((v.1.1.label.clone(), v.0)) } else { None }).collect(),
        functions: functions.into_iter().map(|v| v.1.instructions).collect(),
    })
}