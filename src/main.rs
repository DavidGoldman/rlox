mod syntax;
mod vm;

use std::{env, io, process};

use io::{Write, stdout};
use process::exit;
use syntax::token::TokenType;
use syntax::scanner::Scanner;

fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {
  let args: Vec<String> = env::args().collect();
  match args.len() {
    1 => repl(),
    2 => run_file(&args[1]),
    _ => {
      eprintln!("Usage: rlox [path]\n");
      exit(64);
    }
  }
}

fn interpret(source: &str) {
  let mut scanner = Scanner::new(source);
  loop {
    match scanner.scan_token() {
      Ok(token) => {
        println!("{:?}", token);
        if *token.get_type() == TokenType::Eof {
          break;
        }
      },
      Err(error) => {
        println!("{:?}", error);
      },
    }
  }
}

fn run_file(file_path: &str) -> Result<(), Box<dyn std::error::Error + 'static>> {
  let file_contents = std::fs::read_to_string(file_path)?;
  interpret(&file_contents);
  Ok(())
}

fn repl() -> Result<(), Box<dyn std::error::Error + 'static>> {
  let mut input = String::new();
  print_prompt();

  while let Ok(_) = io::stdin().read_line(&mut input) {
    match input.trim().as_ref() {
      "quit" => break,
      _ => {
        interpret(&input);
        input.clear();
        print_prompt();
      }
    }
  }
  Ok(())
}

fn print_prompt() {
  print!("> ");
  let _ = stdout().flush();
}
