use crate::opcode::Opcode;
use crate::lexer::Token;

pub fn verify_instruction(opcode: &Opcode, operands: &[Token]) -> Result<(), String> {
    match opcode {
        Opcode::Add32 | Opcode::Sub32 | Opcode::Mul32 | Opcode::Div32 | Opcode::Or32 | Opcode::And32 | Opcode::Lsh32 | Opcode::Rsh32 | Opcode::Mod32 | Opcode::Xor32 | Opcode::Mov32 | Opcode::Arsh32 | Opcode::Lmul32 | Opcode::Udiv32 | Opcode::Urem32 | Opcode::Sdiv32 | Opcode::Srem32 | Opcode::Neg32 => {
            if operands.len() != 2 {
                return Err(format!("Expected 2 operands for {:?}, got {}", opcode, operands.len()));
            }
            match (&operands[0], &operands[1]) {
                (Token::Register(_, _), Token::Register(_, _)) => Ok(()),
                (Token::Register(_, _), Token::ImmediateValue(_, _)) => Ok(()),
                _ => Err(format!("Invalid operands for {:?}", opcode)),
            }
        }
        Opcode::Add64 | Opcode::Sub64 | Opcode::Mul64 | Opcode::Div64 | Opcode::Or64 | Opcode::And64 | Opcode::Lsh64 | Opcode::Rsh64 | Opcode::Mod64 | Opcode::Xor64 | Opcode::Mov64 | Opcode::Arsh64 | Opcode::Lmul64 | Opcode::Uhmul64 | Opcode::Udiv64 | Opcode::Urem64 | Opcode::Sdiv64 | Opcode::Srem64 => {
            if operands.len() != 2 {
                return Err(format!("Expected 2 operands for {:?}, got {}", opcode, operands.len()));
            }
            match (&operands[0], &operands[1]) {
                (Token::Register(_, _), Token::Register(_, _)) => Ok(()),
                (Token::Register(_, _), Token::ImmediateValue(_, _)) => Ok(()),
                _ => Err(format!("Invalid operands for {:?}", opcode)),
            }
        }
        Opcode::Jeq | Opcode::Jgt | Opcode::Jge | Opcode::Jlt | Opcode::Jle | Opcode::Jset | Opcode::Jne | Opcode::Jsgt | Opcode::Jsge | Opcode::Jslt | Opcode::Jsle => {
            if operands.len() != 3 {
                return Err(format!("Expected 3 operands for {:?}, got {}", opcode, operands.len()));
            }
            match (&operands[0], &operands[1], &operands[2]) {
                (Token::Register(_, _), Token::Register(_, _), Token::Label(_, _)) => Ok(()),
                (Token::Register(_, _), Token::ImmediateValue(_, _), Token::Label(_, _)) => Ok(()),
                _ => Err(format!("Invalid operands for {:?}", opcode)),
            }
        }
        Opcode::Ja => {
            if operands.len() != 1 {
                return Err(format!("Expected 1 operand for {:?}, got {}", opcode, operands.len()));
            }
            match &operands[0] {
                Token::Label(_, _) | Token::ImmediateValue(_, _) => Ok(()),
                _ => Err(format!("Invalid operand for {:?}", opcode)),
            }
        }
        Opcode::Exit => {
            if !operands.is_empty() {
                return Err(format!("Expected no operands for {:?}, got {}", opcode, operands.len()));
            }
            Ok(())
        }
        Opcode::Call => {
            if operands.len() != 1 {
                return Err(format!("Expected 1 operand for {:?}, got {}", opcode, operands.len()));
            }
            match &operands[0] {
                Token::Label(_, _) => Ok(()),
                _ => Err(format!("Invalid operand for {:?}", opcode)),
            }
        }
        Opcode::Lddw => {
            if operands.len() != 2 {
                return Err(format!("Expected 2 operands for {:?}, got {}", opcode, operands.len()));
            }
            match (&operands[0], &operands[1]) {
                (Token::Register(_, _), Token::ImmediateValue(_, _)) => Ok(()),
                (Token::Register(_, _), Token::Label(_, _)) => Ok(()),
                _ => Err(format!("Invalid operands for {:?}", opcode)),
            }
        }
        // store operations - deprecated
        Opcode::Stb | Opcode::Sth | Opcode::Stw | Opcode::Stdw => {
            Err(format!("{} is deprecated", opcode.to_str()))
        },
        // negate operations - takes register
        Opcode::Neg64 => {
            if operands.len() != 1 {
                return Err(format!("{} reg", opcode.to_str()));
            }
            match &operands[0] {
                Token::Register(_reg, _) => Ok(()),
                _ => Err(format!("{} reg", opcode.to_str())),
            }
        },
        // le be
        Opcode::Le | Opcode::Be => {
            return Err(format!("Unsure how to handle {}", opcode.to_str()));
        },
        _ => Err(format!("Unsupported opcode: {:?}", opcode)),
    }
}