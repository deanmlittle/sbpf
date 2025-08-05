use crate::opcode::Opcode;
use crate::errors::CompileError;
use std::ops::Range;

#[derive(Debug, Clone)]
pub enum Op {
    Add,
    Sub,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ImmediateValue {
    Int(i64),
    Addr(i64),
}

impl std::ops::Add for ImmediateValue {
    type Output = ImmediateValue;
    fn add(self, other: Self) -> ImmediateValue {
        match (self, other) {
            (ImmediateValue::Int(a), ImmediateValue::Int(b)) => ImmediateValue::Int(a + b),
            (ImmediateValue::Addr(a), ImmediateValue::Addr(b)) => ImmediateValue::Addr(a + b),
            (ImmediateValue::Int(a), ImmediateValue::Addr(b)) => ImmediateValue::Addr(a + b),
            (ImmediateValue::Addr(a), ImmediateValue::Int(b)) => ImmediateValue::Addr(a + b),
        }
    }
}

impl std::ops::Sub for ImmediateValue {
    type Output = ImmediateValue;
    fn sub(self, other: Self) -> ImmediateValue {
        match (self, other) {
            (ImmediateValue::Int(a), ImmediateValue::Int(b)) => ImmediateValue::Int(a - b),
            (ImmediateValue::Addr(a), ImmediateValue::Addr(b)) => ImmediateValue::Addr(a - b),
            (ImmediateValue::Int(a), ImmediateValue::Addr(b)) => ImmediateValue::Addr(a - b),
            (ImmediateValue::Addr(a), ImmediateValue::Int(b)) => ImmediateValue::Addr(a - b),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Token {
    Directive(String, Range<usize>),
    Label(String, Range<usize>),
    Identifier(String, Range<usize>),
    Opcode(Opcode, Range<usize>),
    Register(u8, Range<usize>),
    ImmediateValue(ImmediateValue, Range<usize>),
    BinaryOp(Op, Range<usize>),
    StringLiteral(String, Range<usize>),

    LeftBracket(Range<usize>),
    RightBracket(Range<usize>),
    Comma(Range<usize>),
    Colon(Range<usize>),
}

pub fn tokenize(source: &str) -> Result<Vec<Token>, CompileError> {
    let mut tokens = Vec::new();
    let mut byte_offset = 0;

    for line in source.lines() {

        if line.is_empty() {
            byte_offset += 1;
            continue;
        }

        let mut chars = line.char_indices().peekable();
        while let Some((start_idx, c)) = chars.peek() {
            let token_start = byte_offset + start_idx;
            match c {
                c if c.is_ascii_digit() => {
                    let mut number = String::new();
                    let mut is_addr = false;
                    while let Some((_, c)) = chars.peek() {
                        if c.is_digit(10) {
                            number.push(chars.next().unwrap().1);
                        } else if number == "0" && *c == 'x' {
                            chars.next();
                            is_addr = true; /*  */ number = String::new();
                        } else if is_addr && (*c == 'a' || *c == 'b' || *c == 'c' || *c == 'd' || *c == 'e' || *c == 'f') {
                            number.push(chars.next().unwrap().1);
                        } else {
                            break;
                        }
                    }
                    let span = token_start..token_start + number.len();
                    if is_addr {
                        tokens.push(Token::ImmediateValue(ImmediateValue::Addr(i64::from_str_radix(&number, 16).map_err(|_| CompileError::InvalidNumber { number, span: span.clone() })?), span.clone())); 
                    } else {
                        tokens.push(Token::ImmediateValue(ImmediateValue::Int(number.parse::<i64>().map_err(|_| CompileError::InvalidNumber { number, span: span.clone() })?), span.clone()));
                    }      
                }

                c if c.is_ascii_alphanumeric() || *c == '_' => {
                    let mut identifier = String::new();
                    while let Some((_, c)) = chars.peek() {
                        if c.is_ascii_alphanumeric() || *c == '_' || *c == ':' {
                            identifier.push(chars.next().unwrap().1);
                        } else {
                            break;
                        }
                    }
                    let span = token_start..token_start + identifier.len();
                    if identifier.ends_with(':') {
                        let label_name = identifier.trim_end_matches(':').to_string();
                        tokens.push(Token::Label(label_name, span));
                    } else if identifier.starts_with('r') && identifier[1..].chars().all(|c| c.is_ascii_digit()) {
                        tokens.push(Token::Register(identifier[1..].parse::<u8>().map_err(|_| CompileError::InvalidRegister { register: identifier, span: span.clone() })?, span.clone()));
                    } else if let Ok(opcode) = Opcode::from_str(&identifier) {
                        tokens.push(Token::Opcode(opcode, span));
                    } else {
                        tokens.push(Token::Identifier(identifier, span));
                    }
                }
                c if c.is_whitespace() => {
                    chars.next();
                }
                '+' => {
                    chars.next();
                    let span = token_start..token_start + 1;
                    tokens.push(Token::BinaryOp(Op::Add, span));
                }
                '-' => {
                    chars.next();
                    let span = token_start..token_start + 1;
                    tokens.push(Token::BinaryOp(Op::Sub, span));
                }
                '.' => {
                    chars.next();
                    let directive: String = chars.by_ref()
                        .take_while(|(_, c)| c.is_ascii_alphanumeric() || *c == '_')
                        .map(|(_, c)| c)
                        .collect();
                    let span = token_start..token_start + directive.len() + 1;
                    tokens.push(Token::Directive(directive, span));
                }
                '"' => {
                    chars.next();
                    let mut string_literal = String::new();
                    while let Some((_, c)) = chars.peek() {
                        if *c == '"' {
                            chars.next();
                            let span = token_start..token_start + string_literal.len() + 2;
                            tokens.push(Token::StringLiteral(string_literal, span));
                            break;
                        } else if *c == '\n' {
                            return Err(CompileError::UnterminatedStringLiteral { span: token_start..token_start + 1 });
                        }
                        string_literal.push(chars.next().unwrap().1);
                    }
                }
                '[' => {
                    chars.next();
                    let span = token_start..token_start + 1;
                    tokens.push(Token::LeftBracket(span));
                }
                ']' => {
                    chars.next();
                    let span = token_start..token_start + 1;
                    tokens.push(Token::RightBracket(span));
                }
                ',' => {
                    chars.next();
                    let span = token_start..token_start + 1;
                    tokens.push(Token::Comma(span));
                }
                // handle comments
                '#' => {
                    chars.next();
                    break;
                }
                '/' => {
                    chars.next();
                    if let Some((_, '/')) = chars.peek() {
                        chars.next();
                        break;
                    } else {
                        let span = token_start..token_start + 1;
                        return Err(CompileError::UnexpectedCharacter { character: '/', span });
                    }
                }
                _ => {
                    let span = token_start..token_start + 1;
                    return Err(CompileError::UnexpectedCharacter { character: *c, span });
                }
            }
        }
        byte_offset += line.len() + 1;
    }
    Ok(tokens)
}
