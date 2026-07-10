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
    Opcode::Jump,
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
    Jump(i32),
    ConstString(u16),
    ConstBoolean(bool),
    CallOuter(u16),
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
            Instruction::Jump(_) => Opcode::Jump,
            Instruction::ConstString(_) => Opcode::ConstString,
            Instruction::CallOuter(_) => Opcode::CallOuter,
            Instruction::ConstBoolean(_) => Opcode::ConstBoolean,
        }
    }

    pub fn bytecode(&self) -> Vec<u8> {
        let mut buf = vec![];

        buf.push(self.opcode() as u8);
        match self {
            Instruction::Rotate(n) => {
                buf.extend((*n).to_be_bytes());
            }
            Instruction::Exchange(n) => {
                buf.extend((*n).to_be_bytes());
            }
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
            Opcode::Rotate => {
                if *i + size_of::<u32>() >= buf.len() {
                    return None;
                }
                let n = u32::from_be_bytes((&buf[*i..*i+size_of::<u32>()]).try_into().unwrap());
                *i += size_of::<u32>();
                Some(Instruction::Rotate(n))
            }
            Opcode::Exchange => {
                if *i + size_of::<u32>() >= buf.len() {
                    return None;
                }
                let n = u32::from_be_bytes((&buf[*i..*i+size_of::<u32>()]).try_into().unwrap());
                *i += size_of::<u32>();
                Some(Instruction::Exchange(n))
            }
            Opcode::PushEmpty => Some(Instruction::PushEmpty),
            Opcode::RelativeBranch => {
                if *i + size_of::<i32>() >= buf.len() {
                    return None;
                }
                let n = i32::from_be_bytes((&buf[*i..*i+size_of::<i32>()]).try_into().unwrap());
                *i += size_of::<i32>();
                Some(Instruction::RelativeBranch(n))
            }
            Opcode::CompareEqual => Some(Instruction::CompareEqual),
            Opcode::BooleanNot => Some(Instruction::BooleanNot),
            Opcode::Jump => {
                if *i + size_of::<i32>() >= buf.len() {
                    return None;
                }
                let n = i32::from_be_bytes((&buf[*i..*i+size_of::<i32>()]).try_into().unwrap());
                *i += size_of::<i32>();
                Some(Instruction::Jump(n))
            }
            Opcode::ConstString => {
                if *i + size_of::<u16>() >= buf.len() {
                    return None;
                }
                let n = u16::from_be_bytes((&buf[*i..*i+size_of::<u16>()]).try_into().unwrap());
                *i += size_of::<u16>();
                Some(Instruction::ConstString(n))
            }
            Opcode::ConstBoolean => {
                if *i + size_of::<u8>() >= buf.len() {
                    return None;
                }
                let n = u8::from_be_bytes((&buf[*i..*i+size_of::<u8>()]).try_into().unwrap());
                *i += size_of::<u8>();

                Some(Instruction::ConstBoolean(n != 0))
            }
            Opcode::CallOuter => {
                if *i + size_of::<u16>() >= buf.len() {
                    return None;
                }
                let n = u16::from_be_bytes((&buf[*i..*i+size_of::<u16>()]).try_into().unwrap());
                *i += size_of::<u16>();
                Some(Instruction::CallOuter(n))
            }
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

    pub fn from_bytecode(buf: &[u8]) -> Option<Self> {
        let mut i = 0;
        if i + 3 >= buf.len() {
            return None;
        }
        if &buf[i..i+3] != b"FNX" {
            return None;
        }
        i += 3;

        if i + size_of::<u16>() >= buf.len() {
            return None;
        }
        let string_table_len = u16::from_be_bytes((&buf[i..i+size_of::<u16>()]).try_into().unwrap());
        i += size_of::<u16>();
        let mut string_table = Vec::with_capacity(string_table_len as usize);
        for _ in 0..string_table_len {
            if i + size_of::<u16>() >= buf.len() {
                return None;
            }
            let string_len = u16::from_be_bytes((&buf[i..i+size_of::<u16>()]).try_into().unwrap());
            i += size_of::<u16>();
            if i + string_len as usize >= buf.len() {
                return None;
            }
            let string_bytes = &buf[i..i+(string_len as usize)];
            i += string_len as usize;
            string_table.push(String::from_utf8_lossy(string_bytes).to_string());
        }

        if i + size_of::<u16>() >= buf.len() {
            return None;
        }
        let outer_function_table_len = u16::from_be_bytes((&buf[i..i+size_of::<u16>()]).try_into().unwrap());
        i += size_of::<u16>();
        let mut outer_function_table = Vec::with_capacity(outer_function_table_len as usize);
        for _ in 0..outer_function_table_len {
            if i + size_of::<u8>() >= buf.len() {
                return None;
            }
            let string_len = u8::from_be_bytes((&buf[i..i+size_of::<u8>()]).try_into().unwrap());
            i += size_of::<u8>();
            if i + string_len as usize >= buf.len() {
                return None;
            }
            let string_bytes = &buf[i..i+(string_len as usize)];
            i += string_len as usize;
            outer_function_table.push(String::from_utf8_lossy(string_bytes).to_string());
        }

        if i + size_of::<u8>() >= buf.len() {
            return None;
        }
        let toplevel_function_table_len = u8::from_be_bytes((&buf[i..i+size_of::<u8>()]).try_into().unwrap());
        i += size_of::<u8>();
        let mut toplevel_function_table = Vec::with_capacity(toplevel_function_table_len as usize);
        for _ in 0..toplevel_function_table_len {
            if i + size_of::<u8>() >= buf.len() {
                return None;
            }
            let string_len = u8::from_be_bytes((&buf[i..i+size_of::<u8>()]).try_into().unwrap());
            i += size_of::<u8>();
            if i + string_len as usize >= buf.len() {
                return None;
            }
            let string_bytes = &buf[i..i+(string_len as usize)];
            i += string_len as usize;
            if i + size_of::<u16>() >= buf.len() {
                return None;
            }
            let index = u16::from_be_bytes((&buf[i..i+size_of::<u16>()]).try_into().unwrap());
            i += size_of::<u16>();
            toplevel_function_table.push((String::from_utf8_lossy(string_bytes).to_string(), index as usize));
        }

        let mut functions = vec![];
        while i < buf.len() {
            if i + size_of::<u32>() >= buf.len() {
                return None;
            }
            let instruction_count = u32::from_be_bytes((&buf[i..i+size_of::<u32>()]).try_into().unwrap());
            i += size_of::<u32>();
            let mut instructions = vec![];

            for _ in 0..instruction_count {
                if i >= buf.len() {
                    return None;
                }
                let ins = Instruction::from_bytecode(buf, &mut i)?;
                instructions.push(ins);
            }

            functions.push(instructions);
        }

        Some(Program {
            string_table,
            outer_function_table,
            toplevel_function_table,
            functions,
        })
    }
}