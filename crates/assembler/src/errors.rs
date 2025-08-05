use crate::define_compile_errors;
use std::ops::Range;

define_compile_errors! {
    InvalidNumber {
        error = "Invalid number '{number}'",
        label = "Invalid number",
        fields = { number: String, span: Range<usize> }
    },
    InvalidRegister {
        error = "Invalid register '{register}'",
        label = "Invalid register",
        fields = { register: String, span: Range<usize> }
    },
    UnexpectedCharacter {
        error = "Unexpected character '{character}'",
        label = "Unexpected character",
        fields = { character: char, span: Range<usize> }
    },
    UnterminatedStringLiteral {
        error = "Unterminated string literal",
        label = "Unterminated string literal",
        fields = { span: Range<usize> }
    }
}

use codespan_reporting::diagnostic::{Diagnostic, Label};

pub trait AsDiagnostic {
    fn as_diagnostic(&self) -> Diagnostic<usize>;
}

impl AsDiagnostic for CompileError {
    fn as_diagnostic(&self) -> Diagnostic<usize> {
        Diagnostic::error()
            .with_message(self.to_string())
            .with_labels(vec![Label::primary(0, self.span().start..self.span().end).with_message(self.label())])
    }
}

