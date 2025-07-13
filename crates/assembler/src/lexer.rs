use crate::opcode::Opcode;

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
            _ => panic!("Invalid addition of ImmediateValue"),
        }
    }
}

impl std::ops::Sub for ImmediateValue {
    type Output = ImmediateValue;
    fn sub(self, other: Self) -> ImmediateValue {
        match (self, other) {
            (ImmediateValue::Int(a), ImmediateValue::Int(b)) => ImmediateValue::Int(a - b),
            _ => panic!("Invalid subtraction of ImmediateValue"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Token {
    Directive(String, usize),
    Label(String, usize),
    Identifier(String, usize),
    Opcode(Opcode, usize),
    Register(u8, usize),
    ImmediateValue(ImmediateValue, usize),
    BinaryOp(Op, usize),
    StringLiteral(String, usize),

    LeftBracket(usize),
    RightBracket(usize),
    Comma(usize),
    Colon(usize),
}

pub fn tokenize(source: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let mut line_number = 1;

    for line in source.lines() {

        // Handle comments - skip rest of line
        let line = if let Some(comment_pos) = line.find("//") {
            &line[..comment_pos].trim()
        } else if let Some(comment_pos) = line.find("#") {
            &line[..comment_pos].trim()
        } else {
            line.trim()
        };

        if line.is_empty() {
            line_number += 1;
            continue;
        }

        let mut chars = line.chars().peekable();

        while let Some(&c) = chars.peek() {
            match c {
                c if c.is_digit(10) => {
                    let mut number = String::new();
                    let mut isAddr = false;
                    while let Some(&c) = chars.peek() {
                        if c.is_digit(10) {
                            number.push(chars.next().unwrap());
                        } else if number == "0" && c == 'x' {
                            chars.next();
                            isAddr = true; /*  */ number = String::new();
                        } else if isAddr && (c == 'a' || c == 'b' || c == 'c' || c == 'd' || c == 'e' || c == 'f') {
                            number.push(chars.next().unwrap());
                        } else {
                            break;
                        }
                    }
                    if isAddr {
                        tokens.push(Token::ImmediateValue(ImmediateValue::Addr(i64::from_str_radix(&number, 16).map_err(|_| "Invalid number")?), line_number)); 
                    } else {
                        tokens.push(Token::ImmediateValue(ImmediateValue::Int(number.parse::<i64>().map_err(|_| "Invalid number")?), line_number));
                    }      
                }

                // TODO: add address and syscall tokens
                c if c.is_alphanumeric() || c == '_' => {
                    let mut identifier = String::new();
                    while let Some(&c) = chars.peek() {
                        if c.is_alphanumeric() || c == '_' || c == ':' {
                            identifier.push(chars.next().unwrap());
                        } else {
                            break;
                        }
                    }
                    if identifier.ends_with(':') {
                        let label_name = identifier.trim_end_matches(':').to_string();
                        tokens.push(Token::Label(label_name, line_number));
                    } else if identifier.starts_with('r') && identifier[1..].chars().all(|c| c.is_digit(10)) {
                        tokens.push(Token::Register(identifier[1..].parse::<u8>().map_err(|_| "Invalid register")?, line_number));
                    } else if let Ok(opcode) = Opcode::from_str(&identifier) {
                        tokens.push(Token::Opcode(opcode, line_number));
                    } else {
                        tokens.push(Token::Identifier(identifier, line_number));
                    }
                }
                c if c.is_whitespace() => {
                    chars.next();
                }
                '+' => {
                    chars.next();
                    tokens.push(Token::BinaryOp(Op::Add, line_number));
                }
                '-' => {
                    chars.next();
                    tokens.push(Token::BinaryOp(Op::Sub, line_number));
                }
                '.' => {
                    chars.next();
                    let directive: String = chars.by_ref()
                        .take_while(|&c| c.is_alphanumeric() || c == '_')
                        .collect();
                    tokens.push(Token::Directive(directive, line_number));
                }
                '"' => {
                    chars.next();
                    let mut string_literal = String::new();
                    while let Some(&c) = chars.peek() {
                        if c == '"' {
                            chars.next();
                            tokens.push(Token::StringLiteral(string_literal, line_number));
                            break;
                        } else if c == '\n' {
                            return Err(format!("Unterminated string literal on line {}", line_number));
                        }
                        string_literal.push(chars.next().unwrap());
                    }
                }
                '[' => {
                    chars.next();
                    tokens.push(Token::LeftBracket(line_number));
                }
                ']' => {
                    chars.next();
                    tokens.push(Token::RightBracket(line_number));
                }
                ',' => {
                    chars.next();
                    tokens.push(Token::Comma(line_number));
                }
                _ => {
                    return Err(format!("Unexpected character: '{}' on line {}", c, line_number));
                }
            }
        }
        line_number += 1;
    }
    Ok(tokens)
}