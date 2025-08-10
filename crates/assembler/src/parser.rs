use crate::lexer::Op;
use crate::opcode::Opcode;
use crate::lexer::{Token, ImmediateValue};
use crate::section::{CodeSection, DataSection};
use crate::astnode::{ASTNode, Directive, GlobalDecl, EquDecl, ExternDecl, RodataDecl, Label, Instruction, ROData};
use crate::dynsym::{DynamicSymbolMap, RelDynMap, RelocationType};
use codespan_reporting::files::SimpleFile;
use num_traits::FromPrimitive;
use std::collections::HashMap;
use crate::errors::CompileError;
use crate::messages::*;
use crate::bug;

pub struct Parser<> {
    tokens: Vec<Token>,

    pub m_prog_is_static: bool,
    pub m_accum_offset: u64,

    // TODO: consolidate all temporary parsing related informaion
    m_const_map: HashMap<String, ImmediateValue>,
    m_label_offsets: HashMap<String, u64>,

    // TODO: consolidate all dynamic symbol information to one big map
    m_entry_label: Option<String>,
    m_dynamic_symbols: DynamicSymbolMap,
    m_rel_dyns: RelDynMap,

    m_rodata_size: u64,
    m_file: Option<SimpleFile<String, String>>,
}

pub struct ParseResult {
    // TODO: parse result is basically 1. static part 2. dynamic part of the program
    pub code_section: CodeSection,

    pub data_section: DataSection,

    pub dynamic_symbols: DynamicSymbolMap,

    pub relocation_data: RelDynMap,

    // TODO: this can be removed and dynamic-ness should just be 
    // determined by if there's any dynamic symbol
    pub prog_is_static: bool,
}

// for now, we only return one error per parse for simpler error handling
pub trait Parse {
    fn parse(tokens: &[Token]) -> Result<(Self, &[Token]), CompileError>
        where Self: Sized;
}

pub trait ParseInstruction {
    fn parse_instruction<'a>(tokens: &'a [Token], const_map: &HashMap<String, ImmediateValue>) -> Result<(Self, &'a [Token]), CompileError>
        where Self: Sized;
}

impl Parse for GlobalDecl {
    fn parse(tokens: &[Token]) -> Result<(Self, &[Token]), CompileError> {
        let Token::Directive(_, span) = &tokens[0] else { bug!("GlobalDecl not a valid directive") };
        if tokens.len() < 2 {
            return Err(CompileError::InvalidGlobalDecl { span: span.clone(), custom_label: None });
        }
        match &tokens[1] {
            Token::Identifier(name, span) => Ok((
                GlobalDecl {
                    entry_label: name.clone(), 
                    span: span.clone()
                },
                &tokens[2..])),
            _ => Err(CompileError::InvalidGlobalDecl { span: span.clone(), custom_label: None }),
        }
    }
}

impl Parse for EquDecl {
    fn parse(tokens: &[Token]) -> Result<(Self, &[Token]), CompileError> {
        let Token::Directive(_, span) = &tokens[0] else { bug!("EquDecl not a valid directive") };
        if tokens.len() < 3 {
            return Err(CompileError::InvalidEquDecl { span: span.clone(), custom_label: Some(EXPECTS_MORE_OPERAND.to_string()) });
        }
        match (
            &tokens[1],
            &tokens[2],
            &tokens[3],
        ) {
            (
                Token::Identifier(name, span),
                Token::Comma(_),
                Token::ImmediateValue(_value, _)
            ) => {
                Ok((
                    EquDecl {
                        name: name.clone(),
                        value: tokens[3].clone(),
                        span: span.clone()
                    },
                    &tokens[4..]
                ))
            }
            _ => Err(CompileError::InvalidEquDecl { span: span.clone(), custom_label: Some(EXPECTS_IDEN_COM_IMM.to_string()) }),
        }
    }
}

