use std::ops::Range;
use codespan_reporting::files::{SimpleFile, Files};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegisterType {
    Int,
    Addr,
    Null,
}

impl RegisterType {
    pub fn to_string(&self) -> &'static str {
        match self {
            RegisterType::Int => "int",
            RegisterType::Addr => "addr",
            RegisterType::Null => "null",
        }
    }
}

#[derive(Debug, Clone)]
pub struct RegisterHint {
    pub register: usize,
    pub register_type: RegisterType,
}

impl Default for RegisterHint {
    fn default() -> Self {
        Self {
            register: 0,
            register_type: RegisterType::Null,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DebugInfo {
    pub line_number: usize,
    pub register_hint: RegisterHint,
}

impl DebugInfo {
    pub fn new(line_number: usize) -> Self {
        Self { line_number, register_hint: RegisterHint::default() }
    }
}

pub fn span_to_line_number(span: Range<usize>, file: &SimpleFile<String, String>) -> usize {
    let start_line = file.line_index((), span.start).ok();
    start_line.unwrap_or(0) + 1
}
