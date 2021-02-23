use crate::syntax::{parser::Parser, token::TokenType};

use super::{bytecode::Chunk, disassembler::disassemble_chunk};

pub fn compile(text: &str) -> Result<Chunk, ()> {
  let mut chunk = Chunk::default();
  {
    let mut parser = Parser::new(text, &mut chunk);
    parser.advance();
    let result = parser.expression();
    println!("{:?}, internal errors: {:?}", result, parser.take_errors());
    parser.end();
    if let Err(err) = parser.consume(TokenType::Eof, "Expected Eof") {
      eprintln!("{:?}", err);
    }
  }
  println!("{}", disassemble_chunk(&chunk, "code"));
  Ok(chunk)
}
