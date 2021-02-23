#[derive(Clone, Debug, PartialEq)]
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

  // Literals.
  Identifier(String),
  String(String),
  Number(f64),

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

#[derive(Clone, Debug)]
pub struct Token<'a> {
  token_type: TokenType,
  lexeme: &'a str,
  line: usize,
}

impl<'a> Token<'a> {
  pub fn new(token_type: TokenType, lexeme: &'a str, line: usize) -> Token<'a> {
    Token {
      token_type,
      lexeme,
      line,
    }
  }

  pub fn get_type(&self) -> &TokenType {
    &self.token_type
  }

  pub fn get_line(&self) -> usize {
    self.line
  }

  pub fn copy_lexeme(&self) -> String {
    String::from(self.lexeme)
  }
}