impl Parse for ExternDecl {
    fn parse(tokens: &[Token]) -> Result<(Self, &[Token]), CompileError> {
        let Token::Directive(_, span) = &tokens[0] else { bug!("ExternDecl not a valid directive") };
        if tokens.len() < 2 {
            return Err(CompileError::InvalidExternDecl { span: span.clone(), custom_label: Some(EXPECTS_MORE_OPERAND.to_string()) });
        }
        let mut args = Vec::new();
        let mut i = 1;
        while i < tokens.len() {
            match &tokens[i] {
                Token::Identifier(name, span) => {
                    args.push(Token::Identifier(name.clone(), span.clone()));
                    i += 1;
                }
                _ => {
                    break;
                }
            }
        }
        //
        if args.is_empty() {
            Err(CompileError::InvalidExternDecl { span: span.clone(), custom_label: Some(EXPECTS_IDEN.to_string()) })
        } else {
            Ok((
                ExternDecl { 
                    args, 
                    span: span.clone()
                },
                &tokens[i..]
            ))
        }
    }
}

impl Parse for ROData {
    fn parse(tokens: &[Token]) -> Result<(Self, &[Token]), CompileError> {
        let Token::Directive(_, span) = &tokens[0] else { bug!("ROData not a valid directive") };
        if tokens.len() < 3 {
            return Err(CompileError::InvalidRodataDecl { span: span.clone(), custom_label: Some(EXPECTS_MORE_OPERAND.to_string()) });
        }

        let mut args = Vec::new();
        match (
            &tokens[0],
            &tokens[1],
            &tokens[2],
        ) {
            (
                Token::Label(name, span),
                Token::Directive(_, _),
                Token::StringLiteral(_, _)
            ) => {
                args.push(tokens[1].clone());
                args.push(tokens[2].clone());
                Ok((
                    ROData {
                        name: name.clone(),
                        args,
                        span: span.clone()
                    },
                    &tokens[3..]
                ))
            }
            _ => Err(CompileError::InvalidRodataDecl { span: span.clone(), custom_label: Some(EXPECTS_LABEL_DIR_STR.to_string()) }),
        }
    }
}

