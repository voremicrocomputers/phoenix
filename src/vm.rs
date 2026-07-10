use crate::phoenixarch::{Instruction, Program};

pub struct RTOuterFunction  {
    pub argument_count: usize,
    pub f: Box<dyn FnMut(&[PCell])>,
}

#[derive(Debug, Clone)]
pub enum PCell {
    Blank,
    String(String),
}

#[derive(Debug)]
pub enum VMError {
    StackEmpty,
    FunctionNotFound(String),
    OuterFuncIDNotFound(u64),
}

pub struct FunctionContext {
    pub pc: usize,
    pub function: usize,
}

pub struct PhoenixVMState<'a> {
    pub stack: Vec<PCell>,
    pub function_stack: Vec<FunctionContext>,
    pub outer_functions: &'a mut [RTOuterFunction],
    pub program: &'a Program,
}

impl<'a> PhoenixVMState<'a> {
    pub fn new(outer_functions: &'a mut [RTOuterFunction], program: &'a Program) -> PhoenixVMState<'a> {
        PhoenixVMState {
            stack: vec![],
            function_stack: vec![],
            outer_functions,
            program,
        }
    }

    pub fn execute_function(&mut self, func: &str) -> Result<(), VMError> {
        if let Some(idx) = self.program.toplevel_function_table.iter().find(|v| v.0 == func).map(|v| v.1).clone() {
            self.function_stack.push(FunctionContext {
                pc: 0,
                function: idx,
            });
            self.execute()?;
            Ok(())
        } else {
            Err(VMError::FunctionNotFound(func.to_string()))
        }
    }

    pub fn execute(&mut self) -> Result<(), VMError> {
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

            self.execute_instruction(instruction)?;
        }
        Ok(())
    }

    pub fn execute_instruction(&mut self, ins: Instruction) -> Result<(), VMError> {
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
            Instruction::Rotate(n) => {
                let alpha = self.stack.get(n as usize).ok_or(VMError::StackEmpty)?.clone();
                self.stack.push(alpha);
            }
            Instruction::Exchange(n) => {
                let alpha = self.stack.pop().ok_or(VMError::StackEmpty)?;
                *self.stack.get_mut(n as usize).ok_or(VMError::StackEmpty)? = alpha;
            }
            Instruction::PushEmpty => {
                self.stack.push(PCell::Blank);
            }
            Instruction::ConstString(n) => {
                self.stack.push(PCell::String(self.program.string_table[n as usize].clone()));
            }
            Instruction::CallOuter(n) => {
                let outer_func = self.outer_functions.get_mut(n as usize).ok_or(VMError::OuterFuncIDNotFound(n))?;
                if self.stack.len() < outer_func.argument_count {
                    return Err(VMError::StackEmpty);
                }
                let args = &self.stack[self.stack.len()-outer_func.argument_count..self.stack.len()];
                assert_eq!(args.len(), outer_func.argument_count);
                outer_func.f.as_mut()(args);
            }
        }
        Ok(())
    }
}