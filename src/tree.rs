use crate::tokenize::{Token, TokenData};
use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

pub enum FType {
    BadType,
    Void,
    Char,
    Usize,
    Isize,
}

impl FType {
    pub fn from_literal(lit: &str) -> FType {
        match lit {
            "void" => FType::Void,
            "char" => FType::Char,
            "usize" => FType::Usize,
            "isize" => FType::Isize,
            _ => FType::BadType,
        }
    }
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
    pub return_type: FType,
    pub return_type_ref: usize,
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
    UnexpectedEnd,
    UnexpectedToken(TokenData),
    ExpectedFollowingToken(&'static [&'static str]),
    MissingType,
}

pub fn next_id(idstate: &AtomicUsize) -> usize {
    idstate.fetch_add(1, Ordering::Relaxed)
}

/// enter after "extern" "fn"
pub fn add_extern_function(
    tokens: &mut VecDeque<Token>,
    idstate: &AtomicUsize,
    closure: &mut Vec<FElm>,
) -> Result<(), TreeError> {
    // next token is function label
    let label = tokens.pop_front().ok_or(TreeError {
        line: 0,
        character: 0,
        error: TreeErrorType::UnexpectedEnd,
    })?;
    let start_line = label.line;
    let start_character = label.character;

    let mut efunc = FElmExternFunction {
        label: match label.data {
            TokenData::Literal(str) => str.clone(),
            x => {
                return Err(TreeError {
                    line: label.line,
                    character: label.character,
                    error: TreeErrorType::UnexpectedToken(x),
                });
            }
        },
        args: vec![],
        return_type: FType::BadType,
        return_type_ref: 0,
    };

    // next token should be openparen
    let openparen = tokens.pop_front().ok_or(TreeError {
        line: 0,
        character: 0,
        error: TreeErrorType::UnexpectedEnd,
    })?;
    match openparen.data {
        TokenData::OpenParenthesis => {}
        x => {
            return Err(TreeError {
                line: openparen.line,
                character: openparen.character,
                error: TreeErrorType::UnexpectedToken(x),
            });
        }
    }

    // arguments
    let mut refcount = 0;
    let mut started_argument = false;
    let mut finished_argument = true;
    while let Some(token) = tokens.pop_front() {
        match &token.data {
            TokenData::CloseParenthesis => {
                if !finished_argument {
                    return Err(TreeError {
                        line: token.line,
                        character: token.character,
                        error: TreeErrorType::UnexpectedToken(token.data),
                    });
                }
                break;
            }
            TokenData::Ampersand => {
                if !started_argument {
                    finished_argument = false;
                    started_argument = true;
                }
                if finished_argument {
                    return Err(TreeError {
                        line: token.line,
                        character: token.character,
                        error: TreeErrorType::ExpectedFollowingToken(&[",", ")"]),
                    });
                }
                refcount += 1;
            }
            TokenData::Comma => {
                if !finished_argument || !started_argument {
                    return Err(TreeError {
                        line: token.line,
                        character: token.character,
                        error: TreeErrorType::UnexpectedToken(token.data),
                    });
                }
                finished_argument = true;
                started_argument = false;
                refcount = 0;
            }
            TokenData::Literal(str) => {
                if !started_argument {
                    finished_argument = false;
                    started_argument = true;
                }
                if finished_argument {
                    return Err(TreeError {
                        line: token.line,
                        character: token.character,
                        error: TreeErrorType::ExpectedFollowingToken(&[",", ")"]),
                    });
                }
                // must be a type
                let littype = FType::from_literal(str);
                if matches!(&littype, FType::BadType) {
                    return Err(TreeError {
                        line: token.line,
                        character: token.character,
                        error: TreeErrorType::UnexpectedToken(token.data),
                    });
                }
                efunc.args.push(FElmVarDef {
                    ftype: littype,
                    ftype_ref: refcount,
                    name: "".to_string(), // not used
                    assign: None,         // not used
                });
                refcount = 0;
                finished_argument = true;
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

    // next token should be either ; (void return type) or -> (return type follows)
    let next = tokens.pop_front().ok_or(TreeError {
        line: 0,
        character: 0,
        error: TreeErrorType::UnexpectedEnd,
    })?;
    match &next.data {
        TokenData::Arrow => {
            // next token should either be & or a type
            let mut refcount = 0;
            let mut found_type = false;
            while let Some(token) = tokens.pop_front() {
                match &token.data {
                    TokenData::Ampersand => {
                        refcount += 1;
                    }
                    TokenData::Literal(str) => {
                        let ftype = FType::from_literal(str);
                        if matches!(&ftype, FType::BadType) {
                            return Err(TreeError {
                                line: token.line,
                                character: token.character,
                                error: TreeErrorType::UnexpectedToken(token.data),
                            });
                        }
                        efunc.return_type = ftype;
                        efunc.return_type_ref = refcount;
                        found_type = true;
                        break;
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
            if !found_type {
                return Err(TreeError {
                    line: start_line,
                    character: start_character,
                    error: TreeErrorType::MissingType,
                });
            }
        }
        TokenData::Semicolon => {
            efunc.return_type = FType::Void;
            efunc.return_type_ref = 0;
        }
        _ => {
            return Err(TreeError {
                line: next.line,
                character: next.character,
                error: TreeErrorType::UnexpectedToken(next.data),
            });
        }
    }

    closure.push(FElm {
        id: next_id(idstate),
        line: start_line,
        character: start_character,
        data: FElmData::ExternFunction(Arc::new(efunc)),
    });
    Ok(())
}

/// enter after "fn"
pub fn add_function(
    tokens: &mut VecDeque<Token>,
    idstate: &AtomicUsize,
    closure: &mut Vec<FElm>,
) -> Result<(), TreeError> {
    // next token is function label
    let label = tokens.pop_front().ok_or(TreeError {
        line: 0,
        character: 0,
        error: TreeErrorType::UnexpectedEnd,
    })?;
    let start_line = label.line;
    let start_character = label.character;

    let mut func = FElmFunction {
        label: match label.data {
            TokenData::Literal(str) => str.clone(),
            x => return Err(TreeError {
                line: label.line,
                character: label.character,
                error: TreeErrorType::UnexpectedToken(x),
            }),
        },
        args: vec![],
        return_type: FType::BadType,
        return_type_ref: 0,
        body: FElm {
            id: 0,
            line: 0,
            character: 0,
            data: FElmData::ToBeFilled,
        },
    };

    // next token should be openparen
    let openparen = tokens.pop_front().ok_or(TreeError {
        line: 0,
        character: 0,
        error: TreeErrorType::UnexpectedEnd,
    })?;
    match openparen.data {
        TokenData::OpenParenthesis => {}
        x => {
            return Err(TreeError {
                line: openparen.line,
                character: openparen.character,
                error: TreeErrorType::UnexpectedToken(x),
            });
        }
    }

    // arguments
    let mut refcount = 0;
    let mut name = "".to_string();
    let mut started_argument = false;
    let mut colon = false;
    let mut finished_argument = true;
    while let Some(token) = tokens.pop_front() {
        match &token.data {
            TokenData::CloseParenthesis => {
                if !finished_argument {
                    return Err(TreeError {
                        line: token.line,
                        character: token.character,
                        error: TreeErrorType::UnexpectedToken(token.data),
                    });
                }
                break;
            }
            TokenData::Comma => {
                if !finished_argument || !started_argument {
                    return Err(TreeError {
                        line: token.line,
                        character: token.character,
                        error: TreeErrorType::UnexpectedToken(token.data),
                    });
                }
                refcount = 0;
                name = "".to_string();
                started_argument = false;
                colon = false;
                finished_argument = true;
            }
            TokenData::Colon => {
                if name.is_empty() || colon {
                    return Err(TreeError {
                        line: token.line,
                        character: token.character,
                        error: TreeErrorType::UnexpectedToken(token.data),
                    });
                }
                colon = true;
            }
            TokenData::Ampersand => {
                if name.is_empty() || !colon || finished_argument {
                    return Err(TreeError {
                        line: token.line,
                        character: token.character,
                        error: TreeErrorType::UnexpectedToken(token.data),
                    });
                }
                refcount += 1;
            }
            TokenData::Literal(str) => {
                if !started_argument {
                    // this is the argument name
                    name = str.to_string();
                    started_argument = true;
                } else if colon && !finished_argument {
                    // this is the type
                    let littype = FType::from_literal(str);
                    if matches!(&littype, FType::BadType) {
                        return Err(TreeError {
                            line: token.line,
                            character: token.character,
                            error: TreeErrorType::UnexpectedToken(token.data),
                        });
                    }
                    func.args.push(FElmVarDef {
                        ftype: littype,
                        ftype_ref: refcount,
                        name: name.clone(),
                        assign: None,
                    });
                    finished_argument = true;
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

    // next token should be either { (void return type) or -> (return type follows)
    let next = tokens.pop_front().ok_or(TreeError {
        line: 0,
        character: 0,
        error: TreeErrorType::UnexpectedEnd,
    })?;
    match &next.data {
        TokenData::Arrow => {
            // next token should either be & or a type
            let mut refcount = 0;
            let mut found_type = false;
            while let Some(token) = tokens.pop_front() {
                match &token.data {
                    TokenData::Ampersand => {
                        refcount += 1;
                    }
                    TokenData::Literal(str) => {
                        let ftype = FType::from_literal(str);
                        if matches!(&ftype, FType::BadType) {
                            return Err(TreeError {
                                line: token.line,
                                character: token.character,
                                error: TreeErrorType::UnexpectedToken(token.data),
                            });
                        }
                        func.return_type = ftype;
                        func.return_type_ref = refcount;
                        found_type = true;
                        break;
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
            if !found_type {
                return Err(TreeError {
                    line: start_line,
                    character: start_character,
                    error: TreeErrorType::MissingType,
                });
            }
        }
        TokenData::OpenBody => {
            func.return_type = FType::Void;
            func.return_type_ref = 0;
            tokens.push_front(next);
        }
        _ => {
            return Err(TreeError {
                line: next.line,
                character: next.character,
                error: TreeErrorType::UnexpectedToken(next.data),
            });
        }
    }

    // next token should be {
    // todo: parse function body here


    closure.push(FElm {
        id: next_id(idstate),
        line: start_line,
        character: start_character,
        data: FElmData::Function(Arc::new(func)),
    });

    Ok(())
}

pub fn make_tree(tokens: Vec<Token>) -> Result<FElm, TreeError> {
    let idstate = AtomicUsize::new(0);
    // top level element will always be a closure

    let mut instructions = vec![];

    // parse tokens
    let mut tokens = tokens.into_iter().collect::<VecDeque<_>>();

    while let Some(token) = tokens.pop_front() {
        match &token.data {
            TokenData::Literal(lit) => match lit.as_str() {
                "extern" => {
                    if let Some(TokenData::Literal(str)) = tokens.pop_front().map(|v| v.data) {
                        match str.as_str() {
                            "fn" => {
                                add_extern_function(&mut tokens, &idstate, &mut instructions)?
                            }
                            _ => {
                                return Err(TreeError {
                                    line: token.line,
                                    character: token.character,
                                    error: TreeErrorType::UnexpectedToken(TokenData::Literal(str)),
                                });
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
                "fn" => {

                }
                _ => {
                    return Err(TreeError {
                        line: token.line,
                        character: token.character,
                        error: TreeErrorType::UnexpectedToken(token.data),
                    });
                }
            },
            _ => {
                return Err(TreeError {
                    line: token.line,
                    character: token.character,
                    error: TreeErrorType::UnexpectedToken(token.data),
                });
            }
        }
    }

    let tle = FElm {
        id: next_id(&idstate),
        line: 0,
        character: 0,
        data: FElmData::Closure(Arc::new(FElmClosure {
            instructions,
        })),
    };

    Ok(tle)
}
