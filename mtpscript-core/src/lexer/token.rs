#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Literals
    String(String),
    Number(i64),
    Decimal(String),
    Boolean(bool),

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
    Respond,
    Import,

    // HTTP Methods
    Get,
    Post,
    Put,
    Delete,
    Patch,

    // Identifiers
    Ident(String),

    // Delimiters
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Comma,
    Colon,
    Semicolon,

    // Operators
    Plus,
    Minus,
    Star,
    Slash,
    EqualEqual,
    BangEqual,
    Less,
    Greater,
    LessEqual,
    GreaterEqual,
    AndAnd,
    OrOr,
    Bang,
    Dot,
    PipeGreater, // |>

    // Other
    Arrow, // =>
    Equal, // =
    Pipe, // |
    Underscore, // _

    // End of file
    Eof,
}

#[derive(Debug)]
pub struct TokenInfo {
    pub token: Token,
    pub line: usize,
    pub column: usize,
}
