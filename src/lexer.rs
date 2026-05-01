use crate::token::{Keyword, Span, TimeUnit, Token, TokenKind};

#[derive(Debug)]
pub struct LexError {
    pub message: String,
    pub span: Span,
}

pub fn lex(input: &str) -> Result<Vec<Token>, LexError> {
    let mut lexer = Lexer::new(input);
    lexer.lex_all()
}

struct Lexer<'a> {
    src: &'a [u8],
    pos: usize,
    line: usize,
    col: usize,
}

impl<'a> Lexer<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            src: input.as_bytes(),
            pos: 0,
            line: 1,
            col: 1,
        }
    }

    fn lex_all(&mut self) -> Result<Vec<Token>, LexError> {
        let mut tokens = Vec::new();

        while !self.is_eof() {
            self.skip_whitespace_and_comments()?;
            if self.is_eof() {
                break;
            }

            let start = self.mark();
            let ch = self.peek_char().unwrap();
            let token = match ch {
                b'(' => self.simple(TokenKind::LParen, start),
                b')' => self.simple(TokenKind::RParen, start),
                b'{' => self.simple(TokenKind::LBrace, start),
                b'}' => self.simple(TokenKind::RBrace, start),
                b'[' => self.simple(TokenKind::LBracket, start),
                b']' => self.simple(TokenKind::RBracket, start),
                b',' => self.simple(TokenKind::Comma, start),
                b'.' => self.simple(TokenKind::Dot, start),
                b':' => self.simple(TokenKind::Colon, start),
                b';' => self.simple(TokenKind::Semicolon, start),
                b'+' => self.simple(TokenKind::Plus, start),
                b'*' => self.simple(TokenKind::Star, start),
                b'%' => self.simple(TokenKind::Percent, start),
                b'/' => self.simple(TokenKind::Slash, start),
                b'-' => {
                    self.bump();
                    if self.match_char(b'>') {
                        self.finish(start, TokenKind::Arrow)
                    } else {
                        self.finish(start, TokenKind::Minus)
                    }
                }
                b'=' => {
                    self.bump();
                    if self.match_char(b'>') {
                        self.finish(start, TokenKind::FatArrow)
                    } else if self.match_char(b'=') {
                        self.finish(start, TokenKind::EqEq)
                    } else {
                        self.finish(start, TokenKind::Eq)
                    }
                }
                b'!' => {
                    self.bump();
                    if self.match_char(b'=') {
                        self.finish(start, TokenKind::BangEq)
                    } else {
                        return Err(self.err("expected '=' after '!'", start));
                    }
                }
                b'>' => {
                    self.bump();
                    if self.match_char(b'=') {
                        self.finish(start, TokenKind::GtEq)
                    } else {
                        self.finish(start, TokenKind::Gt)
                    }
                }
                b'<' => {
                    self.bump();
                    if self.match_char(b'=') {
                        self.finish(start, TokenKind::LtEq)
                    } else {
                        self.finish(start, TokenKind::Lt)
                    }
                }
                b'"' => self.lex_string(start)?,
                c if is_ident_start(c) => self.lex_ident_or_keyword(start),
                c if c.is_ascii_digit() => self.lex_number_or_time(start),
                _ => return Err(self.err("unexpected character", start)),
            };

            tokens.push(token);
        }

        let eof = self.mark();
        tokens.push(Token {
            kind: TokenKind::Eof,
            span: Span {
                start: eof.start,
                end: eof.end,
                line: eof.line,
                column: eof.column,
            },
        });

        Ok(tokens)
    }

    fn skip_whitespace_and_comments(&mut self) -> Result<(), LexError> {
        loop {
            while let Some(c) = self.peek_char() {
                if c.is_ascii_whitespace() {
                    self.bump();
                } else {
                    break;
                }
            }

            if self.peek_char() == Some(b'/') && self.peek_next() == Some(b'/') {
                while let Some(c) = self.peek_char() {
                    self.bump();
                    if c == b'\n' {
                        break;
                    }
                }
                continue;
            }

            if self.peek_char() == Some(b'/') && self.peek_next() == Some(b'*') {
                let start = self.mark();
                self.bump();
                self.bump();
                loop {
                    match self.peek_char() {
                        Some(b'*') if self.peek_next() == Some(b'/') => {
                            self.bump();
                            self.bump();
                            break;
                        }
                        Some(_) => {
                            self.bump();
                        }
                        None => return Err(self.err("unterminated block comment", start)),
                    }
                }
                continue;
            }

            break;
        }
        Ok(())
    }

    fn lex_ident_or_keyword(&mut self, start: Span) -> Token {
        let mut ident = String::new();
        while let Some(c) = self.peek_char() {
            if is_ident_continue(c) {
                ident.push(c as char);
                self.bump();
            } else {
                break;
            }
        }

        let kind = match_keyword(&ident)
            .map(TokenKind::Keyword)
            .unwrap_or(TokenKind::Identifier(ident));

        self.finish(start, kind)
    }

    fn lex_number_or_time(&mut self, start: Span) -> Token {
        let mut number = String::new();
        while let Some(c) = self.peek_char() {
            if c.is_ascii_digit() || c == b'.' {
                number.push(c as char);
                self.bump();
            } else {
                break;
            }
        }

        let snapshot = (self.pos, self.line, self.col);
        if let Some(unit) = self.match_time_unit() {
            return self.finish(
                start,
                TokenKind::TimeNumber {
                    value: number,
                    unit,
                },
            );
        }
        (self.pos, self.line, self.col) = snapshot;

        self.finish(start, TokenKind::Number(number))
    }

    fn lex_string(&mut self, start: Span) -> Result<Token, LexError> {
        self.bump();
        let mut s = String::new();
        while let Some(c) = self.peek_char() {
            if c == b'"' {
                self.bump();
                return Ok(self.finish(start, TokenKind::StringLiteral(s)));
            }
            if c == b'\\' {
                self.bump();
                match self.peek_char() {
                    Some(b'n') => {
                        s.push('\n');
                        self.bump();
                    }
                    Some(b't') => {
                        s.push('\t');
                        self.bump();
                    }
                    Some(b'"') => {
                        s.push('"');
                        self.bump();
                    }
                    Some(b'\\') => {
                        s.push('\\');
                        self.bump();
                    }
                    Some(_) => return Err(self.err("invalid escape sequence", start)),
                    None => return Err(self.err("unterminated string", start)),
                }
            } else {
                s.push(c as char);
                self.bump();
            }
        }
        Err(self.err("unterminated string", start))
    }

    fn match_time_unit(&mut self) -> Option<TimeUnit> {
        if self.consume_bytes(b"ms") {
            return Some(TimeUnit::Ms);
        }
        if self.consume_bytes(b"min") {
            return Some(TimeUnit::Min);
        }
        if self.consume_bytes(b"s") {
            return Some(TimeUnit::S);
        }
        if self.consume_bytes(b"h") {
            return Some(TimeUnit::H);
        }
        if self.consume_bytes(b"d") {
            return Some(TimeUnit::D);
        }
        None
    }

    fn simple(&mut self, kind: TokenKind, start: Span) -> Token {
        self.bump();
        self.finish(start, kind)
    }

    fn finish(&self, start: Span, kind: TokenKind) -> Token {
        Token {
            kind,
            span: Span {
                start: start.start,
                end: self.pos,
                line: start.line,
                column: start.column,
            },
        }
    }

    fn err(&self, message: &str, start: Span) -> LexError {
        LexError {
            message: message.to_string(),
            span: Span {
                start: start.start,
                end: self.pos,
                line: start.line,
                column: start.column,
            },
        }
    }

    fn consume_bytes(&mut self, bytes: &[u8]) -> bool {
        if self.src.get(self.pos..self.pos + bytes.len()) == Some(bytes) {
            for _ in 0..bytes.len() {
                self.bump();
            }
            true
        } else {
            false
        }
    }

    fn match_char(&mut self, expected: u8) -> bool {
        if self.peek_char() == Some(expected) {
            self.bump();
            true
        } else {
            false
        }
    }

    fn mark(&self) -> Span {
        Span {
            start: self.pos,
            end: self.pos,
            line: self.line,
            column: self.col,
        }
    }

    fn bump(&mut self) {
        if let Some(c) = self.peek_char() {
            self.pos += 1;
            if c == b'\n' {
                self.line += 1;
                self.col = 1;
            } else {
                self.col += 1;
            }
        }
    }

    fn peek_char(&self) -> Option<u8> {
        self.src.get(self.pos).copied()
    }

    fn peek_next(&self) -> Option<u8> {
        self.src.get(self.pos + 1).copied()
    }

    fn is_eof(&self) -> bool {
        self.pos >= self.src.len()
    }
}

