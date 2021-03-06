use std::fmt::Display;

use super::token::{LiteralConstant, Token, TokenType};

pub struct Scanner<'a> {
    source: &'a str,
    start: usize,
    current: usize,
    line: usize,
}

pub struct SourceErrContext {
    pub lexeme: String,
    pub line: usize,
}

impl Display for SourceErrContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[line {}] Error at '{}'", self.line, self.lexeme)
    }
}

impl SourceErrContext {
    fn new(lexeme: String, line: usize) -> Self {
        SourceErrContext { lexeme, line }
    }
}

pub enum ScannerError {
    UnexpectedEof(usize),
    UnsupportedChar(SourceErrContext, u8),
    InvalidNumber(SourceErrContext),
}

impl Display for ScannerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScannerError::UnexpectedEof(line) => write!(f, "[line {}] Error at end", line),
            ScannerError::UnsupportedChar(ctx, char) => {
                write!(f, "{}: invalid char '{}'", ctx, *char as char)
            }
            ScannerError::InvalidNumber(ctx) => write!(f, "{}: invalid number", ctx),
        }
    }
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
                    let ctx = self.err_context();
                    Err(ScannerError::UnsupportedChar(ctx, byte))
                }
            };
        } else {
            return Ok(self.make_token(TokenType::Eof));
        }
    }

    fn make_match_token(
        &mut self,
        byte: u8,
        match_type: TokenType,
        no_match_type: TokenType,
    ) -> Token<'a> {
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

    fn err_context(&self) -> SourceErrContext {
        SourceErrContext::new(self.current_lexeme().to_string(), self.line)
    }

    fn make_token(&mut self, token_type: TokenType) -> Token<'a> {
        self.make_literal(token_type, LiteralConstant::None)
    }

    fn make_literal(&mut self, token_type: TokenType, literal: LiteralConstant<'a>) -> Token<'a> {
        Token::new(token_type, self.current_lexeme(), literal, self.line)
    }

    fn make_string(&mut self) -> Result<Token<'a>, ScannerError> {
        // Read until Eof or ".
        loop {
            match self.current_byte() {
                Some(b'"') => {
                    break;
                }
                None => {
                    break;
                }
                Some(b'\n') => {
                    self.line += 1;
                    self.advance();
                }
                Some(_) => {
                    self.advance();
                }
            }
        }

        if self.at_end() {
            Err(ScannerError::UnexpectedEof(self.line))
        } else {
            self.advance(); // The closing quote.
            let parsed_str = &self.source[self.start + 1..self.current - 1];
            Ok(self.make_literal(TokenType::String, LiteralConstant::String(parsed_str)))
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
        if let Ok(num) = number_str.parse::<f64>() {
            Ok(self.make_literal(TokenType::Number, LiteralConstant::Number(num)))
        } else {
            Err(ScannerError::InvalidNumber(self.err_context()))
        }
    }

    fn make_identifier(&mut self) -> Result<Token<'a>, ScannerError> {
        loop {
            match self.current_byte().unwrap_or(0) {
                b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' => {
                    self.advance();
                }
                _ => {
                    break;
                }
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
            _ => TokenType::Identifier,
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

        self.current = current;
        result
    }

    pub fn at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn skip_whitespace_and_comments(&mut self) {
        loop {
            match self.current_byte().unwrap_or(0) {
                b' ' | b'\r' | b'\t' => {
                    self.advance();
                }
                b'\n' => {
                    self.line += 1;
                    self.advance();
                }
                b'/' => {
                    let next_byte = self.peek_next_byte();
                    if next_byte != Some(b'/') {
                        return;
                    }
                    self.advance();
                    // A comment goes until the end of the line or Eof.
                    loop {
                        match self.current_byte() {
                            Some(b'\n') => {
                                break;
                            }
                            Some(_) => {
                                self.advance();
                            }
                            None => {
                                break;
                            }
                        }
                    }
                }
                _ => {
                    return;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check_type(scanner: &mut Scanner, expected: TokenType) {
        let res = scanner.scan_token();
        match res {
            Ok(token) => assert_eq!(*token.token_type(), expected),
            Err(err) => panic!("Unexpected error: {}", err),
        }
    }

    #[test]
    fn scans_token_types() {
        let mut scanner = Scanner::new("var a = \"hi\" or 5; if (a) { print a; }");
        check_type(&mut scanner, TokenType::Var);
        check_type(&mut scanner, TokenType::Identifier);
        check_type(&mut scanner, TokenType::Equal);
        check_type(&mut scanner, TokenType::String);
        check_type(&mut scanner, TokenType::Or);
        check_type(&mut scanner, TokenType::Number);
        check_type(&mut scanner, TokenType::Semicolon);
        check_type(&mut scanner, TokenType::If);
        check_type(&mut scanner, TokenType::LeftParen);
        check_type(&mut scanner, TokenType::Identifier);
        check_type(&mut scanner, TokenType::RightParen);
        check_type(&mut scanner, TokenType::LeftBrace);
        check_type(&mut scanner, TokenType::Print);
        check_type(&mut scanner, TokenType::Identifier);
        check_type(&mut scanner, TokenType::Semicolon);
        check_type(&mut scanner, TokenType::RightBrace);
        check_type(&mut scanner, TokenType::Eof);
    }
}
