use super::MtpError;

#[derive(Debug)]
pub enum CompileError {
    LexerError(String),
    ParserError(String),
    TypeError(String),
    EffectNotDeclared { effect: String },
    AwaitWithoutAsync,
    RespondOutsideApi,
}

impl From<CompileError> for MtpError {
    fn from(err: CompileError) -> Self {
        match err {
            CompileError::LexerError(msg) => MtpError::new("CompileLexer", &msg),
            CompileError::ParserError(msg) => MtpError::new("CompileParser", &msg),
            CompileError::TypeError(msg) => MtpError::new("CompileType", &msg),
            CompileError::EffectNotDeclared { effect } => {
                MtpError::new("CompileEffect", &format!("Effect '{}' not declared", effect))
            }
            CompileError::AwaitWithoutAsync => {
                MtpError::new("CompileEffect", "await used without Async effect declared")
            }
            CompileError::RespondOutsideApi => {
                MtpError::new("CompileEffect", "respond json used outside API declaration")
            }
        }
    }
}
