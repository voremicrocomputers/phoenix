use std::collections::VecDeque;

#[derive(Clone, Debug)]
pub struct Token {
    pub data: TokenData,
    pub line: usize,
    pub character: usize,
}

#[derive(Clone, Debug)]
pub enum TokenData {
    // ->
    Arrow,
    // ++
    Increment,
    // --
    Decrement,
    // ==
    Equivalent,
    // &&
    BooleanAnd,
    // ||
    BooleanOr,
    // ^^
    BooleanXor,
    // +=
    AddAssign,
    // -=
    SubAssign,
    // *=
    MulAssign,
    // /=
    DivAssign,
    // %=
    ModAssign,
    // !=
    NotEqual,
    // <=
    LessThanEqual,
    // >=
    GreaterThanEqual,
    // (
    OpenParenthesis,
    // )
    CloseParenthesis,
    // {
    OpenBody,
    // }
    CloseBody,
    // [
    OpenBracket,
    // ]
    CloseBracket,
    // <
    LessThan,
    // >
    GreaterThan,
    // ;
    Semicolon,
    // ,
    Comma,
    // :
    Colon,
    // !
    Not,
    // &
    // left with vague name due to multiple purposes
    Ampersand,
    // |
    BitwiseOr,
    // ^
    BitwiseXor,
    // +
    Add,
    // -
    Sub,
    // *
    Mul,
    // /
    Div,
    // %
    Mod,
    // =
    Equal,

    // true or false
    BooleanLiteral(bool),
    // "<string>"
    StringLiteral(String),
    // '<character>'
    CharacterLiteral(char),
    // <0-9>...
    NumberLiteral(i64),
    // <A-z or _>...
    Literal(String),
}

#[derive(Debug)]
pub struct TokenizeError {
    pub line: usize,
    pub character: usize,
    pub error: TokenizeErrorType,
}

#[derive(Debug)]
pub enum TokenizeErrorType {
    UnexpectedCharacter(char),
    UnclosedStringLiteral,
    UnclosedCharacterLiteral,
    CharacterLiteralTooLong,
}

pub fn is_whitespace(c: char) -> bool {
    c.is_whitespace()
}

pub fn double_token(c1: char, c2: char) -> Option<TokenData> {
    match c1 {
        _ if c1 == '-' && c2 == '>' => Some(TokenData::Arrow),
        // handle repeated characters
        x if c1 == c2 => {
            match x {
                '+' => Some(TokenData::Increment), // incr
                '-' => Some(TokenData::Decrement), // decr
                '=' => Some(TokenData::Equivalent), // equiv
                '&' => Some(TokenData::BooleanAnd), // bool AND
                '|' => Some(TokenData::BooleanOr), // bool OR
                '^' => Some(TokenData::BooleanXor), // bool XOR
                _ => None,
            }
        }
        // handle things ending in =
        _ if c2 == '=' => {
            match c1 {
                '!' => Some(TokenData::NotEqual), // !=
                '<' => Some(TokenData::LessThanEqual), // <=
                '>' => Some(TokenData::GreaterThanEqual), // >=
                '+' => Some(TokenData::AddAssign), // +=
                '-' => Some(TokenData::SubAssign), // -=
                '*' => Some(TokenData::MulAssign), // *=
                '/' => Some(TokenData::DivAssign), // /=
                '%' => Some(TokenData::ModAssign), // %=
                _ => None,
            }
        }
        _ => None,
    }
}

pub fn single_token(c: char) -> Option<TokenData> {
    match c {
        '(' => Some(TokenData::OpenParenthesis),
        ')' => Some(TokenData::CloseParenthesis),

        '{' => Some(TokenData::OpenBody),
        '}' => Some(TokenData::CloseBody),

        '[' => Some(TokenData::OpenBracket),
        ']' => Some(TokenData::CloseBracket),

        '<' => Some(TokenData::LessThan),
        '>' => Some(TokenData::GreaterThan),

        ';' => Some(TokenData::Semicolon),
        ',' => Some(TokenData::Comma),
        ':' => Some(TokenData::Colon),

        '!' => Some(TokenData::Not),
        '&' => Some(TokenData::Ampersand),
        '|' => Some(TokenData::BitwiseOr),
        '^' => Some(TokenData::BitwiseXor),

        '+' => Some(TokenData::Add),
        '-' => Some(TokenData::Sub),
        '*' => Some(TokenData::Mul),
        '/' => Some(TokenData::Div),
        '%' => Some(TokenData::Mod),

        '=' => Some(TokenData::Equal),
        _ => None,
    }
}

