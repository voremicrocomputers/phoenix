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
    Opcode::PushVarN,
    Opcode::SetVarN,
    Opcode::ConstString,
    Opcode::CallUpper,
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
    /// brings an element from further down the stack to the top
    /// A, B, C
    /// -
    /// C, A, B
    Rotate = 4,
    /// takes the value in var N and pushes it to the stack
    /// PushVarN( N: u8 )
    /// (intentionally left blank)
    /// -
    /// <value in that var>
    PushVarN = 5,
    /// takes the value on the stack and puts it in var N
    /// SetVarN( N: u8 )
    /// <value>
    /// -
    /// (intentionally left blank)
    SetVarN = 6,
    /// loads a string constant
    /// ConstString( str: String )
    /// (intentionally left blank)
    /// -
    /// <str>
    ConstString = 32,
    ///
    /// calls a function that was defined outside of the scope of the bytecode
    /// CallUpper( FUNC_ID: u64 )
    /// <argument N>, <argument N-1>, ..., <argument 2>, <argument 1>
    /// -
    /// <return value>
    CallUpper = 128,
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