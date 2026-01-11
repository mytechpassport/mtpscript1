use super::MtpError;

#[derive(Debug)]
pub enum CompileError {
    LexerError(String),
    ParserError(String),
    TypeError(String),
    EffectNotDeclared { effect: String },
    AwaitWithoutAsync,
    RespondOutsideApi,
    CodeGenError(String),
}

impl From<CompileError> for MtpError {
    fn from(err: CompileError) -> Self {
        match err {
            CompileError::LexerError(msg) => MtpError::Build(format!("Lexer error: {}", msg)),
            CompileError::ParserError(msg) => MtpError::Build(format!("Parser error: {}", msg)),
            CompileError::TypeError(msg) => MtpError::Build(format!("Type error: {}", msg)),
            CompileError::EffectNotDeclared { effect } => {
                MtpError::Build(format!("Effect '{}' not declared", effect))
            }
            CompileError::AwaitWithoutAsync => {
                MtpError::Build("await used without Async effect declared".to_string())
            }
            CompileError::RespondOutsideApi => {
                MtpError::Build("respond json used outside API declaration".to_string())
            }
            CompileError::CodeGenError(msg) => {
                MtpError::Build(format!("Code generation error: {}", msg))
            }
        }
    }
}