pub fn tokenize(input: String) -> Result<Vec<Token>, TokenizeError> {
    let mut line = 0;
    let mut character = 0;

    let mut tokens = Vec::new();
    let mut chars = input.chars().collect::<VecDeque<char>>();

    let mut comment = false;
    let mut last_was_fslash = false;

    while let Some(c) = chars.pop_front() {
        if comment {
            if c == '\n' {
                comment = false;
                line += 1;
                character = 0;
            }
            continue;
        }
        if c == '/' {
            if last_was_fslash {
                tokens.pop();
                last_was_fslash = false;
                comment = true;
                continue;
            } else {
                last_was_fslash = true;
            }
        } else {
            last_was_fslash = false;
        }

        if c == '\n' {
            line += 1;
            character = 0;
        } else {
            character += 1;
        }
        if is_whitespace(c) {
            continue;
        }
        if let Some(c2) = chars.pop_front() {
            if let Some(c3) = chars.get(0) {
                if c3.is_whitespace() {
                    if let Some(token) = double_token(c, c2) {
                        tokens.push(Token {
                            data: token,
                            line,
                            character,
                        });
                        character += 1;
                        continue;
                    }
                }
            }
            chars.push_front(c2); // only case where we don't need to do this is blocked by a continue
        }

        if let Some(token) = single_token(c) {
            tokens.push(Token {
                data: token,
                line,
                character,
            });
        } else {
            match c {
                '"' => {
                    // string literal
                    let mut str = String::new();
                    let mut found_string_end = false;
                    while let Some(c) = chars.pop_front() {
                        if c == '"' {
                            found_string_end = true;
                            break;
                        }
                        str.push(c);
                    }
                    if !found_string_end {
                        return Err(TokenizeError {
                            line,
                            character,
                            error: TokenizeErrorType::UnclosedStringLiteral,
                        });
                    }
                    // this is done here so that the error shows the opening quote
                    character += str.len() + 1;
                    tokens.push(Token {
                        data: TokenData::StringLiteral(str),
                        line,
                        character,
                    });
                }
                '\'' => {
                    // character literal
                    let clit = chars.pop_front();
                    if clit.is_none() {
                        return Err(TokenizeError {
                            line,
                            character,
                            error: TokenizeErrorType::UnclosedCharacterLiteral,
                        });
                    }
                    let clit = clit.unwrap();
                    let closing = chars.pop_front();
                    if closing.is_none_or(|c| c != '\'') {
                        return Err(TokenizeError {
                            line,
                            character,
                            error: TokenizeErrorType::UnclosedCharacterLiteral,
                        });
                    }
                    // this is done here so that the error shows the opening quote
                    character += 2;
                    tokens.push(Token {
                        data: TokenData::CharacterLiteral(clit),
                        line,
                        character,
                    });
                }
                _ if c.is_ascii_digit() => {
                    let mut digit_str = String::new();
                    digit_str.push(c);
                    while let Some(c) = chars.pop_front() {
                        if c.is_ascii_digit() {
                            digit_str.push(c);
                            character += 1;
                        } else {
                            chars.push_front(c);
                            break;
                        }
                    }
                    let number: i64 = digit_str.parse().expect("bad number literal somehow");
                    tokens.push(Token {
                        data: TokenData::NumberLiteral(number),
                        line,
                        character,
                    });
                }

                _ => {
                    let mut literalstr = String::new();
                    literalstr.push(c);
                    while let Some(c) = chars.pop_front() {
                        if c.is_ascii_alphanumeric() || c == '_' {
                            literalstr.push(c);
                            character += 1;
                        } else {
                            chars.push_front(c);
                            break;
                        }
                    }
                    if literalstr == "true" {
                        tokens.push(Token {
                            data: TokenData::BooleanLiteral(true),
                            line,
                            character,
                        });
                    } else if literalstr == "false" {
                        tokens.push(Token {
                            data: TokenData::BooleanLiteral(false),
                            line,
                            character,
                        });
                    } else {
                        tokens.push(Token {
                            data: TokenData::Literal(literalstr),
                            line,
                            character,
                        });
                    }
                }
            }
        }
    }

    Ok(tokens)
}