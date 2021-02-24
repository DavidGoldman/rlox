mod syntax;
mod vm;

use std::{env, io, process};

use io::{Write, stdout};
use process::exit;
use string_interner::StringInterner;
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

fn interpret(source: &str, vm: &mut Vm, interner: &mut StringInterner) {
  // FIXME: error handling
  if let Ok(chunk) = compile(source, interner) {
    let _ = vm.run(chunk, interner);
  }
}

fn run_file(file_path: &str) -> Result<(), Box<dyn std::error::Error + 'static>> {
  let file_contents = std::fs::read_to_string(file_path)?;

  let mut vm = Vm::default();
  let mut interner = StringInterner::default();
  interpret(&file_contents, &mut vm, &mut interner);
  Ok(())
}

fn repl() -> Result<(), Box<dyn std::error::Error + 'static>> {
  let mut interner = StringInterner::default();
  let mut vm = Vm::default();

  let mut input = String::new();
  print_prompt();

  while let Ok(_) = io::stdin().read_line(&mut input) {
    match input.trim().as_ref() {
      "quit" => break,
      _ => {
        interpret(&input, &mut vm, &mut interner);
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
