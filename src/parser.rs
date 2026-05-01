use crate::cst::*;
use crate::token::{Keyword, Span, Token, TokenKind};

#[derive(Debug)]
pub struct ParseError {
    pub message: String,
    pub span: Span,
}

pub fn parse(tokens: &[Token]) -> Result<CstFile, ParseError> {
    let mut p = Parser { tokens, i: 0 };
    p.parse_file()
}

struct Parser<'a> {
    tokens: &'a [Token],
    i: usize,
}

impl<'a> Parser<'a> {
    fn parse_file(&mut self) -> Result<CstFile, ParseError> {
        let mut items = Vec::new();
        while !self.at_eof() {
            items.push(self.parse_item()?);
        }
        Ok(CstFile { items })
    }

    fn parse_item(&mut self) -> Result<CstNode, ParseError> {
        match self.peek_kind() {
            Some(TokenKind::Keyword(Keyword::State)) => self.parse_state(),
            Some(TokenKind::Keyword(Keyword::Fn)) => self.parse_fn(),
            Some(TokenKind::Keyword(Keyword::Process)) => self.parse_process(),
            Some(TokenKind::Keyword(Keyword::At)) => self.parse_at(),
            Some(TokenKind::Keyword(Keyword::Dock)) => self.parse_dock(),
            Some(TokenKind::Keyword(Keyword::Rewrite)) => self.parse_rewrite(),
            Some(TokenKind::Keyword(Keyword::Commit)) => self.parse_commit(),
            Some(TokenKind::Keyword(Keyword::Send)) => {
                self.parse_command_node(|c| CstNode::Send(c))
            }
            Some(TokenKind::Keyword(Keyword::Receive)) => {
                self.parse_command_node(|c| CstNode::Receive(c))
            }
            Some(TokenKind::Keyword(Keyword::Yield)) => {
                self.parse_command_node(|c| CstNode::Yield(c))
            }
            Some(TokenKind::Keyword(Keyword::Observe)) => {
                self.parse_command_node(|c| CstNode::Observe(c))
            }
            Some(TokenKind::Keyword(Keyword::Morph)) => {
                self.parse_command_node(|c| CstNode::Morph(c))
            }
            Some(TokenKind::Keyword(Keyword::Spawn)) => {
                self.parse_command_node(|c| CstNode::Spawn(c))
            }
            Some(TokenKind::Keyword(Keyword::Grant)) => {
                self.parse_command_node(|c| CstNode::Grant(c))
            }
            Some(TokenKind::Keyword(Keyword::Revoke)) => {
                self.parse_command_node(|c| CstNode::Revoke(c))
            }
            _ => self.parse_expr_stmt(),
        }
    }

    fn parse_state(&mut self) -> Result<CstNode, ParseError> {
        let start = self.current_span();
        let tokens = self.take_stmt_with_lead();
        let end = tokens.last().map(|t| t.span).unwrap_or(start);
        Ok(CstNode::State(StateDecl {
            span: join(start, end),
            tokens,
        }))
    }

    fn parse_fn(&mut self) -> Result<CstNode, ParseError> {
        let start = self.current_span();
        let header = self.take_until_block_start()?;
        let body = self.parse_block()?;
        Ok(CstNode::Fn(FnDecl {
            span: join(start, body.span),
            header,
            body,
        }))
    }

    fn parse_process(&mut self) -> Result<CstNode, ParseError> {
        let start = self.current_span();
        let header = self.take_until_block_start()?;
        let body = self.parse_block()?;
        Ok(CstNode::Process(ProcessDecl {
            span: join(start, body.span),
            header,
            body,
        }))
    }

    fn parse_at(&mut self) -> Result<CstNode, ParseError> {
        let start = self.current_span();
        let header = self.take_until_block_start()?;
        let body = self.parse_block()?;
        Ok(CstNode::At(AtBlock {
            span: join(start, body.span),
            header,
            body,
        }))
    }

    fn parse_dock(&mut self) -> Result<CstNode, ParseError> {
        let start = self.current_span();
        let header = self.take_until_block_start()?;
        let body = self.parse_block()?;
        Ok(CstNode::Dock(DockBlock {
            span: join(start, body.span),
            header,
            body,
        }))
    }

    fn parse_rewrite(&mut self) -> Result<CstNode, ParseError> {
        let start = self.current_span();
        let tokens = self.take_stmt_with_lead();
        let end = tokens.last().map(|t| t.span).unwrap_or(start);
        Ok(CstNode::Rewrite(RewriteStmt {
            span: join(start, end),
            tokens,
        }))
    }

    fn parse_commit(&mut self) -> Result<CstNode, ParseError> {
        let start = self.current_span();
        let tokens = self.take_stmt_with_lead();
        let end = tokens.last().map(|t| t.span).unwrap_or(start);
        Ok(CstNode::Commit(CommitStmt {
            span: join(start, end),
            tokens,
        }))
    }

    fn parse_command_node<F>(&mut self, wrap: F) -> Result<CstNode, ParseError>
    where
        F: FnOnce(CommandStmt) -> CstNode,
    {
        let start = self.current_span();
        let tokens = self.take_stmt_with_lead();
        let end = tokens.last().map(|t| t.span).unwrap_or(start);
        Ok(wrap(CommandStmt {
            span: join(start, end),
            tokens,
        }))
    }

    fn parse_expr_stmt(&mut self) -> Result<CstNode, ParseError> {
        let start = self.current_span();
        let tokens = self.take_stmt_with_lead();
        let end = tokens.last().map(|t| t.span).unwrap_or(start);
        Ok(CstNode::Expr(ExprStmt {
            span: join(start, end),
            tokens,
        }))
    }