impl ParseInstruction for Instruction {
    fn parse_instruction<'a>(tokens: &'a [Token], const_map: &HashMap<String, ImmediateValue>) -> Result<(Self, &'a [Token]), CompileError> {
        let next_token_num;
        match &tokens[0] {
            Token::Opcode(opcode, span) => {
                let mut opcode = opcode.clone();
                let mut operands = Vec::new();
                match opcode {
                    Opcode::Lddw => {
                        if tokens.len() < 4 {
                            return Err(
                                CompileError::InvalidInstruction {  // 
                                    instruction: opcode.to_string() //
                                    , span: span.clone()            //
                                    , custom_label: Some(EXPECTS_MORE_OPERAND.to_string()) });
                        }
                        let (value, advance_token_num) = inline_and_fold_constant(tokens, const_map, 3);
                        if let Some(value) = value {
                            match (
                                &tokens[1],
                                &tokens[2],
                                // Third operand is folded to an immediate value
                            ) {
                                (
                                    Token::Register(_, _),
                                    Token::Comma(_),
                                    // Third operand is folded to an immediate value
                                ) => {
                                    operands.push(tokens[1].clone());
                                    operands.push(Token::ImmediateValue(value, span.clone()));
                                }
                                _ => {
                                    return Err(
                                        CompileError::InvalidInstruction {  //
                                            instruction: opcode.to_string() //
                                            , span: span.clone()            //
                                            , custom_label: Some(EXPECTS_REG_COM_IMM_OR_IDEN.to_string()) });
                                }
                            }
                            next_token_num = advance_token_num;
                        } else {
                            match (
                                &tokens[1],
                                &tokens[2],
                                &tokens[3],
                            ) {
                                (
                                    Token::Register(_, _),
                                    Token::Comma(_),
                                    Token::Identifier(_, _)
                                ) => {
                                    operands.push(tokens[1].clone());
                                    operands.push(tokens[3].clone());
                                }
                                _ => {
                                    return Err(
                                        CompileError::InvalidInstruction {  //
                                            instruction: opcode.to_string() //
                                            , span: span.clone()            //
                                            , custom_label: Some(EXPECTS_REG_COM_IMM_OR_IDEN.to_string()) });
                                }
                            }
                            next_token_num = 4;
                        }
                    }
                    Opcode::Ldxw | Opcode::Ldxh | Opcode::Ldxb | Opcode::Ldxdw => {
                        if tokens.len() < 8 {
                            return Err(
                                CompileError::InvalidInstruction {  //
                                    instruction: opcode.to_string() //
                                    , span: span.clone()            //
                                    , custom_label: Some(EXPECTS_MORE_OPERAND.to_string()) });
                        }
                        let (value, advance_token_num) = inline_and_fold_constant(tokens, const_map, 6);
                        if let Some(value) = value {
                            match (
                                &tokens[1],
                                &tokens[2],
                                &tokens[3],
                                &tokens[4],
                                &tokens[5],
                                // Sixth operand is folded to an immediate value
                                &tokens[advance_token_num],
                            ) {
                                (
                                    Token::Register(_, _),
                                    Token::Comma(_),
                                    Token::LeftBracket(_),
                                    Token::Register(_, _),
                                    Token::BinaryOp(_, _),
                                    // Sixth operand is folded to an immediate value 
                                    Token::RightBracket(_)
                                ) => {
                                    operands.push(tokens[1].clone());
                                    operands.push(tokens[4].clone());
                                    operands.push(Token::ImmediateValue(value, span.clone()));                                    
                                }
                                _ => {
                                    return Err(
                                        CompileError::InvalidInstruction {  //
                                            instruction: opcode.to_string() //
                                            , span: span.clone()            //
                                            , custom_label: Some(EXPECTS_REG_COM_LB_REG_BIOP_IMM_RB.to_string()) });
                                }
                            }
                            next_token_num = advance_token_num + 1;
                        } else {
                            return Err(
                                CompileError::InvalidInstruction {  //
                                    instruction: opcode.to_string() //
                                    , span: span.clone()            //
                                    , custom_label: Some(EXPECTS_REG_COM_LB_REG_BIOP_IMM_RB.to_string()) });
                        }
                    }
                    Opcode::Stw | Opcode::Sth | Opcode::Stb | Opcode::Stdw
                    | Opcode::Stxb | Opcode::Stxh | Opcode::Stxw | Opcode::Stxdw => {
                        if tokens.len() < 8 {
                            return Err(
                                CompileError::InvalidInstruction {  //
                                    instruction: opcode.to_string() //
                                    , span: span.clone()            //
                                    , custom_label: Some(EXPECTS_MORE_OPERAND.to_string()) });
                        }
                        let (value, advance_token_num) = inline_and_fold_constant(tokens, const_map, 4);
                        if let Some(value) = value {
                            match (
                                &tokens[1],
                                &tokens[2],
                                &tokens[3],
                                // Fourth operand is folded to an immediate value
                                &tokens[advance_token_num],
                                &tokens[advance_token_num + 1],
                                &tokens[advance_token_num + 2],
                            ) {
                                (
                                    Token::LeftBracket(_),
                                    Token::Register(_, _),
                                    Token::BinaryOp(_, _),
                                    // Fourth operand is folded to an immediate value
                                    Token::RightBracket(_),
                                    Token::Comma(_),
                                    Token::Register(_, _)
                                ) => {
                                    operands.push(tokens[2].clone());
                                    operands.push(Token::ImmediateValue(value, span.clone()));
                                    operands.push(tokens[advance_token_num + 2].clone());
                                }
                                _ => {
                                    return Err(
                                        CompileError::InvalidInstruction {  //
                                            instruction: opcode.to_string() //
                                            , span: span.clone()            //
                                            , custom_label: Some(EXPECTS_LB_REG_BIOP_IMM_RB_COM_REG.to_string()) });
                                }
                            }
                            next_token_num = advance_token_num + 3;
                        } else {
                            return Err(
                                CompileError::InvalidInstruction {  //
                                    instruction: opcode.to_string() //
                                    , span: span.clone()            //
                                    , custom_label: Some(EXPECTS_LB_REG_BIOP_IMM_RB_COM_REG.to_string()) });
                        }
                    }
                    Opcode::Add32 | Opcode::Sub32 | Opcode::Mul32 
                    | Opcode::Div32 | Opcode::Or32 | Opcode::And32 
                    | Opcode::Lsh32 | Opcode::Rsh32 | Opcode::Mod32 
                    | Opcode::Xor32 | Opcode::Mov32 | Opcode::Arsh32 
                    | Opcode::Lmul32 | Opcode::Udiv32 | Opcode::Urem32 
                    | Opcode::Sdiv32 | Opcode::Srem32 | Opcode::Neg32
                    | Opcode::Add64 | Opcode::Sub64 | Opcode::Mul64 
                    | Opcode::Div64 | Opcode::Or64 | Opcode::And64 
                    | Opcode::Lsh64 | Opcode::Rsh64 | Opcode::Mod64 
                    | Opcode::Xor64 | Opcode::Mov64 | Opcode::Arsh64 
                    | Opcode::Lmul64 | Opcode::Uhmul64 | Opcode::Udiv64 
                    | Opcode::Urem64 | Opcode::Sdiv64 | Opcode::Srem64 => {
                        if tokens.len() < 4 {
                            return Err(
                                CompileError::InvalidInstruction {  //
                                    instruction: opcode.to_string() //
                                    , span: span.clone()            //
                                    , custom_label: Some(EXPECTS_MORE_OPERAND.to_string()) });
                        }
                        let (value, advance_token_num) = inline_and_fold_constant(tokens, const_map, 3);
                        if let Some(value) = value {
                            match (
                                &tokens[1],
                                &tokens[2],
                                // Third operand is folded to an immediate value
                            ) {
                                (
                                    Token::Register(_, _),
                                    Token::Comma(_),
                                    // Third operand is folded to an immediate value
                                ) => {
                                    opcode = FromPrimitive::from_u8((opcode as u8) + 1).expect("Invalid opcode conversion"); 
                                    operands.push(tokens[1].clone());
                                    operands.push(Token::ImmediateValue(value, span.clone()));
                                }
                                _ => {
                                    return Err(
                                        CompileError::InvalidInstruction {  //
                                            instruction: opcode.to_string() //
                                            , span: span.clone()            //
                                            , custom_label: Some(EXPECTS_REG_COM_IMM.to_string()) });
                                }
                            } 
                            next_token_num = advance_token_num;
                        } else {
                            match (
                                &tokens[1],
                                &tokens[2],
                                &tokens[3],
                            ) {
                                (
                                    Token::Register(_, _),
                                    Token::Comma(_),
                                    Token::Register(_, _)
                                ) => {
                                    opcode = FromPrimitive::from_u8((opcode as u8) + 2).expect("Invalid opcode conversion"); 
                                    operands.push(tokens[1].clone());
                                    operands.push(tokens[3].clone());
                                }
                                _ => {
                                    return Err(
                                        CompileError::InvalidInstruction {  //
                                            instruction: opcode.to_string() //
                                            , span: span.clone()            //
                                            , custom_label: Some(EXPECTS_REG_COM_REG.to_string()) });
                                }
                            }                           
                            next_token_num = 4;
                        }
                    }
                    Opcode::Jeq | Opcode::Jgt | Opcode::Jge
                    | Opcode::Jlt | Opcode::Jle | Opcode::Jset
                    | Opcode::Jne | Opcode::Jsgt | Opcode::Jsge
                    | Opcode::Jslt | Opcode::Jsle => {
                        if tokens.len() < 6 {
                            return Err(
                                CompileError::InvalidInstruction {  //
                                    instruction: opcode.to_string() //
                                    , span: span.clone()            //
                                    , custom_label: Some(EXPECTS_MORE_OPERAND.to_string()) });
                        }
                        let (value, advance_token_num) = inline_and_fold_constant(tokens, const_map, 3);
                        if let Some(value) = value {
                            match (
                                &tokens[1],
                                &tokens[2],
                                // Third operand is folded to an immediate value
                                &tokens[advance_token_num],
                                &tokens[advance_token_num + 1],
                            ) {
                                (
                                    Token::Register(_, _),
                                    Token::Comma(_),
                                    // Third operand is folded to an immediate value
                                    Token::Comma(_),
                                    Token::Identifier(_, _)
                                ) => {
                                    opcode = FromPrimitive::from_u8((opcode as u8) + 1).expect("Invalid opcode conversion"); 
                                    operands.push(tokens[1].clone());
                                    operands.push(Token::ImmediateValue(value, span.clone()));
                                    operands.push(tokens[advance_token_num + 1].clone());
                                }
                                _ => {
                                    return Err(
                                        CompileError::InvalidInstruction {  //
                                            instruction: opcode.to_string() //
                                            , span: span.clone()            //
                                            , custom_label: Some(EXPECTS_REG_COM_IMM_COM_IMM_OR_IDEN.to_string()) });
                                }
                            }
                            next_token_num = advance_token_num + 2;
                        } else {
                            match (
                                &tokens[1],
                                &tokens[2],
                                &tokens[3],
                                &tokens[4],
                                &tokens[5],
                            ) {
                                (
                                    Token::Register(_, _),
                                    Token::Comma(_),
                                    Token::Register(_, _),
                                    Token::Comma(_),
                                    Token::Identifier(_, _)
                                ) => {
                                    // turn "invalid opcode" to a bug
                                    opcode = FromPrimitive::from_u8((opcode as u8) + 2).expect("Invalid opcode conversion"); 
                                    operands.push(tokens[1].clone());
                                    operands.push(tokens[3].clone());
                                    operands.push(tokens[5].clone());
                                }
                                _ => {
                                    return Err(
                                        CompileError::InvalidInstruction {  //
                                            instruction: opcode.to_string() //
                                            , span: span.clone()            //
                                            , custom_label: Some(EXPECTS_REG_COM_IMM_COM_IMM_OR_IDEN.to_string()) });
                                }
                            }
                            next_token_num = 6;
                        }
                    }
                    Opcode::Ja => {
                        if tokens.len() < 2 {
                            return Err(
                                CompileError::InvalidInstruction {  //
                                    instruction: opcode.to_string() //
                                    , span: span.clone()            //
                                    , custom_label: Some(EXPECTS_MORE_OPERAND.to_string()) });
                        }
                        let (value, advance_token_num) = inline_and_fold_constant(tokens, const_map, 1);
                        if let Some(value) = value {
                            operands.push(Token::ImmediateValue(value, span.clone()));
                            next_token_num = advance_token_num;
                        } else {
                            match &tokens[1] {
                                Token::Identifier(_, _) => {
                                    operands.push(tokens[1].clone());
                                }
                                _ => {
                                    return Err(
                                        CompileError::InvalidInstruction {  //
                                            instruction: opcode.to_string() //
                                            , span: span.clone()            //
                                            , custom_label: Some(EXPECTS_IDEN.to_string()) });
                                }
                            }
                            next_token_num = 2;
                        }
                    }
                    Opcode::Call => {
                        if tokens.len() < 2 {
                            return Err(
                                CompileError::InvalidInstruction {  //
                                    instruction: opcode.to_string() //
                                    , span: span.clone()            //
                                    , custom_label: Some(EXPECTS_MORE_OPERAND.to_string()) });
                        }
                        match &tokens[1] {
                            Token::Identifier(_, _) => {
                                operands.push(tokens[1].clone());
                            }
                            _ => {
                                return Err(
                                    CompileError::InvalidInstruction {  //
                                        instruction: opcode.to_string() //
                                        , span: span.clone()            //
                                        , custom_label: Some(EXPECTS_IDEN.to_string()) });
                            }
                        }
                        next_token_num = 2;
                    }
                    Opcode::Exit => {
                        next_token_num = 1;
                    }
                    _ => {
                        bug!("invalid opcode: {}", opcode.to_str());
                    }
                }
                Ok((
                    Instruction {
                        opcode,
                        operands,
                        span: span.clone()
                    },
                    &tokens[next_token_num..]
                ))
            }
            _ => {
                bug!("invalid instruction");
            }
        }
        
    }
}

