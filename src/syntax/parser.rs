use std::{convert::TryFrom, mem::replace};

use crate::vm::{bytecode::{ByteCode, Chunk, OpCode}, value::Value};

use super::{scanner::Scanner, token::{Token, TokenType}};

pub type Line = usize;

#[derive(Debug)]
pub enum ParserError {
  ExpectExpression(String, Line),
  TooManyConstants(Value, Line),
  TypeMismatch(TokenType, TokenType),  // expected, got
  UnexpectedToken(String, Line),
  InternalError(String, Line),
}

#[derive(PartialEq, PartialOrd, Eq, Ord, Copy, Clone)]
#[repr(u8)]
enum Precedence {
  None = 0,
  Assignment,  // =
  Or,          // or
  And,         // and
  Equality,    // == !=
  Comparison,  // < > <= >=
  Term,        // + -
  Factor,     // * /
  Unary,       // ! -
  Call,        // . ()
  Primary,
}

impl TryFrom<u8> for Precedence {
  type Error = ();

  fn try_from(p: u8) -> Result<Self, Self::Error> {
    use Precedence::*;
    match p {
      x if x == None as u8 => Ok(None),
      x if x == Assignment as u8 => Ok(Assignment),
      x if x == Or as u8 => Ok(Or),
      x if x == And as u8 => Ok(And),
      x if x == Equality as u8 => Ok(Equality),
      x if x == Comparison as u8 => Ok(Comparison),
      x if x == Term as u8 => Ok(Term),
      x if x == Factor as u8 => Ok(Factor),
      x if x == Unary as u8 => Ok(Unary),
      x if x == Call as u8 => Ok(Call),
      x if x == Primary as u8 => Ok(Primary),
      _ => Err(()),
    }
  }
}

impl Precedence {
  fn one_higher(&self) -> Precedence {
    Precedence::try_from(*self as u8 + 1).ok().unwrap_or(Precedence::Primary)
  }
}

pub struct Parser<'a> {
  scanner: Scanner<'a>,
  chunk: &'a mut Chunk,
  errors: Vec<ParserError>,
  panic_mode: bool,
  current: Token<'a>,
  previous: Token<'a>,
}

type ParseFn<'a> = fn(&mut Parser<'a>) -> Result<(), ParserError>;

struct ParseRule<'a> {
  prefix: Option<ParseFn<'a>>,
  infix: Option<ParseFn<'a>>,
  precedence: Precedence,
}

impl <'a> ParseRule<'a> {
  fn new(prefix: Option<ParseFn<'a>>, infix: Option<ParseFn<'a>>, precedence: Precedence) -> ParseRule<'a> {
    ParseRule {
      prefix,
      infix,
      precedence,
    }
  }
}