fn is_ident_start(c: u8) -> bool {
    c.is_ascii_alphabetic() || c == b'_'
}

fn is_ident_continue(c: u8) -> bool {
    is_ident_start(c) || c.is_ascii_digit()
}

fn match_keyword(s: &str) -> Option<Keyword> {
    Some(match s {
        "state" => Keyword::State,
        "at" => Keyword::At,
        "until" => Keyword::Until,
        "from" => Keyword::From,
        "to" => Keyword::To,
        "now" => Keyword::Now,
        "fn" => Keyword::Fn,
        "rewrite" => Keyword::Rewrite,
        "morph" => Keyword::Morph,
        "dock" => Keyword::Dock,
        "commit" => Keyword::Commit,
        "yield" => Keyword::Yield,
        "spawn" => Keyword::Spawn,
        "send" => Keyword::Send,
        "receive" => Keyword::Receive,
        "self" => Keyword::SelfKw,
        "grant" => Keyword::Grant,
        "revoke" => Keyword::Revoke,
        "if" => Keyword::If,
        "else" => Keyword::Else,
        "loop" => Keyword::Loop,
        "while" => Keyword::While,
        "for" => Keyword::For,
        "in" => Keyword::In,
        "and" => Keyword::And,
        "or" => Keyword::Or,
        "not" => Keyword::Not,
        "true" => Keyword::True,
        "false" => Keyword::False,
        "null" => Keyword::Null,
        "observe" => Keyword::Observe,
        "old" => Keyword::Old,
        "pre" => Keyword::Pre,
        "post" => Keyword::Post,
        "process" => Keyword::Process,
        _ => return None,
    })
}
