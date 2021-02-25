use std::{convert::TryFrom, fmt::Display};

use string_interner::StringInterner;

use crate::vm::bytecode::{ByteCode, Chunk, ChunkConstant, OpCode};

use super::{
    scanner::Scanner,
    token::{LiteralConstant, Token, TokenErrContext, TokenType},
};

pub enum ParserError {
    ExpectExpression(TokenErrContext),
    InternalError(TokenErrContext, String),
    InvalidAssignment(TokenErrContext),
    TooManyConstants(TokenErrContext),
    UnexpectedToken(TokenErrContext, String),
}

impl Display for ParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParserError::ExpectExpression(ctx) => write!(f, "{}: Expect expression", ctx),
            ParserError::InternalError(ctx, msg) => write!(f, "{}: {}", ctx, msg),
            ParserError::InvalidAssignment(ctx) => write!(f, "{}: Invalid assignment", ctx),
            ParserError::TooManyConstants(ctx) => write!(f, "{}: Too many constants", ctx),
            ParserError::UnexpectedToken(ctx, msg) => write!(f, "{}: {}", ctx, msg),
        }
    }
}

#[derive(PartialEq, PartialOrd, Eq, Ord, Copy, Clone)]
#[repr(u8)]
enum Precedence {
    None = 0,
    Assignment, // =
    Or,         // or
    And,        // and
    Equality,   // == !=
    Comparison, // < > <= >=
    Term,       // + -
    Factor,     // * /
    Unary,      // ! -
    Call,       // . ()
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
        Precedence::try_from(*self as u8 + 1)
            .ok()
            .unwrap_or(Precedence::Primary)
    }
}

pub struct Parser<'a> {
    scanner: Scanner<'a>,
    chunk: &'a mut Chunk,
    interner: &'a mut StringInterner,
    current: Token<'a>,
    previous: Token<'a>,
}

type ParseFn<'a> = fn(&mut Parser<'a>, bool) -> Result<(), ParserError>;

struct ParseRule<'a> {
    prefix: Option<ParseFn<'a>>,
    infix: Option<ParseFn<'a>>,
    precedence: Precedence,
}

impl<'a> ParseRule<'a> {
    fn new(
        prefix: Option<ParseFn<'a>>,
        infix: Option<ParseFn<'a>>,
        precedence: Precedence,
    ) -> ParseRule<'a> {
        ParseRule {
            prefix,
            infix,
            precedence,
        }
    }
}

