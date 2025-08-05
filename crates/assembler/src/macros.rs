#[macro_export]
macro_rules! define_compile_errors {
    (
        $(
            $variant:ident {
                error = $error_msg:literal,
                label = $label_msg:literal,
                fields = { $( $field_name:ident : $field_ty:ty ),* $(,)? }
            }
        ),* $(,)?
    ) => {
        #[derive(Debug, thiserror::Error)]
        pub enum CompileError {
            $(
                #[error($error_msg)]
                $variant { $( $field_name: $field_ty ),* }
            ),*
        }

        impl CompileError {
            pub fn label(&self) -> &'static str {
                match self {
                    $(
                        Self::$variant { .. } => $label_msg,
                    )*
                }
            }

            pub fn span(&self) -> &Range<usize> {
                match self {
                    $(
                        Self::$variant { span, .. } => span,
                    )*
                }
            }
        }
    };
}

