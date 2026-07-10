/// stack writing convention
/// TOP, ONE FROM TOP, TWO FROM TOP ... BOTTOM
///
/// the String type in bytecode is represented as an index to the string in the string table

pub const ALL_OPCODES: &[Opcode] = &[
    Opcode::Nop,
    Opcode::Drop,
    Opcode::Dup,
    Opcode::Swap,
    Opcode::Rotate,
    Opcode::Exchange,
    Opcode::PushEmpty,
    Opcode::RelativeBranch,
    Opcode::CompareEqual,
    Opcode::BooleanNot,
    Opcode::ConstString,
    Opcode::ConstBoolean,
    Opcode::CallOuter,
];

#[repr(u8)]
#[derive(Copy, Clone, Debug)]
pub enum Opcode {
    /// does nothing
    Nop = 0,
    /// drops a stack element
    /// <stack element>
    /// -
    /// (intentionally left blank)
    Drop = 1,
    /// duplicates a stack element
    /// A
    /// -
    /// A, A
    Dup = 2,
    /// swaps the top two stack elements
    /// A, B
    /// -
    /// B, A
    Swap = 3,
    /// copies an element from further down the stack to the top
    /// Rotate( N: u32 )
    /// A, B, ..., <element N from bottom>
    /// -
    /// <element N now at top>, A, B, ..., <element N from bottom>
    Rotate = 4,
    /// takes the top element of the stack and moves it backwards into another stack cell
    /// Exchange( N: u32 )
    /// A, B, ..., <element N from bottom>
    /// -
    /// B, ..., <element N now == A, previous value lost>
    Exchange = 5,
    /// pushes an empty cell to the stack
    /// (intentionally left blank)
    /// -
    /// <EMPTY ELEMENT>
    PushEmpty = 6,
    /// adds the given number to the program counter if the top cell on the stack is a Boolean::True
    /// RelativeBranch( N: i32 )
    /// <boolean>, ...
    /// -
    /// (intentionally left blank)
    RelativeBranch = 7,
    /// compares two elements and pushes the comparison result as a boolean
    /// A, B, ...
    /// -
    /// if A == B then True else False, ...
    CompareEqual = 8,
    /// inverts the value of a boolean
    /// A
    /// -
    /// !A
    BooleanNot,
    /// loads a string constant
    /// ConstString( str: String )
    /// (intentionally left blank)
    /// -
    /// <str>
    ConstString = 32,
    /// loads a boolean constant
    /// ConstBoolean( b: bool )
    /// (intentionally left blank)
    /// -
    /// <bool>
    ConstBoolean = 33,
    ///
    /// calls a function that was defined outside of the scope of the bytecode
    /// CallOuter( FUNC_ID: u64 )
    /// <argument N>, <argument N-1>, ..., <argument 2>, <argument 1>
    /// -
    /// <return value>
    CallOuter = 128,
}

impl Opcode {
    pub fn decode(byte: u8) -> Option<Opcode> {
        for opcode in ALL_OPCODES {
            if byte == (*opcode as u8) {
                return Some(*opcode);
            }
        }

        None
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Instruction {
    Nop,
    Drop,
    Dup,
    Swap,
    Rotate(u32),
    Exchange(u32),
    PushEmpty,
    RelativeBranch(i32),
    CompareEqual,
    BooleanNot,
    ConstString(u64),
    ConstBoolean(bool),
    CallOuter(u64),
}

impl Instruction {
    pub fn opcode(&self) -> Opcode {
        match self {
            Instruction::Nop => Opcode::Nop,
            Instruction::Drop => Opcode::Drop,
            Instruction::Dup => Opcode::Dup,
            Instruction::Swap => Opcode::Swap,
            Instruction::Rotate(_) => Opcode::Rotate,
            Instruction::Exchange(_) => Opcode::Exchange,
            Instruction::PushEmpty => Opcode::PushEmpty,
            Instruction::RelativeBranch(_) => Opcode::RelativeBranch,
            Instruction::CompareEqual => Opcode::CompareEqual,
            Instruction::BooleanNot => Opcode::BooleanNot,
            Instruction::ConstString(_) => Opcode::ConstString,
            Instruction::CallOuter(_) => Opcode::CallOuter,
            Instruction::ConstBoolean(_) => Opcode::ConstBoolean,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Program {
    pub string_table: Vec<String>,
    pub outer_function_table: Vec<String>,
    pub inner_function_table: Vec<(String, usize)>,
    pub toplevel_function_table: Vec<(String, usize)>,
    pub functions: Vec<Vec<Instruction>>,
}