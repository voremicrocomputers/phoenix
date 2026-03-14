pub enum FType {
    BadType,
    Void,
    Char,
    Usize,
    Isize,
}

pub struct FElm {
    pub id: usize,
    pub data: FElmData,
}

pub enum FElmData {
    ToBeFilled,
    Function(Box<FElmFunction>),
    Closure(Box<FElmClosure>),
    Call(Box<FElmCall>),
    NumberLiteral,
    StringLiteral,
    ExternFunction,
    VarRef,
    VarDef,
    UnaryExpression,
    BinaryExpression,
}

pub struct FElmFunction {
    pub label: String,
    pub args: Vec<FElmVarDef>, // doesn't use assigns
    pub return_type: FType,
    pub return_type_ref: usize,
    pub body: FElm,
}

pub struct FElmClosure {
    pub instructions: Vec<FElm>,
}

pub struct FElmCall {
    pub label: String,
    pub args: Vec<FElm>,
}

pub struct FElmNumberLiteral {
    pub value: isize,
}

pub struct FElmStringLiteral {
    pub value: String,
}

pub struct FElmExternFunction {
    pub label: String,
    pub args: Vec<FElmVarDef>, // doesn't use names or assigns
}

pub struct FElmVarRef {
    pub value: String,
}

pub struct FElmVarDef {
    pub ftype: FType,
    pub ftype_ref: usize,
    pub name: String,
    pub assign: Option<FElm>,
}

pub enum Operator {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    BitwiseAnd,
    BooleanAnd,
    BitwiseOr,
    BooleanOr,
    BitwiseXor,
    Not,
    EQ,
    NotEQ,
    GreaterThan,
    LessThan,
    GreaterThanEqual,
    LessThanEqual,
}

pub struct FElmUnaryExpression {
    pub alpha: FElm,
    pub operator: Operator,
}

pub struct FElmBinaryExpression {
    pub alpha: FElm,
    pub beta: FElm,
    pub operator: Operator,
}

pub enum TreeErrorType {

}