/// stack writing convention
/// TOP, ONE FROM TOP, TWO FROM TOP ... BOTTOM
///
/// the String type in bytecode is represented as an index to the string in the string table

pub const ALL_OPCODES: &[Opcode] = &[
    Opcode::Nop,
    Opcode::Drop,
    Opcode::Dup,
    Opcode::Swap,
    Opcode::PushEmpty,
    Opcode::RelativeBranch,
    Opcode::CompareEqual,
    Opcode::BooleanNot,
    Opcode::Jump,
    Opcode::RotateDynamic,
    Opcode::ExchangeDynamic,
    Opcode::AddU32,
    Opcode::SubU32,
    Opcode::CompareGT,
    Opcode::CompareLT,
    Opcode::ConstString,
    Opcode::ConstBoolean,
    Opcode::ConstU32,
    Opcode::PushStackPointer,
    Opcode::CallOuter,
    Opcode::Call,
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
    /// pushes an empty cell to the stack
    /// (intentionally left blank)
    /// -
    /// <EMPTY ELEMENT>
    PushEmpty = 6,
    /// adds the given number to the program counter if the top cell on the stack is a Boolean::False
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
    BooleanNot = 9,
    /// unconditional program counter jump
    /// Jump( N: i32 )
    /// (has no effect on stack)
    Jump = 10,
    /// same as Rotate, but N is taken from the top of the stack
    /// N, A, ..., <element N from bottom>
    /// -
    /// <element N copy>, A, ..., <element N from bottom>
    RotateDynamic = 11,
    /// same as Exchange, but N is taken from the top of the stack
    /// N, V, A, ..., <element N from bottom>
    /// -
    /// A, ..., <element N now == V, previous value lost>
    ExchangeDynamic = 12,
    /// adds two u32s
    /// A, B
    /// -
    /// <A + B>
    AddU32 = 13,
    /// subtracts top u32 from bottom u32
    /// A, B
    /// -
    /// <B - A>
    SubU32 = 14,
    /// compares two elements and pushes the comparison result as a boolean
    /// A, B, ...
    /// -
    /// if B > A then True else False, ...
    CompareGT = 15,
    /// compares two elements and pushes the comparison result as a boolean
    /// A, B, ...
    /// -
    /// if B < A then True else False, ...
    CompareLT = 16,
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
    /// loads a u32 constant
    /// ConstU32( n: u32 )
    /// (intentionally left blank)
    /// -
    /// <n>
    ConstU32 = 34,
    /// pushes the stack pointer as a u32 to the stack
    /// A, B, C <BOTTOM>
    /// -
    /// 2, A, B, C <BOTTOM>
    PushStackPointer = 64,
    ///
    /// calls a function that was defined outside of the scope of the bytecode
    /// CallOuter( FUNC_ID: u64 )
    /// <argument N>, <argument N-1>, ..., <argument 2>, <argument 1>
    /// -
    /// <return value>
    CallOuter = 128,
    /// calls a function with the given id, does not clean up
    /// ID: u32, <argument N>, <argument N-1>, ..., <argument 2>, <argument 1>
    /// -
    /// <return value>, <argument N>, <argument N-1>, ..., <argument 2>, <argument 1>
    Call = 129,
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
    PushEmpty,
    RelativeBranch(i32),
    CompareEqual,
    BooleanNot,
    Jump(i32),
    RotateDynamic,
    ExchangeDynamic,
    AddU32,
    SubU32,
    CompareGT,
    CompareLT,
    ConstString(u16),
    ConstBoolean(bool),
    ConstU32(u32),
    PushStackPointer,
    CallOuter(u16),
    Call,
}

impl Instruction {
    pub fn opcode(&self) -> Opcode {
        match self {
            Instruction::Nop => Opcode::Nop,
            Instruction::Drop => Opcode::Drop,
            Instruction::Dup => Opcode::Dup,
            Instruction::Swap => Opcode::Swap,
            Instruction::PushEmpty => Opcode::PushEmpty,
            Instruction::RelativeBranch(_) => Opcode::RelativeBranch,
            Instruction::CompareEqual => Opcode::CompareEqual,
            Instruction::BooleanNot => Opcode::BooleanNot,
            Instruction::Jump(_) => Opcode::Jump,
            Instruction::RotateDynamic => Opcode::RotateDynamic,
            Instruction::ExchangeDynamic => Opcode::ExchangeDynamic,
            Instruction::AddU32 => Opcode::AddU32,
            Instruction::SubU32 => Opcode::SubU32,
            Instruction::CompareGT => Opcode::CompareGT,
            Instruction::CompareLT => Opcode::CompareLT,
            Instruction::ConstString(_) => Opcode::ConstString,
            Instruction::CallOuter(_) => Opcode::CallOuter,
            Instruction::ConstBoolean(_) => Opcode::ConstBoolean,
            Instruction::ConstU32(_) => Opcode::ConstU32,
            Instruction::PushStackPointer => Opcode::PushStackPointer,
            Instruction::Call => Opcode::Call,
        }
    }

