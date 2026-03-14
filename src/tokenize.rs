use std::collections::VecDeque;

#[derive(Clone, Debug)]
pub enum Token {
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

    // "<string>"
    StringLiteral(String),
    // '<character>'
    CharacterLiteral(char),
    // <0-9>...
    NumberLiteral(usize),
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

pub fn double_token(c1: char, c2: char) -> Option<Token> {
    match c1 {
        _ if c1 == '-' && c2 == '>' => Some(Token::Arrow),
        // handle repeated characters
        x if c1 == c2 => {
            match x {
                '+' => Some(Token::Increment), // incr
                '-' => Some(Token::Decrement), // decr
                '=' => Some(Token::Equivalent), // equiv
                '&' => Some(Token::BooleanAnd), // bool AND
                '|' => Some(Token::BooleanOr), // bool OR
                '^' => Some(Token::BooleanXor), // bool XOR
                _ => None,
            }
        }
        // handle things ending in =
        _ if c2 == '=' => {
            match c1 {
                '!' => Some(Token::NotEqual), // !=
                '<' => Some(Token::LessThanEqual), // <=
                '>' => Some(Token::GreaterThanEqual), // >=
                '+' => Some(Token::AddAssign), // +=
                '-' => Some(Token::SubAssign), // -=
                '*' => Some(Token::MulAssign), // *=
                '/' => Some(Token::DivAssign), // /=
                '%' => Some(Token::ModAssign), // %=
                _ => None,
            }
        }
        _ => None,
    }
}

pub fn single_token(c: char) -> Option<Token> {
    match c {
        '(' => Some(Token::OpenParenthesis),
        ')' => Some(Token::CloseParenthesis),

        '{' => Some(Token::OpenBody),
        '}' => Some(Token::CloseBody),

        '[' => Some(Token::OpenBracket),
        ']' => Some(Token::CloseBracket),

        '<' => Some(Token::LessThan),
        '>' => Some(Token::GreaterThan),

        ';' => Some(Token::Semicolon),
        ',' => Some(Token::Comma),
        ':' => Some(Token::Colon),

        '!' => Some(Token::Not),
        '&' => Some(Token::Ampersand),
        '|' => Some(Token::BitwiseOr),
        '^' => Some(Token::BitwiseXor),

        '+' => Some(Token::Add),
        '-' => Some(Token::Sub),
        '*' => Some(Token::Mul),
        '/' => Some(Token::Div),
        '%' => Some(Token::Mod),

        '=' => Some(Token::Equal),
        _ => None,
    }
}

pub fn tokenize(input: String) -> Result<Vec<Token>, TokenizeError> {
    let mut line = 0;
    let mut character = 0;

    let mut tokens = Vec::new();
    let mut chars = input.chars().collect::<VecDeque<char>>();

    while let Some(c) = chars.pop_front() {
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
            if let Some(token) = double_token(c, c2) {
                tokens.push(token);
                character += 1;
                continue;
            } else {
                chars.push_front(c2);
            }
        }

        if let Some(token) = single_token(c) {
            tokens.push(token);
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
                    tokens.push(Token::StringLiteral(str));
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
                    tokens.push(Token::CharacterLiteral(clit));
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
                    let number: usize = digit_str.parse().expect("bad number literal somehow");
                    tokens.push(Token::NumberLiteral(number));
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
                    tokens.push(Token::Literal(literalstr));
                }
            }
        }
    }

    Ok(tokens)
}