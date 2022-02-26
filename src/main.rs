mod lexer;
mod parser;
mod parsetable;

use lexer::{Lexer, Token};
use parser::Parser;
use std::env;
use std::fs;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    println!("{:?}", args);

    if args.len() != 2 {
        println!("usage: ./lrjson file");
        process::exit(1);
    }
    let contents = fs::read_to_string(&args[1]);
    let mut lexer = Lexer::new(contents.unwrap());
    let mut token = lexer.next_token(true);
    while token != Some(Token::EOF) {
        println!("{:?}", token);
        token = lexer.next_token(true);
    }
    println!("{:?}", token);

    println!("\n\n--------------\n\n");

    let contents = fs::read_to_string(&args[1]);
    let mut parser: Parser = Parser::new(Lexer::new(contents.unwrap()));
    let result = parser.step();
    println!("{:?}", result);
}
