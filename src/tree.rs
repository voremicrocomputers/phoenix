use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use crate::tokenize::{Token, TokenData};

pub enum FType {
    BadType,
    Void,
    Char,
    Usize,
    Isize,
}

pub struct FElm {
    pub id: usize,
    pub line: usize,
    pub character: usize,
    pub data: FElmData,
}

pub enum FElmData {
    ToBeFilled,
    Function(Arc<FElmFunction>),
    Closure(Arc<FElmClosure>),
    Call(Arc<FElmCall>),
    NumberLiteral(Arc<FElmNumberLiteral>),
    StringLiteral(Arc<FElmStringLiteral>),
    ExternFunction(Arc<FElmExternFunction>),
    VarRef(Arc<FElmVarRef>),
    VarDef(Arc<FElmVarDef>),
    UnaryExpression(Arc<FElmUnaryExpression>),
    BinaryExpression(Arc<FElmBinaryExpression>),
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

#[derive(Debug)]
pub struct TreeError {
    pub line: usize,
    pub character: usize,
    pub error: TreeErrorType,
}

#[derive(Debug)]
pub enum TreeErrorType {
    UnexpectedToken(TokenData),
    ExpectedFollowingToken(&'static [&'static str])
}

pub fn next_id(idstate: &AtomicUsize) -> usize {
    idstate.fetch_add(1, Ordering::Relaxed)
}

pub fn make_tree(tokens: Vec<Token>) -> Result<FElm, TreeError> {
    let idstate = AtomicUsize::new(0);
    // top level element will always be a closure
    let mut tle = FElm {
        id: next_id(&idstate),
        line: 0,
        character: 0,
        data: FElmData::Closure(Arc::new(FElmClosure {
            instructions: vec![],
        })),
    };

    // parse tokens
    let mut tokens = tokens.into_iter().collect::<VecDeque<_>>();

    while let Some(token) = tokens.pop_front() {
        match &token.data {
            TokenData::Literal(lit) => {
                match lit.as_str() {
                    "extern" => {
                        if let Some(TokenData::Literal(str)) = tokens.pop_front().map(|v| v.data) {
                            match str.as_str() {
                                "fn" => {

                                }
                                _ => {
                                    return Err(TreeError {
                                        line: token.line,
                                        character: token.character,
                                        error: TreeErrorType::UnexpectedToken(TokenData::Literal(str)),
                                    })
                                }
                            }
                        } else {
                            return Err(TreeError {
                                line: token.line,
                                character: token.character,
                                error: TreeErrorType::ExpectedFollowingToken(&["fn"]),
                            });
                        }
                    }
                    _ => {
                        return Err(TreeError {
                            line: token.line,
                            character: token.character,
                            error: TreeErrorType::UnexpectedToken(token.data),
                        });
                    }
                }
            }
            _ => {
                return Err(TreeError {
                    line: token.line,
                    character: token.character,
                    error: TreeErrorType::UnexpectedToken(token.data),
                });
            }
        }
    }

    Ok(tle)
}