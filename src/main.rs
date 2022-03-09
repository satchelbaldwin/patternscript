mod lexer;
mod parser;

use lexer::{Lexer, Token};
use std::env;
use std::fs;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    println!("{:?}", args);

    if args.len() != 2 {
        println!("usage: ./??? file");
        process::exit(1);
    }
    let contents = fs::read_to_string(&args[1]);
    let mut lexer = Lexer::new(contents.unwrap());
    let mut token = lexer.next_token();
    while token != Some(Token::EOF) {
        println!("{:?}", token.unwrap());
        token = lexer.next_token();
    }
    println!("{:?}", token.unwrap());
}
