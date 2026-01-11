#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Keywords
    Function,
    Type,
    Api,
    Const,
    If,
    Else,
    Match,
    Await,
    Uses,
    Import,
    Respond,
    True,
    False,

    // Operators
    Plus,
    Minus,
    Star,
    Slash,
    EqualEqual,
    BangEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    Bang,
    AmpAmp,
    PipePipe,
    PipeGreater, // |>
    Dot,

    // Delimiters
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Comma,
    Colon,
    Semicolon,
    EqualGreater, // =>

    // HTTP Methods
    Get,
    Post,
    Put,
    Delete,
    Patch,

    // Literals
    String(String),
    Number(String), // Store as string to preserve precision
    Identifier(String),

    // Special
    Eof,
}
