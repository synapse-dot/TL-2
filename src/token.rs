#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Keyword {
    State,
    At,
    Until,
    From,
    To,
    Now,
    Fn,
    Rewrite,
    Morph,
    Dock,
    Commit,
    Yield,
    Spawn,
    Send,
    Receive,
    SelfKw,
    Grant,
    Revoke,
    If,
    Else,
    Loop,
    While,
    For,
    In,
    And,
    Or,
    Not,
    True,
    False,
    Null,
    Observe,
    Old,
    Pre,
    Post,
    Process,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeUnit {
    Ms,
    S,
    Min,
    H,
    D,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    Keyword(Keyword),
    Identifier(String),
    Number(String),
    TimeNumber { value: String, unit: TimeUnit },
    StringLiteral(String),

    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Comma,
    Dot,
    Colon,
    Semicolon,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Eq,
    EqEq,
    BangEq,
    Gt,
    Lt,
    GtEq,
    LtEq,
    Arrow,    // ->
    FatArrow, // =>

    Eof,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}
