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

#[derive(Clone, Debug)]
pub struct Token<'a> {
  token_type: TokenType,
  lexeme: &'a str,
  literal: LiteralConstant<'a>,
  line: usize,
}

impl<'a> Token<'a> {
  pub fn new(token_type: TokenType, lexeme: &'a str, literal: LiteralConstant<'a>, line: usize) -> Token<'a> {
    Token {
      token_type,
      lexeme,
      literal,
      line,
    }
  }

  pub fn get_type(&self) -> &TokenType {
    &self.token_type
  }

  pub fn get_lexeme(&self) -> &str {
    self.lexeme
  }

  pub fn get_literal(&self) -> LiteralConstant<'a> {
    self.literal
  }

  pub fn get_line(&self) -> usize {
    self.line
  }
}