    pub fn bytecode(&self) -> Vec<u8> {
        let mut buf = vec![];

        buf.push(self.opcode() as u8);
        match self {
            Instruction::RelativeBranch(i) => {
                buf.extend((*i).to_be_bytes());
            }
            Instruction::Jump(i) => {
                buf.extend((*i).to_be_bytes());
            }
            Instruction::ConstString(n) => {
                buf.extend((*n).to_be_bytes());
            }
            Instruction::ConstBoolean(v) => {
                buf.push(*v as u8);
            }
            Instruction::CallOuter(n) => {
                buf.extend((*n).to_be_bytes());
            }
            Instruction::ConstU32(n) => {
                buf.extend((*n).to_be_bytes());
            }
            _ => {}
        }

        buf
    }

    pub fn from_bytecode(buf: &[u8], i: &mut usize) -> Option<Instruction> {
        let opcode = buf[*i];
        *i += 1;
        let opcode = Opcode::decode(opcode)?;

        match opcode {
            Opcode::Nop => Some(Instruction::Nop),
            Opcode::Drop => Some(Instruction::Drop),
            Opcode::Dup => Some(Instruction::Dup),
            Opcode::Swap => Some(Instruction::Swap),
            Opcode::PushEmpty => Some(Instruction::PushEmpty),
            Opcode::RelativeBranch => {
                if *i + size_of::<i32>() > buf.len() {
                    return None;
                }
                let n = i32::from_be_bytes((&buf[*i..*i+size_of::<i32>()]).try_into().unwrap());
                *i += size_of::<i32>();
                Some(Instruction::RelativeBranch(n))
            }
            Opcode::CompareEqual => Some(Instruction::CompareEqual),
            Opcode::BooleanNot => Some(Instruction::BooleanNot),
            Opcode::Jump => {
                if *i + size_of::<i32>() > buf.len() {
                    return None;
                }
                let n = i32::from_be_bytes((&buf[*i..*i+size_of::<i32>()]).try_into().unwrap());
                *i += size_of::<i32>();
                Some(Instruction::Jump(n))
            }
            Opcode::RotateDynamic => Some(Instruction::RotateDynamic),
            Opcode::ExchangeDynamic => Some(Instruction::ExchangeDynamic),
            Opcode::AddU32 => Some(Instruction::AddU32),
            Opcode::SubU32 => Some(Instruction::SubU32),
            Opcode::CompareGT => Some(Instruction::CompareGT),
            Opcode::CompareLT => Some(Instruction::CompareLT),
            Opcode::ConstString => {
                if *i + size_of::<u16>() > buf.len() {
                    return None;
                }
                let n = u16::from_be_bytes((&buf[*i..*i+size_of::<u16>()]).try_into().unwrap());
                *i += size_of::<u16>();
                Some(Instruction::ConstString(n))
            }
            Opcode::ConstBoolean => {
                if *i + size_of::<u8>() > buf.len() {
                    return None;
                }
                let n = u8::from_be_bytes((&buf[*i..*i+size_of::<u8>()]).try_into().unwrap());
                *i += size_of::<u8>();

                Some(Instruction::ConstBoolean(n != 0))
            }
            Opcode::CallOuter => {
                if *i + size_of::<u16>() > buf.len() {
                    return None;
                }
                let n = u16::from_be_bytes((&buf[*i..*i+size_of::<u16>()]).try_into().unwrap());
                *i += size_of::<u16>();
                Some(Instruction::CallOuter(n))
            }
            Opcode::ConstU32 => {
                if *i + size_of::<u32>() > buf.len() {
                    return None;
                }
                let n = u32::from_be_bytes((&buf[*i..*i+size_of::<u32>()]).try_into().unwrap());
                *i += size_of::<u32>();
                Some(Instruction::ConstU32(n))
            }
            Opcode::PushStackPointer => Some(Instruction::PushStackPointer),
            Opcode::Call => Some(Instruction::Call),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Program {
    pub string_table: Vec<String>,
    pub outer_function_table: Vec<String>,
    pub toplevel_function_table: Vec<(String, usize)>,
    pub functions: Vec<Vec<Instruction>>,
}

#[derive(Debug)]
pub enum BytecodeParseError {
    NoMagic,
    NoStringTableLength,
    InvalidStringTableEntry(usize),
    NoOuterFunctionTableLength,
    InvalidOuterFunctionTableEntry(usize),
    NoTopLevelFunctionTableLength,
    InvalidTopLevelFunctionTableEntry(usize),
    NoFunctionsLength,
    InvalidFunction(usize),
    InvalidInstructionInFunction(usize, usize),
}

impl Program {
    pub fn bytecode(&self) -> Vec<u8> {
        let mut buf = vec![];

        buf.extend(b"FNX");

        buf.extend((self.string_table.len() as u16).to_be_bytes());
        for string in &self.string_table {
            buf.extend((string.len() as u16).to_be_bytes());
            buf.extend(string.as_bytes());
        }

        buf.extend((self.outer_function_table.len() as u16).to_be_bytes());
        for string in &self.outer_function_table {
            buf.extend((string.len() as u8).to_be_bytes());
            buf.extend(string.as_bytes());
        }

        buf.extend((self.toplevel_function_table.len() as u8).to_be_bytes());
        for (string, index) in &self.toplevel_function_table {
            buf.extend((string.len() as u8).to_be_bytes());
            buf.extend(string.as_bytes());
            buf.extend((*index as u16).to_be_bytes());
        }

        for func in &self.functions {
            buf.extend((func.len() as u32).to_be_bytes());
            for ins in func {
                buf.extend(ins.bytecode());
            }
        }

        buf
    }

    pub fn from_bytecode(buf: &[u8]) -> Result<Self, BytecodeParseError> {
        let mut i = 0;
        if i + 3 >= buf.len() {
            return Err(BytecodeParseError::NoMagic);
        }
        if &buf[i..i+3] != b"FNX" {
            return Err(BytecodeParseError::NoMagic);
        }
        i += 3;

        if i + size_of::<u16>() >= buf.len() {
            return Err(BytecodeParseError::NoStringTableLength);
        }
        let string_table_len = u16::from_be_bytes((&buf[i..i+size_of::<u16>()]).try_into().unwrap());
        i += size_of::<u16>();
        let mut string_table = Vec::with_capacity(string_table_len as usize);
        for j in 0..string_table_len {
            if i + size_of::<u16>() >= buf.len() {
                return Err(BytecodeParseError::InvalidStringTableEntry(j as usize));
            }
            let string_len = u16::from_be_bytes((&buf[i..i+size_of::<u16>()]).try_into().unwrap());
            i += size_of::<u16>();
            if i + string_len as usize >= buf.len() {
                return Err(BytecodeParseError::InvalidStringTableEntry(j as usize));
            }
            let string_bytes = &buf[i..i+(string_len as usize)];
            i += string_len as usize;
            string_table.push(String::from_utf8_lossy(string_bytes).to_string());
        }

        if i + size_of::<u16>() >= buf.len() {
            return Err(BytecodeParseError::NoOuterFunctionTableLength);
        }
        let outer_function_table_len = u16::from_be_bytes((&buf[i..i+size_of::<u16>()]).try_into().unwrap());
        i += size_of::<u16>();
        let mut outer_function_table = Vec::with_capacity(outer_function_table_len as usize);
        for j in 0..outer_function_table_len {
            if i + size_of::<u8>() >= buf.len() {
                return Err(BytecodeParseError::InvalidOuterFunctionTableEntry(j as usize));
            }
            let string_len = u8::from_be_bytes((&buf[i..i+size_of::<u8>()]).try_into().unwrap());
            i += size_of::<u8>();
            if i + string_len as usize >= buf.len() {
                return Err(BytecodeParseError::InvalidOuterFunctionTableEntry(j as usize));
            }
            let string_bytes = &buf[i..i+(string_len as usize)];
            i += string_len as usize;
            outer_function_table.push(String::from_utf8_lossy(string_bytes).to_string());
        }

        if i + size_of::<u8>() >= buf.len() {
            return Err(BytecodeParseError::NoTopLevelFunctionTableLength);
        }
        let toplevel_function_table_len = u8::from_be_bytes((&buf[i..i+size_of::<u8>()]).try_into().unwrap());
        i += size_of::<u8>();
        let mut toplevel_function_table = Vec::with_capacity(toplevel_function_table_len as usize);
        for j in 0..toplevel_function_table_len {
            if i + size_of::<u8>() >= buf.len() {
                return Err(BytecodeParseError::InvalidTopLevelFunctionTableEntry(j as usize));
            }
            let string_len = u8::from_be_bytes((&buf[i..i+size_of::<u8>()]).try_into().unwrap());
            i += size_of::<u8>();
            if i + string_len as usize >= buf.len() {
                return Err(BytecodeParseError::InvalidTopLevelFunctionTableEntry(j as usize));
            }
            let string_bytes = &buf[i..i+(string_len as usize)];
            i += string_len as usize;
            if i + size_of::<u16>() >= buf.len() {
                return Err(BytecodeParseError::InvalidTopLevelFunctionTableEntry(j as usize));
            }
            let index = u16::from_be_bytes((&buf[i..i+size_of::<u16>()]).try_into().unwrap());
            i += size_of::<u16>();
            toplevel_function_table.push((String::from_utf8_lossy(string_bytes).to_string(), index as usize));
        }

        let mut functions = vec![];
        while i < buf.len() {
            if i + size_of::<u32>() >= buf.len() {
                return Err(BytecodeParseError::InvalidFunction(functions.len()));
            }
            let instruction_count = u32::from_be_bytes((&buf[i..i+size_of::<u32>()]).try_into().unwrap());
            i += size_of::<u32>();
            let mut instructions = vec![];

            for j in 0..instruction_count {
                if i >= buf.len() {
                    return Err(BytecodeParseError::InvalidFunction(functions.len()));
                }
                let ins = Instruction::from_bytecode(buf, &mut i).ok_or(BytecodeParseError::InvalidInstructionInFunction(functions.len(), j as usize))?;
                instructions.push(ins);
            }

            functions.push(instructions);
        }

        Ok(Program {
            string_table,
            outer_function_table,
            toplevel_function_table,
            functions,
        })
    }
}