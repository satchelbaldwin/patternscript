use patternscript::parser::lexer::{Lexer, Token};
use patternscript::parser::parser::*;
use std::env;
use std::fs;
use std::process;

const USAGE: &'static str = "./patternscript [action] [file]
    actions:
        -p : parse
        -l : lex
    file: 
        a patternscript file, see examples";

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("{}", USAGE);
        process::exit(1);
    }

    if &args[1] == "-l" {
        let contents = fs::read_to_string(&args[2]);
        let mut lexer = Lexer::new(contents.unwrap());
        let mut token = lexer.next_token();
        while token != Some(Token::EOF) {
            println!("{:?}", token.unwrap());
            token = lexer.next_token();
        }
        println!("{:?}", token.unwrap());
        process::exit(0);
    }

    if &args[1] == "-p" {
        let head = Parser::parse_from_file(args[2].clone());
        match head {
            Ok(h) => println!("{:?}", h),
            Err(e) => println!("{}", e),
        }
        process::exit(0);
    }
}
