use crate::opcode::Opcode;
use crate::utils::evaluate_constant_expression;

#[derive(Debug, Clone, PartialEq)]
pub enum ImmediateValue {
    Int(i64),
    Addr(i64),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Directive(String, usize),
    Global(usize),
    Extern(usize),
    Equ(usize),
    Rodata(usize),
    Label(String, usize),
    Opcode(Opcode, usize),
    Register(String, usize),
    ImmediateValue(ImmediateValue, usize),
    StringLiteral(String, usize),
    Expression(String, usize),
    Comma(usize),

    // for refactoring
    Identifier(String, usize),
    Number(i64, usize),
    Colon(usize),
    LeftBracket(usize),
    RightBracket(usize),
    BinaryOp(String, usize),
}

pub fn tokenize(source: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let mut line_number = 1;

    for line in source.lines() {
        let line = line.trim();

        if line.is_empty() {
            line_number += 1;
            continue;
        }

        // Handle comments - skip rest of line
        let line = if let Some(comment_pos) = line.find("//") {
            &line[..comment_pos].trim()
        } else {
            line
        };

        if line.is_empty() {
            line_number += 1;
            continue;
        }
    
        let mut chars = line.chars().peekable();
    
        // iterate over chars
        while let Some(&c) = chars.peek() {
            match c {
                // handle directives
                '.' => {
                    chars.next();
                    let directive: String = chars.by_ref()
                        .take_while(|&c| c.is_alphanumeric() || c == '_')
                        .collect();
                    // TODO: text section doesn't aslways have a global directive
                    if directive == "global" || directive == "globl" {
                        tokens.push(Token::Global(line_number));
                    } else if directive == "extern" {
                        tokens.push(Token::Extern(line_number));
                    } else if directive == "rodata" {
                        tokens.push(Token::Rodata(line_number));
                    } else if directive == "equ"{
                        tokens.push(Token::Equ(line_number));
                    } else{
                        tokens.push(Token::Directive(directive, line_number));
                    }
                }
                // handle string literals
                '"' => {
                    chars.next(); // consume opening quote
                    let mut string_literal = String::new();
                    while let Some(&c) = chars.peek() {
                        if c == '"' {
                            chars.next(); // consume the closing quote
                            tokens.push(Token::StringLiteral(string_literal, line_number));
                            break;
                        } else if c == '\n' {
                            return Err(format!("Unterminated string literal on line {}", line_number));
                        }
                        string_literal.push(chars.next().unwrap());
                    }
                }
                // handle []
                '[' => {
                    chars.next();
                    let mut string_literal = String::new();
                    while let Some(&c) = chars.peek() {
                        if c == ']' {
                            chars.next(); // consume the closing quote
                            string_literal = evaluate_constant_expression(&string_literal)?;
                            tokens.push(Token::Expression(string_literal, line_number));
                            break;
                        } else if c == '\n' {
                            return Err(format!("Unterminated square bracket on line {}", line_number));
                        }
                        string_literal.push(chars.next().unwrap());
                    }
                }
                // handle comma
                ',' => {
                    chars.next();
                    tokens.push(Token::Comma(line_number));
                }
                // handle comments
                '/' if chars.clone().nth(1) == Some('/') => {
                    chars.by_ref().take_while(|&c| c != '\n').for_each(drop);
                }
                '#' => {
                    chars.by_ref().take_while(|&c| c != '\n').for_each(drop);
                }
                c if c.is_digit(10) => {
                    let number: String = chars.by_ref()
                        .take_while(|&c| c.is_digit(10)).collect();
                    tokens.push(
                        Token::ImmediateValue(
                            ImmediateValue::Int(number.parse::<i64>().map_err(|_| "Invalid number")?),
                            line_number
                        )
                    );
                }

                c if c.is_alphanumeric() || c == '_' => {
                    let identifier: String = chars.by_ref()
                        .take_while(|&c| c.is_alphanumeric() || c == '_' || c == ':')
                        .collect();
                    // Check if the next character is ':' for labels
                    if identifier.ends_with(':') {
                        let label_name = identifier.trim_end_matches(':').to_string(); 
                        tokens.push(Token::Label(label_name, line_number));
                    } else if let Some(Token::Directive(_, _)) = tokens.last() {
                        tokens.push(Token::Label(identifier, line_number));
                    } else if identifier.starts_with('r') //
                            && identifier[1..].chars().all(|c| c.is_digit(10)){
                        tokens.push(Token::Register(identifier, line_number));
                    } else if let Ok(opcode) = Opcode::from_str(&identifier) {
                        tokens.push(Token::Opcode(opcode, line_number));
                    } else {
                        tokens.push(Token::Label(identifier, line_number));
                    }
                }
                c if c.is_whitespace() => {
                    chars.next();
                }
                _ => return Err(format!("Unexpected charcter: '{}' on line {}", c, line_number)),
            }
        }
        line_number += 1;
    }
    
    Ok(tokens)
}