use super::token::{Token, TokenType};

pub struct Scanner<'a> {
  source: &'a str,
  start: usize,
  current: usize,
  line: usize,
}

#[derive(Debug)]
pub struct SourceContext {
  pub lexeme: String,
  pub line: usize,
}

impl SourceContext {
  fn new(lexeme: String, line: usize) -> Self {
    SourceContext {
      lexeme,
      line,
    }
  }
}

#[derive(Debug)]
pub enum ScannerError<> {
  UnexpectedEof(SourceContext),
  UnsupportedChar(SourceContext, u8),
  InvalidNumber(SourceContext),
}

fn is_digit(byte: u8) -> bool {
  byte >= b'0' && byte <= b'9'
}

impl<'a> Scanner<'a> {
  pub fn new(source: &'a str) -> Scanner<'a> {
    Scanner {
      source,
      start: 0,
      current: 0,
      line: 1,
    }
  }

  pub fn scan_token(&mut self) -> Result<Token<'a>, ScannerError> {
    self.skip_whitespace_and_comments();

    self.start = self.current;

    use TokenType::*;

    // FIXME: To properly support utf-8 we'd need to support extended grapheme
    // clusters.
    if let Some(byte) = self.advance() {
      return match byte {
        b'(' => Ok(self.make_token(LeftParen)),
        b')' => Ok(self.make_token(RightParen)),
        b'{' => Ok(self.make_token(LeftBrace)),
        b'}' => Ok(self.make_token(RightBrace)),
        b';' => Ok(self.make_token(Semicolon)),
        b',' => Ok(self.make_token(Comma)),
        b'.' => Ok(self.make_token(Dot)),
        b'-' => Ok(self.make_token(Minus)),
        b'+' => Ok(self.make_token(Plus)),
        b'/' => Ok(self.make_token(Slash)),
        b'*' => Ok(self.make_token(Star)),
        b'!' => Ok(self.make_match_token(b'=', BangEqual, Bang)),
        b'=' => Ok(self.make_match_token(b'=', EqualEqual, Equal)),
        b'<' => Ok(self.make_match_token(b'=', LessEqual, Less)),
        b'>' => Ok(self.make_match_token(b'=', GreaterEqual, Greater)),
        b'"' => self.make_string(),
        b'0'..=b'9' => self.make_number(),
        b'a'..=b'z' | b'A'..=b'Z' => self.make_identifier(),
        _ => {
          let ctx = self.current_source_context();
          Err(ScannerError::UnsupportedChar(ctx, byte))
        },
      }
    } else {
      return Ok(self.make_token(TokenType::Eof));
    }
  }

  fn make_match_token(&mut self, byte: u8, match_type: TokenType, no_match_type: TokenType) -> Token<'a> {
    if self.match_byte(byte) {
      self.make_token(match_type)
    } else {
      self.make_token(no_match_type)
    }
  }

  /// Returns the current lexeme.
  fn current_lexeme(&self) -> &'a str {
    &self.source[self.start..self.current]
  }

  fn current_source_context(&self) -> SourceContext {
    SourceContext::new(self.current_lexeme().to_string(), self.line)
  }

  fn make_token(&mut self, token_type: TokenType) -> Token<'a> {
    Token::new(token_type, self.current_lexeme(), self.line)
  }

  fn make_string(&mut self) -> Result<Token<'a>, ScannerError> {
    // Read until Eof or ".
    loop {
      match self.current_byte() {
        Some(b'"') => { break; },
        None => { break; }
        Some(b'\n') => {
          self.line += 1;
          self.advance();
        }
        Some(_) => { self.advance(); },
      }
    }

    if self.at_end() {
      Err(ScannerError::UnexpectedEof(self.current_source_context()))
    } else {
      self.advance();  // The closing quote.
      let parsed_str = self.source[self.start + 1..self.current - 1].to_string();
      Ok(self.make_token(TokenType::String(parsed_str)))
    }
  }

  fn make_number(&mut self) -> Result<Token<'a>, ScannerError> {
    while is_digit(self.current_byte().unwrap_or(0)) {
      self.advance();
    }

    // Look for a fractional part.
    if self.current_byte() == Some(b'.') && is_digit(self.peek_next_byte().unwrap_or(0)) {
      // Consume the ".".
      self.advance();

      while is_digit(self.current_byte().unwrap_or(0)) {
        self.advance();
      }
    }

    let number_str = self.current_lexeme();
    if let Ok(number) = number_str.parse::<f64>() {
      return Ok(self.make_token(TokenType::Number(number)));
    }
    Err(ScannerError::InvalidNumber(self.current_source_context()))
  }

  fn make_identifier(&mut self) -> Result<Token<'a>, ScannerError> {
    loop {
      match self.current_byte().unwrap_or(0) {
        b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' => { self.advance(); }
        _ => { break; }
      }
    }

    Ok(self.make_token(self.identifier_type()))
  }

  fn identifier_type(&self) -> TokenType {
    let identifier_str = self.current_lexeme();
    match identifier_str {
      "and" => TokenType::And,
      "class" => TokenType::Class,
      "else" => TokenType::Else,
      "false" => TokenType::False,
      "for" => TokenType::For,
      "fun" => TokenType::Fun,
      "if" => TokenType::If,
      "nil" => TokenType::Nil,
      "or" => TokenType::Or,
      "print" => TokenType::Print,
      "return" => TokenType::Return,
      "super" => TokenType::Super,
      "this" => TokenType::This,
      "true" => TokenType::True,
      "var" => TokenType::Var,
      "while" => TokenType::While,
      _ => {
        TokenType::Identifier(identifier_str.to_string())
      }
    }
  }

  fn advance(&mut self) -> Option<u8> {
    let current_byte = self.current_byte()?;
    self.current += 1;
    Some(current_byte)
  }

  fn match_byte(&mut self, byte: u8) -> bool {
    if let Some(current_byte) = self.current_byte() {
      if byte == current_byte {
        self.current += 1;
        return true;
      }
    }
    return false;
  }

  /// Returns the current character byte, potentially partial utf-8.
  fn current_byte(&self) -> Option<u8> {
    if self.at_end() {
      return None;
    }
    Some(self.source.as_bytes()[self.current])
  }

  fn peek_next_byte(&mut self) -> Option<u8> {
    // Reuse `current_byte` because I'm lazy.
    let current = self.current;
    self.current += 1;

    let result = self.current_byte();

    self.current =  current;
    result
  }

  fn at_end(&self) -> bool {
    self.current >= self.source.len()
  }

  fn skip_whitespace_and_comments(&mut self) {
    loop {
      match self.current_byte().unwrap_or(0) {
        b' ' | b'\r' | b'\t' => { self.advance(); },
        b'\n' => {
          self.line += 1;
          self.advance();
        },
        b'/' => {
          let next_byte = self.peek_next_byte();
          if next_byte != Some(b'/') {
            return;
          }
          self.advance();
          // A comment goes until the end of the line or Eof.
          loop {
            match self.current_byte() {
              Some(b'\n') => { break; },
              Some(_) => { self.advance(); },
              None => { break; }
            }
          }
        },
        _ => {
          return;
        },
      }
    }
  }
}
