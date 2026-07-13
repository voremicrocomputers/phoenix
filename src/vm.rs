use std::collections::BTreeMap;
use crate::phoenixarch::{Instruction, Program};

pub struct RTOuterFunction<S> {
    pub argument_count: usize,
    pub returns_value: bool,
    pub f: Box<dyn FnMut(Vec<PCell>, &mut S) -> Option<PCell>>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum PCell {
    Blank,
    String(String),
    Boolean(bool),
    U32(u32),
}

impl PCell {
    pub fn take_string(&mut self) -> Option<String> {
        let mut alpha = PCell::Blank;
        std::mem::swap(self, &mut alpha);
        if let PCell::String(str) = alpha {
            Some(str)
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub enum VMError {
    StackEmpty,
    FunctionNotFound(String),
    OuterFuncIDNotFound(u16),
    OuterFuncDidNotReturnValue(u16),
    ExpectedStackElementOfType(&'static str, PCell), // (expected, found)
}

pub struct FunctionContext {
    pub pc: usize,
    pub function: usize,
}

pub struct PhoenixVMState<'a, S> {
    pub stack: Vec<PCell>,
    pub function_stack: Vec<FunctionContext>,
    pub outer_functions: BTreeMap<String, RTOuterFunction<S>>,
    pub program: &'a Program,
}

impl<'a, S> PhoenixVMState<'a, S> {
    pub fn new(outer_functions: BTreeMap<String, RTOuterFunction<S>>, program: &'a Program) -> PhoenixVMState<'a, S> {
        PhoenixVMState {
            stack: vec![],
            function_stack: vec![],
            outer_functions,
            program,
        }
    }

    pub fn swap_program(&mut self, new_program: &'a Program) {
        self.stack.clear();
        self.function_stack.clear();
        self.program = new_program;
    }

    pub fn execute_function(&mut self, func: &str, state: &mut S) -> Result<(), VMError> {
        if let Some(idx) = self.program.toplevel_function_table.iter().find(|v| v.0 == func).map(|v| v.1).clone() {
            self.function_stack.push(FunctionContext {
                pc: 0,
                function: idx,
            });
            self.execute(state)?;
            Ok(())
        } else {
            Err(VMError::FunctionNotFound(func.to_string()))
        }
    }

    pub fn execute(&mut self, state: &mut S) -> Result<(), VMError> {
        loop {
            let instruction = if let Some(ctx) = self.function_stack.last_mut() {
                let instruction = self.program.functions[ctx.function].get(ctx.pc);
                if let Some(instruction) = instruction {
                    ctx.pc += 1;
                    *instruction
                } else {
                    break;
                }
            } else {
                break;
            };

            self.execute_instruction(instruction, state)?;
        }
        Ok(())
    }

    pub fn execute_instruction(&mut self, ins: Instruction, state: &mut S) -> Result<(), VMError> {
        match ins {
            Instruction::Nop => {}
            Instruction::Drop => {
                let _ = self.stack.pop().ok_or(VMError::StackEmpty);
            }
            Instruction::Dup => {
                let top = self.stack.pop().ok_or(VMError::StackEmpty)?;
                self.stack.push(top.clone());
                self.stack.push(top);
            }
            Instruction::Swap => {
                let alpha = self.stack.pop().ok_or(VMError::StackEmpty)?;
                let beta = self.stack.pop().ok_or(VMError::StackEmpty)?;
                self.stack.push(alpha);
                self.stack.push(beta);
            }
            Instruction::PushEmpty => {
                self.stack.push(PCell::Blank);
            }
            Instruction::RelativeBranch(pcoff) => {
                let alpha = self.stack.pop().ok_or(VMError::StackEmpty)?;
                if let PCell::Boolean(v) = alpha {
                    if !v {
                        if let Some(ctx) = self.function_stack.last_mut() {
                            if pcoff.is_positive() {
                                ctx.pc += pcoff as usize;
                            } else {
                                ctx.pc -= pcoff.abs() as usize;
                            }
                        } else {
                            return Err(VMError::FunctionNotFound("current".to_string()));
                        }
                    }
                } else {
                    return Err(VMError::ExpectedStackElementOfType("boolean", alpha));
                }
            }
            Instruction::CompareEqual => {
                let alpha = self.stack.pop().ok_or(VMError::StackEmpty)?;
                let beta = self.stack.pop().ok_or(VMError::StackEmpty)?;
                self.stack.push(PCell::Boolean(alpha == beta));
            }
            Instruction::BooleanNot => {
                let alpha = self.stack.pop().ok_or(VMError::StackEmpty)?;
                if let PCell::Boolean(v) = alpha {
                    self.stack.push(PCell::Boolean(!v));
                } else {
                    return Err(VMError::ExpectedStackElementOfType("boolean", alpha));
                }
            }
            Instruction::Jump(pcoff) => {
                if let Some(ctx) = self.function_stack.last_mut() {
                    if pcoff.is_positive() {
                        ctx.pc += pcoff as usize;
                    } else {
                        ctx.pc -= pcoff.abs() as usize;
                    }
                } else {
                    return Err(VMError::FunctionNotFound("current".to_string()));
                }
            }
            Instruction::ConstBoolean(v) => {
                self.stack.push(PCell::Boolean(v));
            }
            Instruction::ConstString(n) => {
                self.stack.push(PCell::String(self.program.string_table[n as usize].clone()));
            }
            Instruction::CallOuter(n) => {
                let label = self.program.outer_function_table.get(n as usize).ok_or(VMError::OuterFuncIDNotFound(n))?;
                let outer_func = self.outer_functions.get_mut(label).ok_or(VMError::OuterFuncIDNotFound(n))?;
                if self.stack.len() < outer_func.argument_count {
                    return Err(VMError::StackEmpty);
                }
                let args = self.stack.drain(self.stack.len()-outer_func.argument_count..self.stack.len()).collect::<Vec<_>>();
                assert_eq!(args.len(), outer_func.argument_count);
                let ret = outer_func.f.as_mut()(args, state);
                if let Some(ret) = ret {
                    if outer_func.returns_value {
                        self.stack.push(ret);
                    }
                } else if outer_func.returns_value {
                    return Err(VMError::OuterFuncDidNotReturnValue(n));
                }
            }
            Instruction::RotateDynamic => {
                let n = self.stack.pop().ok_or(VMError::StackEmpty)?;
                if let PCell::U32(n) = n {
                    let alpha = self.stack.get(n as usize).ok_or(VMError::StackEmpty)?.clone();
                    self.stack.push(alpha);
                } else {
                    return Err(VMError::ExpectedStackElementOfType("u32", n));
                }
            }
            Instruction::ExchangeDynamic => {
                let n = self.stack.pop().ok_or(VMError::StackEmpty)?;
                if let PCell::U32(n) = n {
                    let alpha = self.stack.pop().ok_or(VMError::StackEmpty)?;
                    *self.stack.get_mut(n as usize).ok_or(VMError::StackEmpty)? = alpha;
                } else {
                    return Err(VMError::ExpectedStackElementOfType("u32", n));
                }
            }
            Instruction::ConstU32(n) => {
                self.stack.push(PCell::U32(n));
            }
            Instruction::Call => {
                let function_id = self.stack.pop().ok_or(VMError::StackEmpty)?;
                if let PCell::U32(n) = function_id {
                    self.function_stack.push(FunctionContext {
                        pc: 0,
                        function: n as usize,
                    });
                    self.execute(state)?;
                    self.function_stack.pop();
                } else {
                    return Err(VMError::ExpectedStackElementOfType("u32", function_id));
                }
            }
            Instruction::AddU32 => {
                let alpha = self.stack.pop().ok_or(VMError::StackEmpty)?;
                let beta = self.stack.pop().ok_or(VMError::StackEmpty)?;
                match (alpha, beta) {
                    (PCell::U32(alpha), PCell::U32(beta)) => {
                        self.stack.push(PCell::U32(alpha.wrapping_add(beta)));
                    }
                    (alpha, beta) => {
                        return Err(VMError::ExpectedStackElementOfType("(u32, u32)", alpha));
                    }
                }
            }
            Instruction::PushStackPointer => {
                let stack_size = self.stack.len();
                self.stack.push(PCell::U32(stack_size as u32));
            }
            Instruction::SubU32 => {
                let alpha = self.stack.pop().ok_or(VMError::StackEmpty)?;
                let beta = self.stack.pop().ok_or(VMError::StackEmpty)?;
                match (alpha, beta) {
                    (PCell::U32(alpha), PCell::U32(beta)) => {
                        self.stack.push(PCell::U32(beta.wrapping_sub(alpha)));
                    }
                    (alpha, beta) => {
                        return Err(VMError::ExpectedStackElementOfType("(u32, u32)", alpha));
                    }
                }
            }
        }
        Ok(())
    }
}