fn inline_and_fold_constant_helper(tokens: &[Token]                             //
                                , const_map: &HashMap<String, ImmediateValue>   //
                                , value: ImmediateValue                         //
                                , idx: usize) -> (Option<ImmediateValue>, usize) {
    if tokens.len() < idx + 3 {
        return (Some(value), idx + 1);
    }

    match (
        &tokens[idx + 1],
        &tokens[idx + 2],
    ) {
        (
            Token::BinaryOp(op, _),
            Token::ImmediateValue(value2, _)
        ) => {
            let result = match op {
                Op::Add => {
                    value + value2.clone()
                }
                Op::Sub => {
                    value - value2.clone()
                }
            };
            inline_and_fold_constant_helper(tokens, const_map, result, idx + 2)
        }
        _ => (Some(value), idx + 1),
    }
}

fn inline_and_fold_constant(tokens: &[Token]                            //
                        , const_map: &HashMap<String, ImmediateValue>   //
                        , idx: usize) -> (Option<ImmediateValue>, usize) {
    let value = match &tokens[idx] {
        Token::ImmediateValue(value, _) => value.clone(),
        Token::Identifier(name, _) => {
            if let Some(val) = const_map.get(name) {
                val.clone()
            } else {
                return (None, idx + 1);
            }
        },
        _ => return (None, idx + 1),
    };
    inline_and_fold_constant_helper(tokens, const_map, value, idx)
}