impl<'a> Parser<'a> {
  pub fn new(source: &'a str, chunk: &'a mut Chunk) -> Parser<'a> {
    Parser {
      scanner: Scanner::new(source),
      chunk,
      errors: Vec::new(),
      panic_mode: false,
      current: Token::new(TokenType::Eof, "", 0),
      previous: Token::new(TokenType::Eof, "", 0),
    }
  }

  pub fn end(&mut self) {
    self.emit_opcode(OpCode::Return);
  }

  pub fn take_errors(&mut self) -> Vec<ParserError> {
    replace(&mut self.errors, Vec::new())
  }

  fn binary(&mut self) -> Result<(), ParserError> {
    let previous = &self.previous;
    let op_type = previous.get_type();

    // Remember the operator, including if we need a not.
    let mut add_not = false;
    let opcode = match op_type {
      TokenType::EqualEqual => OpCode::Equal,
      TokenType::BangEqual => {
        add_not = true;
        OpCode::Equal
      }
      TokenType::Greater => OpCode::Greater,
      TokenType::GreaterEqual => {
        add_not = true;
        OpCode::Less
      }
      TokenType::Less => OpCode::Less,
      TokenType::LessEqual => {
        add_not = true;
        OpCode::Greater
      }
      TokenType::Plus => OpCode::Add,
      TokenType::Minus => OpCode::Subtract,
      TokenType::Star => OpCode::Multiply,
      TokenType::Slash => OpCode::Divide,
      _ => {
        let error =
            format!("Invalid binary operator {}", previous.get_lexeme());
        let line = previous.get_line();
        return Err(ParserError::UnexpectedToken(error, line));
      },
    };

    // Compile the right operand.
    let rule = Parser::get_rule(op_type);
    self.parse_precedence(rule.precedence.one_higher())?;

    self.emit_opcode(opcode);
    if add_not {
      self.emit_opcode(OpCode::Not);
    }
    Ok(())
  }

  fn literal(&mut self) -> Result<(), ParserError> {
    let prev = &self.previous;
    match prev.get_type() {
      TokenType::False => self.emit_opcode(OpCode::False),
      TokenType::Nil => self.emit_opcode(OpCode::Nil),
      TokenType::True => self.emit_opcode(OpCode::True),
      _ => {
        let message = format!("Invalid literal {}", prev.get_lexeme());
        return Err(ParserError::InternalError(message, prev.get_line()));
      },
    }
    Ok(())
  }

  pub fn expression(&mut self) -> Result<(), ParserError> {
    self.parse_precedence(Precedence::Assignment)
  }

  fn get_rule(token: &TokenType) -> ParseRule<'a> {
    match token {
      TokenType::LeftParen => ParseRule::new(Some(Parser::grouping), None, Precedence::None),
      TokenType::False | TokenType::Nil | TokenType::True => ParseRule::new(Some(Parser::literal), None, Precedence::None),
      TokenType::Minus => ParseRule::new(Some(Parser::unary), Some(Parser::binary), Precedence::Term),
      TokenType::Plus => ParseRule::new(None, Some(Parser::binary), Precedence::Term),
      TokenType::Slash | TokenType::Star =>
          ParseRule::new(None, Some(Parser::binary), Precedence::Factor),
      TokenType::Bang => ParseRule::new(Some(Parser::unary), None, Precedence::None),
      TokenType::BangEqual | TokenType::EqualEqual => ParseRule::new(None, Some(Parser::binary), Precedence::Equality),
      TokenType::Greater | TokenType::GreaterEqual => ParseRule::new(None, Some(Parser::binary), Precedence::Comparison),
      TokenType::Less | TokenType::LessEqual => ParseRule::new(None, Some(Parser::binary), Precedence::Comparison),
      TokenType::Number(_) => ParseRule::new(Some(Parser::number), None, Precedence::None),
      _ => ParseRule::new(None, None, Precedence::None),
    }
  }

  fn parse_precedence(&mut self, precedence: Precedence) -> Result<(), ParserError> {
    self.advance();
    if let Some(prefix_fn) = Parser::get_rule(self.previous.get_type()).prefix {
      prefix_fn(self)?;
      while precedence <= Parser::get_rule(self.current.get_type()).precedence {
        self.advance();

        if let Some(infix_fn) = Parser::get_rule(self.previous.get_type()).infix {
          infix_fn(self)?;
        } else {
          let prev = &self.previous;
          let message = format!("No infix parser rule for {}", prev.get_lexeme());
          return Err(ParserError::InternalError(message, prev.get_line()));
        }
      }
      Ok(())
    } else {
      let got = self.previous.get_lexeme().to_string();
      let line = self.previous.get_line();
      Err(ParserError::ExpectExpression(got, line))
    }
  }

  fn grouping(&mut self) -> Result<(), ParserError> {
    self.expression()?;
    self.consume(TokenType::RightParen, "Expect ')' after expression.")?;
    Ok(())
  }

  fn unary(&mut self) -> Result<(), ParserError> {
    let prev = &self.previous;
    match prev.get_type() {
      // FIXME: We need to pass the line number here.
      TokenType::Bang => {
        self.parse_precedence(Precedence::Unary)?;  // Compile the operand.
        self.emit_opcode(OpCode::Not);
      },
      TokenType::Minus => {
        self.parse_precedence(Precedence::Unary)?;  // Compile the operand.
        self.emit_opcode(OpCode::Negate);
      }
      _ => {
        let message = format!("Invalid unary op {}", prev.get_lexeme());
        return Err(ParserError::InternalError(message, prev.get_line()));
      },
    };
    Ok(())
  }

  fn number(&mut self) -> Result<(), ParserError> {
    return match self.previous.get_type() {
      TokenType::Number(num) => {
        let val = Value::Number(*num);
        self.emit_constant(val)
      },
      _ => Err(ParserError::TypeMismatch(
          TokenType::Number(0f64), self.previous.get_type().clone())),
    }
  }

  pub fn advance(&mut self) {
    let result = self.scanner.scan_token();
    if let Ok(new_token) = result {
      let old_value = std::mem::replace(&mut self.current, new_token);
      self.previous = old_value;
    } else {
      // FIXME: Handle scanner errors
      println!("scanner error {:?}", result);
    }
  }

  pub fn consume(&mut self, token: TokenType, message: &str) -> Result<(), ParserError> {
    if *self.current.get_type() == token {
      self.advance();
      Ok(())
    } else {
      Err(ParserError::UnexpectedToken(message.to_string(), self.current.get_line()))
    }
  }

  fn emit_bytecode(&mut self, bytecode: ByteCode) {
    self.chunk.write(bytecode, self.previous.get_line());
  }

  fn emit_opcode(&mut self, opcode: OpCode) {
    self.emit_bytecode(opcode as u8);
  }

  fn emit_constant(&mut self, value: Value) -> Result<(), ParserError> {
    match self.chunk.add_constant(value) {
      Ok(idx) => {
        self.emit_opcode(OpCode::Constant);
        self.emit_bytecode(idx);
        Ok(())
      },
      Err(value) => {
        Err(ParserError::TooManyConstants(value, self.previous.get_line()))
      },
    }
  }
}