impl<'a> Parser<'a> {
    pub fn new(
        source: &'a str,
        chunk: &'a mut Chunk,
        interner: &'a mut StringInterner,
    ) -> Parser<'a> {
        Parser {
            scanner: Scanner::new(source),
            chunk,
            interner,
            current: Token::new(TokenType::Eof, "", LiteralConstant::None, 0),
            previous: Token::new(TokenType::Eof, "", LiteralConstant::None, 0),
        }
    }

    pub fn is_done(&self) -> bool {
        self.scanner.at_end()
    }

    pub fn end(&mut self) {
        self.emit_opcode(OpCode::Return);
    }

    fn binary(&mut self, _can_assign: bool) -> Result<(), ParserError> {
        let previous = &self.previous;
        let op_type = previous.token_type();

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
                let error = format!("Invalid binary operator {}", previous.lexeme());
                let err_ctx = previous.to_err_context();
                return Err(ParserError::UnexpectedToken(err_ctx, error));
            }
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

    fn literal(&mut self, _can_assign: bool) -> Result<(), ParserError> {
        let prev = &self.previous;
        match prev.token_type() {
            TokenType::False => self.emit_opcode(OpCode::False),
            TokenType::Nil => self.emit_opcode(OpCode::Nil),
            TokenType::True => self.emit_opcode(OpCode::True),
            _ => {
                let err_ctx = prev.to_err_context();
                let msg = "Invalid literal".to_string();
                return Err(ParserError::InternalError(err_ctx, msg));
            }
        }
        Ok(())
    }

    fn expression(&mut self) -> Result<(), ParserError> {
        self.parse_precedence(Precedence::Assignment)
    }

    fn var_declaration(&mut self) -> Result<(), ParserError> {
        self.consume(TokenType::Identifier, "Expect variable name.")?;
        let maybe_global = self.parse_variable();

        if self.match_token(TokenType::Equal) {
            self.expression()?;
        } else {
            self.emit_opcode(OpCode::Nil);
        }
        self.consume(
            TokenType::Semicolon,
            "Expect ';' after variable declaration.",
        )?;

        self.emit_constant(maybe_global, OpCode::DefineGlobal)
    }

    fn expression_statement(&mut self) -> Result<(), ParserError> {
        self.expression()?;
        self.consume(TokenType::Semicolon, "Expect ';' after expression.")?;
        self.emit_opcode(OpCode::Pop);
        Ok(())
    }

    fn print_statement(&mut self) -> Result<(), ParserError> {
        self.expression()?;
        self.consume(TokenType::Semicolon, "Expect ';' after value.")?;
        self.emit_opcode(OpCode::Print);
        Ok(())
    }

    fn synchronize(&mut self) {
        use TokenType::*;
        while *self.current.token_type() != Eof {
            // Skip until we reach something that looks like a statement boundary.
            if *self.previous.token_type() == Semicolon {
                return;
            }

            match *self.current.token_type() {
                Class | Fun | Var | For | If | While | Print | Return => {
                    return;
                }
                _ => {}
            }

            self.advance();
        }
    }

    pub fn declaration(&mut self) -> Result<(), ParserError> {
        let result = if self.match_token(TokenType::Var) {
            self.var_declaration()
        } else {
            self.statement()
        };
        if let Err(err) = result {
            self.synchronize();
            Err(err)
        } else {
            Ok(())
        }
    }

    fn statement(&mut self) -> Result<(), ParserError> {
        if self.match_token(TokenType::Print) {
            self.print_statement()
        } else {
            self.expression_statement()
        }
    }

    fn get_rule(token: &TokenType) -> ParseRule<'a> {
        match token {
            TokenType::LeftParen => ParseRule::new(Some(Parser::grouping), None, Precedence::None),
            TokenType::False | TokenType::Nil | TokenType::True => {
                ParseRule::new(Some(Parser::literal), None, Precedence::None)
            }
            TokenType::Minus => {
                ParseRule::new(Some(Parser::unary), Some(Parser::binary), Precedence::Term)
            }
            TokenType::Plus => ParseRule::new(None, Some(Parser::binary), Precedence::Term),
            TokenType::Slash | TokenType::Star => {
                ParseRule::new(None, Some(Parser::binary), Precedence::Factor)
            }
            TokenType::Bang => ParseRule::new(Some(Parser::unary), None, Precedence::None),
            TokenType::BangEqual | TokenType::EqualEqual => {
                ParseRule::new(None, Some(Parser::binary), Precedence::Equality)
            }
            TokenType::Greater | TokenType::GreaterEqual => {
                ParseRule::new(None, Some(Parser::binary), Precedence::Comparison)
            }
            TokenType::Less | TokenType::LessEqual => {
                ParseRule::new(None, Some(Parser::binary), Precedence::Comparison)
            }
            TokenType::Identifier => ParseRule::new(Some(Parser::variable), None, Precedence::None),
            TokenType::String => ParseRule::new(Some(Parser::string), None, Precedence::None),
            TokenType::Number => ParseRule::new(Some(Parser::number), None, Precedence::None),
            _ => ParseRule::new(None, None, Precedence::None),
        }
    }

    fn parse_precedence(&mut self, precedence: Precedence) -> Result<(), ParserError> {
        self.advance();
        if let Some(prefix_fn) = Parser::get_rule(self.previous.token_type()).prefix {
            let can_assign = precedence <= Precedence::Assignment;
            prefix_fn(self, can_assign)?;
            while precedence <= Parser::get_rule(self.current.token_type()).precedence {
                self.advance();

                if let Some(infix_fn) = Parser::get_rule(self.previous.token_type()).infix {
                    infix_fn(self, can_assign)?;
                } else {
                    let err_ctx = self.previous.to_err_context();
                    let msg = "No infix parser rule".to_string();
                    return Err(ParserError::InternalError(err_ctx, msg));
                }
            }
            if can_assign && self.match_token(TokenType::Equal) {
                let err_ctx = self.current.to_err_context();
                Err(ParserError::InvalidAssignment(err_ctx))
            } else {
                Ok(())
            }
        } else {
            let err_ctx = self.previous.to_err_context();
            Err(ParserError::ExpectExpression(err_ctx))
        }
    }

    fn grouping(&mut self, _can_assign: bool) -> Result<(), ParserError> {
        self.expression()?;
        self.consume(TokenType::RightParen, "Expect ')' after expression.")?;
        Ok(())
    }

    fn unary(&mut self, _can_assign: bool) -> Result<(), ParserError> {
        let prev = &self.previous;
        match prev.token_type() {
            // FIXME: We need to pass the line number here.
            TokenType::Bang => {
                self.parse_precedence(Precedence::Unary)?; // Compile the operand.
                self.emit_opcode(OpCode::Not);
            }
            TokenType::Minus => {
                self.parse_precedence(Precedence::Unary)?; // Compile the operand.
                self.emit_opcode(OpCode::Negate);
            }
            _ => {
                let err_ctx = self.previous.to_err_context();
                let msg = "Invalid unary operator".to_string();
                return Err(ParserError::InternalError(err_ctx, msg));
            }
        };
        Ok(())
    }

    fn parse_variable(&mut self) -> Option<ByteCode> {
        let name = self.previous.lexeme();
        self.chunk
            .add_constant(&mut self.interner, ChunkConstant::String(name))
    }

    fn string(&mut self, _can_assign: bool) -> Result<(), ParserError> {
        if *self.previous.token_type() == TokenType::String {
            if let LiteralConstant::String(str) = self.previous.literal() {
                let maybe_global = self
                    .chunk
                    .add_constant(&mut self.interner, ChunkConstant::String(str));
                return self.emit_constant(maybe_global, OpCode::Constant);
            }
        }
        return Err(ParserError::InternalError(
            self.previous.to_err_context(),
            "invalid string literal".to_string(),
        ));
    }

    fn named_variable(&mut self, can_assign: bool) -> Result<(), ParserError> {
        let name = self.previous.lexeme();
        let maybe_global = self
            .chunk
            .add_constant(&mut self.interner, ChunkConstant::String(name));

        if maybe_global == None {
            return Err(self.err_constants());
        }

        if can_assign && self.match_token(TokenType::Equal) {
            self.expression()?;
            self.emit_constant(maybe_global, OpCode::SetGlobal)
        } else {
            self.emit_constant(maybe_global, OpCode::GetGlobal)
        }
    }

    fn variable(&mut self, can_assign: bool) -> Result<(), ParserError> {
        self.named_variable(can_assign)
    }

    fn number(&mut self, _can_assign: bool) -> Result<(), ParserError> {
        if *self.previous.token_type() == TokenType::Number {
            if let LiteralConstant::Number(num) = self.previous.literal() {
                let res = self
                    .chunk
                    .add_constant(&mut self.interner, ChunkConstant::Number(num));
                return self.emit_constant(res, OpCode::Constant);
            }
        }
        return Err(ParserError::InternalError(
            self.previous.to_err_context(),
            "invalid number literal".to_string(),
        ));
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
        if *self.current.token_type() == token {
            self.advance();
            Ok(())
        } else {
            Err(ParserError::UnexpectedToken(
                self.current.to_err_context(),
                message.to_string(),
            ))
        }
    }

    fn match_token(&mut self, token: TokenType) -> bool {
        if !self.check(token) {
            false
        } else {
            self.advance();
            true
        }
    }

    fn check(&self, token: TokenType) -> bool {
        *self.current.token_type() == token
    }

    fn emit_bytecode(&mut self, bytecode: ByteCode) {
        self.chunk.write(bytecode, self.previous.line());
    }

    fn emit_opcode(&mut self, opcode: OpCode) {
        self.emit_bytecode(opcode as u8);
    }

    fn emit_constant(
        &mut self,
        maybe_global: Option<ByteCode>,
        opcode: OpCode,
    ) -> Result<(), ParserError> {
        match maybe_global {
            Some(idx) => {
                self.emit_opcode(opcode);
                self.emit_bytecode(idx);
                Ok(())
            }
            None => Err(self.err_constants()),
        }
    }

    fn err_constants(&self) -> ParserError {
        ParserError::TooManyConstants(self.previous.to_err_context())
    }
}