impl Parser {

    pub fn new(tokens: Vec<Token>, file: &SimpleFile<String, String>) -> Self {
        Self { tokens
            , m_prog_is_static: true
            , m_accum_offset: 0
            , m_entry_label: None
            , m_const_map: HashMap::new()
            , m_label_offsets: HashMap::new()
            , m_rodata_size: 0
            , m_dynamic_symbols: DynamicSymbolMap::new()
            , m_rel_dyns: RelDynMap::new()
            , m_file: Some(file.clone())
        }
    }

    pub fn parse(&mut self) -> Result<ParseResult, Vec<CompileError>> {
        let mut nodes = Vec::new();
        let mut rodata_nodes = Vec::new();
        let mut rodata_phase = false;

        let mut errors = Vec::new();

        let mut tokens = self.tokens.as_slice();

        // TODO: when parse error occurs, we should probably just jump to the next line
        // if we're able to error out the scenario where users put 2 instructions in the same line
        // for now we just continue to the next token

        // TODO: it would be nice if we build a token iterator that can 
        // 1. peek the next multiple tokens (for detecting patterns)
        // 2. jump to the next line
        // 3. continue to the next token
        while !tokens.is_empty() {
            match &tokens[0] {
                Token::Directive(name, span) => {
                    match name.as_str() {
                        "global" | "globl" => {
                            match GlobalDecl::parse(tokens) {
                                Ok((node, rest)) => {
                                self.m_entry_label = Some(node.get_entry_label());
                                nodes.push(ASTNode::GlobalDecl { global_decl: node });
                                tokens = rest;
                                }
                                Err(e) => {
                                    errors.push(e);
                                    tokens = &tokens[1..];
                                }
                            }
                        }
                        "extern" => {
                            match ExternDecl::parse(tokens) {
                                Ok((node, rest)) => {
                                nodes.push(ASTNode::ExternDecl { extern_decl: node });
                                tokens = rest;
                                }
                                Err(e) => {
                                    errors.push(e);
                                    tokens = &tokens[1..];
                                }
                            }
                        }
                        "rodata" => {
                            nodes.push(ASTNode::RodataDecl { rodata_decl: RodataDecl { span: span.clone() } });
                            rodata_phase = true;
                            tokens = &tokens[1..];
                        }
                        "equ" => {
                            match EquDecl::parse(tokens) {
                                Ok((node, rest)) => {
                                self.m_const_map.insert(node.get_name(), node.get_val());
                                nodes.push(ASTNode::EquDecl { equ_decl: node });
                                tokens = rest;
                                }
                                Err(e) => {
                                    errors.push(e);
                                    tokens = &tokens[1..];
                                }
                            }
                        }
                        "section" => {
                            nodes.push(ASTNode::Directive { directive: Directive { name: name.clone(), args: Vec::new(), span: span.clone() } });
                            tokens = &tokens[1..];
                        }
                        _ => {
                            errors.push(CompileError::InvalidDirective { directive: name.clone(), span: span.clone(), custom_label: None });
                            tokens = &tokens[1..];
                        }
                    }
                }
                Token::Label(name, span) => {
                    if rodata_phase {
                        match ROData::parse(tokens) {
                            Ok((rodata, rest)) => {
                            self.m_label_offsets.insert(name.clone(), self.m_accum_offset + self.m_rodata_size);
                            self.m_rodata_size += rodata.get_size();
                            rodata_nodes.push(ASTNode::ROData { rodata, offset: self.m_accum_offset });
                            tokens = rest;
                            }
                            Err(e) => {
                                errors.push(e);
                                tokens = &tokens[1..];
                            }
                        }
                    } else {
                        self.m_label_offsets.insert(name.clone(), self.m_accum_offset);
                        nodes.push(ASTNode::Label { label: Label { name: name.clone(), span: span.clone() } });
                        tokens = &tokens[1..];
                    }
                }
                Token::Opcode(_, _) => {
                    match Instruction::parse_instruction(tokens, &self.m_const_map) {
                        Ok((inst, rest)) => {
                            if inst.needs_relocation() {
                                self.m_prog_is_static = false;
                                let (reloc_type, label) = inst.get_relocation_info();
                                self.m_rel_dyns.add_rel_dyn(self.m_accum_offset, reloc_type, label.clone());
                                if reloc_type == RelocationType::RSbfSyscall {
                                    self.m_dynamic_symbols.add_call_target(label.clone(), self.m_accum_offset);
                                }
                            }
                            let offset = self.m_accum_offset;
                            self.m_accum_offset += inst.get_size();
                            nodes.push(ASTNode::Instruction { instruction: inst, offset });
                            tokens = rest;
                        }
                        Err(e) => {
                            errors.push(e);
                            tokens = &tokens[1..];
                        }
                    }
                }
                _ => {
                    tokens = &tokens[1..];
                }
            }
        }

        if !errors.is_empty() {
            return Err(errors);
        }

        // Second pass to resolve labels
        for node in &mut nodes {
            match node {
                ASTNode::Instruction { instruction: Instruction { opcode, operands, .. }, offset, .. } => {
                    // For jump instructions, replace label operands with relative offsets
                    if *opcode == Opcode::Ja || *opcode == Opcode::JeqImm || *opcode == Opcode::JgtImm || *opcode == Opcode::JgeImm 
                    || *opcode == Opcode::JltImm || *opcode == Opcode::JleImm || *opcode == Opcode::JsetImm || *opcode == Opcode::JneImm     
                    || *opcode == Opcode::JsgtImm || *opcode == Opcode::JsgeImm || *opcode == Opcode::JsltImm || *opcode == Opcode::JsleImm
                    || *opcode == Opcode::JeqReg || *opcode == Opcode::JgtReg || *opcode == Opcode::JgeReg || *opcode == Opcode::JltReg 
                    || *opcode == Opcode::JleReg || *opcode == Opcode::JsetReg || *opcode == Opcode::JneReg || *opcode == Opcode::JsgtReg 
                    || *opcode == Opcode::JsgeReg || *opcode == Opcode::JsltReg || *opcode == Opcode::JsleReg {
                        if let Some(Token::Identifier(label, span)) = operands.last() {
                            let label = label.clone(); // Clone early to avoid borrow conflict
                            if let Some(target_offset) = self.m_label_offsets.get(&label) {
                                let rel_offset = (*target_offset as i64 - *offset as i64) / 8 - 1;
                                // Replace label with immediate value
                                let last_idx = operands.len() - 1;
                                operands[last_idx] = Token::ImmediateValue(ImmediateValue::Int(rel_offset), span.clone());
                            }
                        }
                    }
                    if *opcode == Opcode::Lddw {
                        if let Some(Token::Identifier(name, span)) = operands.last() {
                            let label = name.clone();
                            if let Some(target_offset) = self.m_label_offsets.get(&label) {
                                let ph_count = if self.m_prog_is_static { 1 } else { 3 };
                                let ph_offset = 64 + (ph_count as u64 * 56) as i64;
                                let abs_offset = *target_offset as i64 + ph_offset;
                                // Replace label with immediate value
                                let last_idx = operands.len() - 1;
                                operands[last_idx] = Token::ImmediateValue(ImmediateValue::Addr(abs_offset), span.clone());
                            }  else {
                                errors.push(CompileError::UndefinedLabel { label: name.clone(), span: span.clone(), custom_label: None });
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        // Set entry point offset if an entry label was specified
        if let Some(entry_label) = &self.m_entry_label {
            if let Some(offset) = self.m_label_offsets.get(entry_label) {
                self.m_dynamic_symbols.add_entry_point(entry_label.clone(), *offset);
            }
        }

        if !errors.is_empty() {
            return Err(errors);
        } else {
            Ok(ParseResult {
                code_section: CodeSection::new(nodes, self.m_accum_offset, self.m_file.as_ref().unwrap()),
                data_section: DataSection::new(rodata_nodes, self.m_rodata_size),
                dynamic_symbols: DynamicSymbolMap::copy(&self.m_dynamic_symbols),
                relocation_data: RelDynMap::copy(&self.m_rel_dyns),
                prog_is_static: self.m_prog_is_static,
            })
        }
    }
}
