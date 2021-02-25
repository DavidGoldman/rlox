use std::fmt::Display;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum TokenType {
    Eof,

    // Single-character tokens.
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    // One or two character tokens.
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals, values stored in `LiteralConstant` or the lexeme.
    Identifier,
    String,
    Number,

    // Keywords.
    And,
    Class,
    Else,
    False,
    For,
    Fun,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,
}

#[derive(Copy, Clone, Debug)]
pub enum LiteralConstant<'a> {
    None,
    String(&'a str),
    Number(f64),
}

pub type Line = usize;

#[derive(Clone, Debug)]
pub struct Token<'a> {
    token_type: TokenType,
    lexeme: &'a str,
    literal: LiteralConstant<'a>,
    line: Line,
}

// For error messages.
pub struct TokenErrContext {
    pub token_type: TokenType,
    pub lexeme: String,
    pub line: Line,
}

impl Display for TokenErrContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.token_type {
            TokenType::Eof => write!(f, "[line {}] Error at end", self.line),
            _ => write!(f, "[line {}] Error at '{}'", self.line, &self.lexeme),
        }
    }
}

impl<'a> Token<'a> {
    pub fn new(
        token_type: TokenType,
        lexeme: &'a str,
        literal: LiteralConstant<'a>,
        line: usize,
    ) -> Token<'a> {
        Token {
            token_type,
            lexeme,
            literal,
            line,
        }
    }

    pub fn to_err_context(&self) -> TokenErrContext {
        TokenErrContext {
            token_type: self.token_type,
            lexeme: self.lexeme.to_string(),
            line: self.line,
        }
    }

    pub fn token_type(&self) -> &TokenType {
        &self.token_type
    }

    pub fn lexeme(&self) -> &str {
        self.lexeme
    }

    pub fn literal(&self) -> LiteralConstant<'a> {
        self.literal
    }

    pub fn line(&self) -> Line {
        self.line
    }
}
