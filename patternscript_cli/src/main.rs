use patternscript::interpreter::entity::{Entity, Hitbox};
use patternscript::interpreter::*;
use patternscript::parser::lexer::{Lexer, Token};
use patternscript::parser::parser::*;
use std::env;
use std::fs;
use std::process;

use cgmath::{Deg, Vector2, Vector3};

const USAGE: &'static str = "./patternscript [action] [file]
    actions:
        -p : parse
        -l : lex
        -i : initialize interpreter and dump details from pattern
    file: 
        a patternscript file, see examples";

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
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

    if &args[1] == "-i" {
        let mut world =
            Interpreter::from_parse_result(Parser::parse_from_file(args[2].clone()).unwrap())
                .unwrap();
        let pattern_name = &args[3];

        let e = Entity {
            position: Vector2 { x: 20.0, y: 20.0 },
            velocity: Vector2 { x: 0.0, y: 20.0 },
            rotation: Deg(0),
            lifetime: 240,
            color: Vector3 { x: 255, y: 0, z: 0 },
            hitbox: Hitbox {
                size: Vector2 { x: 8, y: 8 },
                offset: Vector2 { x: 0.0, y: 0.0 },
                hitbox_type: entity::HitboxType::Rectangle,
            },
            behavior: entity::Behavior::Pattern(pattern_name.clone()),
        };

        world.spawn_direct(&e);
        world.step();

        println!("{:?}", world);
    }
}
