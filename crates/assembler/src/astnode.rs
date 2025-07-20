use crate::opcode::Opcode;
use crate::lexer::{Token, ImmediateValue};
use crate::dynsym::RelocationType;
use crate::debuginfo::{DebugInfo, RegisterHint, RegisterType};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum ASTNode {
    // only present in the AST
    Directive(Directive),
    GlobalDecl(GlobalDecl),
    EquDecl(EquDecl),
    ExternDecl(ExternDecl),
    RodataDecl(RodataDecl),
    Label(Label),
    // present in the bytecode
    Instruction {
        instruction: Instruction,
        offset: u64,
    },
    ROData {
        rodata: ROData,
        offset: u64,
    },
}

#[derive(Debug, Clone)]
pub struct Directive {
    pub name: String,
    pub args: Vec<Token>,
    pub line_number: usize,
}

#[derive(Debug, Clone)]
pub struct GlobalDecl {
    pub entry_label: String,
    pub line_number: usize,
}

impl GlobalDecl {
    pub fn get_entry_label(&self) -> String {
        self.entry_label.clone()
    }
}

#[derive(Debug, Clone)]
pub struct EquDecl {
    pub name: String,
    pub value: Token,
    pub line_number: usize,
}

impl EquDecl {
    pub fn get_name(&self) -> String {
        self.name.clone()
    }
    pub fn get_val(&self) -> ImmediateValue {
        match &self.value {
            Token::ImmediateValue(val, _) => val.clone(),
            _ => panic!("Invalid Equ declaration"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ExternDecl {
    pub args: Vec<Token>,
    pub line_number: usize,
}

#[derive(Debug, Clone)]
pub struct RodataDecl {
    pub line_number: usize,
}

#[derive(Debug, Clone)]
pub struct Label {
    pub name: String,
    pub line_number: usize,
}

#[derive(Debug, Clone)]
pub struct Instruction {
    pub opcode: Opcode,
    pub operands: Vec<Token>,
    pub line_number: usize,
}

impl Instruction {
    pub fn get_size(&self) -> u64 {
        match self.opcode {
            Opcode::Lddw => 16,
            _ => 8,
        }
    }
    pub fn needs_relocation(&self) -> bool {
        match self.opcode {
            Opcode::Call => true,
            Opcode::Lddw => {
                match &self.operands[1] {
                    Token::Identifier(_, _) => true,
                    _ => false,
                }
            },
            _ => false,
        }
    }
    pub fn get_relocation_info(&self) -> (RelocationType, String) {
        match self.opcode {
            Opcode::Lddw => {
                match &self.operands[1] {
                    Token::Identifier(name, _) => (RelocationType::RSbf64Relative, name.clone()),
                    _ => panic!("Expected label operand"),
                }
            },
            _ => {
                if let Token::Identifier(name, _) = &self.operands[0] {
                    (RelocationType::RSbfSyscall, name.clone()) 
                } else {
                    panic!("Expected label operand")
                }
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct ROData {
    pub name: String,
    pub args: Vec<Token>,
    pub line_number: usize,
}

impl ROData {
    pub fn get_size(&self) -> u64 {
        let mut size = 0;
        for arg in &self.args {
            if let Token::StringLiteral(s, _) = arg {
                size += s.len() as u64;
            }
        }
        size
    }
}

impl ASTNode {
    pub fn bytecode_with_debug_map(&self) -> Option<(Vec<u8>, HashMap<u64, DebugInfo>)> {
        match self {
            ASTNode::Instruction { instruction: Instruction { opcode, operands, line_number }, offset } => {
                let mut bytes = Vec::new();
                let mut line_map = HashMap::new();
                let mut debug_map = HashMap::new();
                // Record the start of this instruction
                line_map.insert(*offset, *line_number);
                let mut debug_info = DebugInfo::new(*line_number);
                bytes.push(opcode.to_bytecode());  // 1 byte opcode
                
                if *opcode == Opcode::Call {
                    // currently hardcoded to call sol_log_
                    bytes.extend_from_slice(&[0x10, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF]);
                } else if *opcode == Opcode::Lddw {
                    match &operands[..] {
                        [Token::Register(reg, _), Token::ImmediateValue(imm, _)] => {
                            // 1 byte register number (strip 'r' prefix)
                            bytes.push(*reg);
                            
                            // 2 bytes of zeros (offset/reserved)
                            bytes.extend_from_slice(&[0, 0]);

                            // 8 bytes immediate value in little-endian
                            let imm64 = match imm {
                                ImmediateValue::Int(val) => *val as i64,
                                ImmediateValue::Addr(val) => *val as i64,
                            };
                            bytes.extend_from_slice(&imm64.to_le_bytes()[..4]);
                            bytes.extend_from_slice(&[0, 0, 0, 0]);
                            bytes.extend_from_slice(&imm64.to_le_bytes()[4..8]);
                        }
                        _ => {}
                    }
                } else {
                    match &operands[..] {
                        [Token::ImmediateValue(imm, _)] => {
                            // 1 byte of zeros (no register)
                            bytes.push(0);
                            
                            if *opcode == Opcode::Ja {
                                // 2 bytes immediate value in little-endian for 'ja'
                                let imm16 = match imm {
                                    ImmediateValue::Int(val) => *val as i16,
                                    ImmediateValue::Addr(val) => *val as i16,
                                };
                                bytes.extend_from_slice(&imm16.to_le_bytes());
                            } else {
                                // 4 bytes immediate value in little-endian
                                let imm32 = match imm {
                                    ImmediateValue::Int(val) => *val as i32,
                                    ImmediateValue::Addr(val) => *val as i32,
                                };
                                bytes.extend_from_slice(&imm32.to_le_bytes());
                            }
                        },

                        [Token::Register(reg, _), Token::ImmediateValue(imm, _)] => {
                            // 1 byte register number (strip 'r' prefix)
                            bytes.push(*reg);
                            
                            // 2 bytes of zeros (offset/reserved)
                            bytes.extend_from_slice(&[0, 0]);
                            
                            // 4 bytes immediate value in little-endian
                            let imm32 = match imm {
                                ImmediateValue::Int(val) => *val as i32,
                                ImmediateValue::Addr(val) => {
                                    debug_info.register_hint = RegisterHint {
                                        register: *reg as usize,
                                        register_type: RegisterType::Addr
                                    };
                                    *val as i32
                                }
                            };
                            bytes.extend_from_slice(&imm32.to_le_bytes());
                        },

                        [Token::Register(reg, _), Token::ImmediateValue(imm, _), Token::ImmediateValue(offset, _)] => {
                            // 1 byte register number (strip 'r' prefix)
                            bytes.push(*reg);
                            
                            // 2 bytes of offset in little-endian
                            let offset16 = match offset {
                                ImmediateValue::Int(val) => *val as u16,
                                ImmediateValue::Addr(val) => *val as u16,
                            };
                            bytes.extend_from_slice(&offset16.to_le_bytes());
                            
                            // 4 bytes immediate value in little-endianß
                            let imm32 = match imm {
                                ImmediateValue::Int(val) => *val as i32,
                                ImmediateValue::Addr(val) => {
                                    debug_info.register_hint = RegisterHint {
                                        register: *reg as usize,
                                        register_type: RegisterType::Addr
                                    };
                                    *val as i32
                                }
                            };
                            bytes.extend_from_slice(&imm32.to_le_bytes());
                        },                    
                        
                        [Token::Register(dst, _), Token::Register(src, _)] => {
                            // Convert register strings to numbers
                            let dst_num = dst;
                            let src_num = src;
                            
                            // Combine src and dst into a single byte (src in high nibble, dst in low nibble)
                            let reg_byte = (src_num << 4) | dst_num;
                            bytes.push(reg_byte);
                        },
                        [Token::Register(dst, _), Token::Register(reg, _), Token::ImmediateValue(offset, _)] => {
                            // Combine base register and destination register into a single byte
                            let reg_byte = (reg << 4) | dst;
                            bytes.push(reg_byte);
                            
                            // Add the offset as a 16-bit value in little-endian
                            let offset16 = match offset {
                                ImmediateValue::Int(val) => *val as u16,
                                ImmediateValue::Addr(val) => *val as u16,
                            };
                            bytes.extend_from_slice(&offset16.to_le_bytes());
                        },
                        [Token::Register(reg, _), Token::ImmediateValue(offset, _), Token::Register(dst, _)] => {
                            // Combine base register and destination register into a single byte
                            let reg_byte = (dst << 4) | reg;
                            bytes.push(reg_byte);
                            
                            // Add the offset as a 16-bit value in little-endian
                            let offset16 = match offset {
                                ImmediateValue::Int(val) => *val as u16,
                                ImmediateValue::Addr(val) => *val as u16,
                            };
                            bytes.extend_from_slice(&offset16.to_le_bytes());
                        }
                        
                        _ => {}
                    }
                }

                // Add padding to make it 8 or 16 bytes depending on opcode
                let target_len = if *opcode == Opcode::Lddw { 16 } else { 8 };
                while bytes.len() < target_len {
                    bytes.push(0);
                }

                debug_map.insert(*offset, debug_info);
                
                Some((bytes, debug_map))
            },
            ASTNode::ROData { rodata: ROData { name: _, args, .. }, .. } => {
                let mut bytes = Vec::new();
                let debug_map = HashMap::<u64, DebugInfo>::new();
                for arg in args {
                    if let Token::StringLiteral(s, _) = arg {
                        // Convert string to bytes and add null terminator
                        let str_bytes = s.as_bytes().to_vec();
                        bytes.extend(str_bytes);
                    }
                }
                Some((bytes, debug_map))
            },
            _ => None
        }
    }

    // Keep the old bytecode method for backward compatibility
    pub fn bytecode(&self) -> Option<Vec<u8>> {
        self.bytecode_with_debug_map().map(|(bytes, _)| bytes)
    }
}
