mod syntax;
mod vm;

use std::{env, io, process};

use io::{Write, stdout};
use process::exit;
use vm::{compiler::compile, vm::Vm};

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
  // FIXME: error handling
  if let Ok(chunk) = compile(source) {
    let mut vm = Vm::new(&chunk);
    let _ = vm.run();
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
