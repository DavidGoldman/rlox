use std::{convert::TryFrom, mem::replace};

use crate::vm::{bytecode::{ByteCode, Chunk, OpCode}, value::Value};

use super::{scanner::Scanner, token::{Token, TokenType}};

#[derive(Debug)]
pub enum ParserError {
  UnexpectedEof(usize),
  UnexpectedToken(String, usize),
  TooManyConstants(usize),
  ToDo,
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
    let op_type = self.previous.get_type();

    // Remember the operator.
    let maybe_opcode = match op_type {
      TokenType::Plus => Some(OpCode::Add),
      TokenType::Minus => Some(OpCode::Subtract),
      TokenType::Star => Some(OpCode::Multiply),
      TokenType::Slash => Some(OpCode::Divide),
      _ => None,
    };

    // Compile the right operand.
    let rule = Parser::get_rule(op_type);
    self.parse_precedence(rule.precedence.one_higher())?;

    let opcode = maybe_opcode.ok_or(ParserError::ToDo)?;
    self.emit_opcode(opcode);

    Ok(())
  }

  pub fn expression(&mut self) -> Result<(), ParserError> {
    self.parse_precedence(Precedence::Assignment)
  }

  fn get_rule(token: &TokenType) -> ParseRule<'a> {
    match token {
      TokenType::LeftParen => ParseRule::new(Some(Parser::grouping), None, Precedence::None),
      TokenType::Minus => ParseRule::new(Some(Parser::unary), Some(Parser::binary), Precedence::Term),
      TokenType::Plus => ParseRule::new(None, Some(Parser::binary), Precedence::Term),
      TokenType::Slash => ParseRule::new(None, Some(Parser::binary), Precedence::Factor),
      TokenType::Star => ParseRule::new(None, Some(Parser::binary), Precedence::Factor),
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
          return Err(ParserError::ToDo);
        }
      }
      Ok(())
    } else {
      Err(ParserError::ToDo)
    }
  }

  fn grouping(&mut self) -> Result<(), ParserError> {
    self.expression()?;
    self.consume(TokenType::RightParen, "Expect ')' after expression.");
    Ok(())
  }

  fn unary(&mut self) -> Result<(), ParserError> {
    match self.previous.get_type() {
      TokenType::Minus => {
        // Compile the operand.
        self.parse_precedence(Precedence::Unary);
        // FIXME: We need to pass the line number here.
        self.emit_opcode(OpCode::Negate);
      }
      _ => {
        // Unreachable.
      },
    };
    Ok(())
  }

  fn number(&mut self) -> Result<(), ParserError> {
    return match self.previous.get_type() {
      TokenType::Number(num) => self.emit_constant(Value::Number(*num)),
      _ => Err(ParserError::ToDo),
    }
  }

  pub fn advance(&mut self) {
    let result = self.scanner.scan_token();
    if let Ok(new_token) = result {
      let old_value = std::mem::replace(&mut self.current, new_token);
      self.previous = old_value;
    } else {
      println!("scanner error {:?}", result);
    }
    // FIXME: handle errors;
  }

  pub fn consume(&mut self, token: TokenType, message: &str) {
    if *self.current.get_type() == token {
      self.advance();
      return;
    }
    // FIXME: error messages.
  }

  fn emit_bytecode(&mut self, bytecode: ByteCode) {
    self.chunk.write(bytecode, self.previous.get_line());
  }

  fn emit_opcode(&mut self, opcode: OpCode) {
    self.emit_bytecode(opcode as u8);
  }

  fn emit_constant(&mut self, value: Value) -> Result<(), ParserError> {
    match self.chunk.add_constant(value) {
      Some(idx) => {
        self.emit_opcode(OpCode::Constant);
        self.emit_bytecode(idx);
        Ok(())
      },
      None => {
        Err(ParserError::TooManyConstants(self.previous.get_line()))
      },
    }
  }

  fn error_at(&mut self, token: &Token) {
    if (self.panic_mode) {
      return;
    }
    self.panic_mode = true;
    let error = match token.get_type() {
      TokenType::Eof => ParserError::UnexpectedEof(token.get_line()),
      _ => ParserError::UnexpectedToken(token.copy_lexeme(), token.get_line())
    };
    self.errors.push(error);
  }
}
