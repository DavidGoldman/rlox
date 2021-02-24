use string_interner::StringInterner;

use crate::syntax::{parser::Parser, token::TokenType};

use super::{bytecode::Chunk, disassembler::disassemble_chunk};

pub fn compile(text: &str, strings: &mut StringInterner) -> Result<Chunk, ()> {
  let mut chunk = Chunk::default();
  {
    let mut parser = Parser::new(text, &mut chunk, strings);
    parser.advance();
    while !parser.is_done() {
      let result = parser.declaration();
      println!("{:?}, internal errors: {:?}", result, parser.take_errors());
    }
    parser.end();
    if let Err(err) = parser.consume(TokenType::Eof, "Expected Eof") {
      eprintln!("{:?}", err);
    }
  }
  println!("{}", disassemble_chunk(&chunk, "code"));
  Ok(chunk)
}
