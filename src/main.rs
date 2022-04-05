mod lexer;
mod parser;
mod types;

use lexer::{Lexer, Token};
use parser::Parser;
use std::env;
use std::fs;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    println!("{:?}", args);

    if args.len() < 2 {
        println!("usage: ./patternscript [options] file");
        process::exit(1);
    }

    if &args[1] == "-l" {
        println!("Lexer:\n---------\n");

        let contents = fs::read_to_string(&args[2]);
        let mut lexer = Lexer::new(contents.unwrap());
        let mut token = lexer.next_token();
        while token != Some(Token::EOF) {
            println!("{:?}", token.unwrap());
            token = lexer.next_token();
        }
        println!("{:?}", token.unwrap());
    }

    if &args[1] == "-p" {
        println!("Parser:\n---------\n");
        let contents = fs::read_to_string(&args[2]);
        let lexer = Lexer::new(contents.unwrap());
        let mut parser = Parser::new(lexer);
        let mut head = parser.evaluate();
        match head {
            Ok(h) => println!("{:?}", h),
            Err(e) => println!("{}", e),
        }
    }
}
