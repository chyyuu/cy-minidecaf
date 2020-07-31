// Made by following along with https://norasandler.com/2017/11/29/Write-a-Compiler.html

use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::process::Command;

mod scanner;
mod ast;
mod generator;

use scanner::lex;
use ast::*;
use generator::generate;

fn main() {
    let argv: Vec<String> = env::args().collect();

    let mut contents = String::new();
    File::open(&argv[1]).unwrap().read_to_string(&mut contents).unwrap();

    let tokens = lex(&contents);
    println!("Scanner production:\n{:?}\n", tokens);

    // Currently this parser is only onto stage 3.
    // I am having trouble with parsing unary operators alone with terms instead of entire expressions.
    let ast = parse(&tokens);
    println!("Abstract syntax tree:\n{:#?}\n", ast);

    // Comment out everything below this line to disable code generation
    let generated = generate(&ast);
    println!("Generated assembly:\n{}", generated);

    let file_name = std::path::Path::new(&argv[1]).file_stem().unwrap().to_str().unwrap();

    let mut output_file = File::create(&format!("{}.s", file_name)).unwrap();
    output_file.write_all(generated.as_bytes()).unwrap();

    Command::new("gcc")
        .args(&["-m32", &format!("{}.s", file_name), "-o", "out"])
        .spawn()
        .unwrap()
        .wait()
        .unwrap();
}
