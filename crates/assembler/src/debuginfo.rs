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