    fn parse_block(&mut self) -> Result<Block, ParseError> {
        let lbrace = self.expect(TokenKind::LBrace, "expected '{' to start block")?;
        let mut items = Vec::new();
        while !self.at(TokenKind::RBrace) {
            if self.at_eof() {
                return Err(ParseError {
                    message: "unterminated block".into(),
                    span: lbrace.span,
                });
            }
            items.push(self.parse_item()?);
        }
        let rbrace = self.bump().unwrap();
        Ok(Block {
            span: join(lbrace.span, rbrace.span),
            items,
        })
    }

    fn take_until_block_start(&mut self) -> Result<Vec<Token>, ParseError> {
        let mut out = Vec::new();
        while !self.at(TokenKind::LBrace) {
            if self.at_eof() {
                return Err(ParseError {
                    message: "expected block".into(),
                    span: self.current_span(),
                });
            }
            out.push(self.bump().unwrap());
        }
        Ok(out)
    }

    fn take_stmt_with_lead(&mut self) -> Vec<Token> {
        let mut out = Vec::new();
        if !self.at_eof() {
            out.push(self.bump().unwrap());
        }
        let start_line = out
            .first()
            .map(|t| t.span.line)
            .unwrap_or(self.current_span().line);
        out.extend(self.take_until_stmt_end(start_line));
        out
    }
    fn take_until_stmt_end(&mut self, start_line: usize) -> Vec<Token> {
        let mut out = Vec::new();
        let mut brace_depth = 0usize;
        while !self.at_eof() {
            if self.at(TokenKind::Semicolon) && brace_depth == 0 {
                out.push(self.bump().unwrap());
                break;
            }

            if brace_depth == 0 {
                if self.looks_like_statement_boundary() {
                    break;
                }
                if let Some(tok) = self.tokens.get(self.i) {
                    if tok.span.line > start_line && self.looks_like_line_start_statement() {
                        break;
                    }
                }
            }

            let t = self.bump().unwrap();
            match t.kind {
                TokenKind::LBrace => brace_depth += 1,
                TokenKind::RBrace => {
                    if brace_depth == 0 {
                        break;
                    }
                    brace_depth -= 1;
                }
                _ => {}
            }
            out.push(t);
        }
        out
    }

    fn looks_like_line_start_statement(&self) -> bool {
        matches!(
            self.peek_kind(),
            Some(TokenKind::Identifier(_))
                | Some(TokenKind::Keyword(Keyword::If))
                | Some(TokenKind::Keyword(Keyword::While))
                | Some(TokenKind::Keyword(Keyword::For))
                | Some(TokenKind::Keyword(Keyword::Loop))
                | Some(TokenKind::Keyword(Keyword::State))
                | Some(TokenKind::Keyword(Keyword::Rewrite))
                | Some(TokenKind::Keyword(Keyword::Send))
                | Some(TokenKind::Keyword(Keyword::Receive))
                | Some(TokenKind::Keyword(Keyword::Yield))
                | Some(TokenKind::Keyword(Keyword::Observe))
                | Some(TokenKind::Keyword(Keyword::Morph))
                | Some(TokenKind::Keyword(Keyword::Spawn))
                | Some(TokenKind::Keyword(Keyword::Grant))
                | Some(TokenKind::Keyword(Keyword::Revoke))
        )
    }
    fn looks_like_statement_boundary(&self) -> bool {
        matches!(
            self.peek_kind(),
            Some(TokenKind::Keyword(Keyword::State))
                | Some(TokenKind::Keyword(Keyword::Fn))
                | Some(TokenKind::Keyword(Keyword::Process))
                | Some(TokenKind::Keyword(Keyword::At))
                | Some(TokenKind::Keyword(Keyword::Dock))
                | Some(TokenKind::Keyword(Keyword::Rewrite))
                | Some(TokenKind::Keyword(Keyword::Commit))
                | Some(TokenKind::Keyword(Keyword::Send))
                | Some(TokenKind::Keyword(Keyword::Receive))
                | Some(TokenKind::Keyword(Keyword::Yield))
                | Some(TokenKind::Keyword(Keyword::Observe))
                | Some(TokenKind::Keyword(Keyword::Morph))
                | Some(TokenKind::Keyword(Keyword::Spawn))
                | Some(TokenKind::Keyword(Keyword::Grant))
                | Some(TokenKind::Keyword(Keyword::Revoke))
                | Some(TokenKind::RBrace)
                | Some(TokenKind::Eof)
        )
    }

    fn at(&self, kind: TokenKind) -> bool {
        self.peek_kind() == Some(&kind)
    }

    fn expect(&mut self, kind: TokenKind, msg: &str) -> Result<Token, ParseError> {
        if self.at(kind.clone()) {
            Ok(self.bump().unwrap())
        } else {
            Err(ParseError {
                message: msg.into(),
                span: self.current_span(),
            })
        }
    }

    fn bump(&mut self) -> Option<Token> {
        let t = self.tokens.get(self.i).cloned();
        if t.is_some() {
            self.i += 1;
        }
        t
    }

    fn peek_kind(&self) -> Option<&TokenKind> {
        self.tokens.get(self.i).map(|t| &t.kind)
    }

    fn current_span(&self) -> Span {
        self.tokens.get(self.i).map(|t| t.span).unwrap_or(Span {
            start: 0,
            end: 0,
            line: 1,
            column: 1,
        })
    }

    fn at_eof(&self) -> bool {
        matches!(self.peek_kind(), Some(TokenKind::Eof) | None)
    }
}

fn join(a: Span, b: Span) -> Span {
    Span {
        start: a.start,
        end: b.end,
        line: a.line,
        column: a.column,
    }
}
