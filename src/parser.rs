use crate::lexer::{Keyword, Lexer, Token};
use anyhow::{Context, Result};
use ordered_float::NotNan;
use std::collections::HashMap;
use std::fmt;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Unexpected token {0:?}")]
    Token(Token),
    #[error("Expected token {0:?} but got token {1:?}")]
    Expected(Token, Token),
    #[error("Unexpected EOF.")]
    EOF,
    #[error("Unknown parser error occured.")]
    Unknown,
}

#[derive(Debug)]
pub struct HeadData {
    definitions: HashMap<String, Node>,
}

type Values = HashMap<String, ExpressionType>;

pub enum ExpressionType {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    // Vector Types
    Runtime(fn(&Values) -> ExpressionType),
}

impl fmt::Debug for ExpressionType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "primitive")
    }
}

#[derive(Debug)]
pub struct Block {
    definitions: Values,
    statements: Vec<Node>,
}

#[derive(Debug)]
pub struct PatternData {
    block: Block,
}

#[derive(Debug)]
pub struct BulletData {
    definitions: Values,
}

#[derive(Debug)]
pub struct PathData {}

#[derive(Debug)]
pub struct AssignmentData {
    lvalue: String,
    rvalue: ExpressionType,
}

#[derive(Debug)]
pub enum WaitData {
    Frames(i64),
    Time(f64),
}

#[derive(Debug)]
pub enum Conditional {
    When(ExpressionType),
    Unless(ExpressionType),
}

#[derive(Debug)]
pub struct ForData {
    initial_definitions: Values,
    condition: Conditional,
    body: Block,
}

#[derive(Debug)]
pub struct SpawnData {
    definitions: Values,
}

#[derive(Debug)]
pub enum Node {
    Head(HeadData),
    Pattern(PatternData),
    Bullet(BulletData),
    Path(PathData),
    Assignment(AssignmentData),
    Wait(WaitData),
    For(ForData),
    Expression(ExpressionType),
    Spawn(SpawnData),
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fn fmt_single(n: &Node, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "test")
        }
        fmt_single(&self, f)
    }
}

pub struct Parser {
    lexer: Lexer,
}

type NamedToplevel = (String, Node);

impl Parser {
    pub fn new(lexer: Lexer) -> Parser {
        Parser { lexer }
    }

    pub fn next_token(&mut self) -> Result<Token> {
        self.lexer.next_token().ok_or(ParseError::EOF.into())
    }

    pub fn expect_next(&mut self, expected: Token) -> Result<Token> {
        let t = self.next_token()?;
        if t != expected {
            return Err(ParseError::Expected(expected, t).into());
        }
        Ok(t)
    }

    pub fn evaluate(&mut self) -> Result<Node> {
        self.parse_head()
    }

    pub fn parse_head(&mut self) -> Result<Node> {
        let mut head = Node::Head(HeadData {
            definitions: HashMap::new(),
        });
        while let Some(token) = self.lexer.next_token() {
            let (name, node) = match token {
                Token::EOF => {
                    break;
                }
                Token::Keyword(Keyword::Pattern) => self.parse_pattern()?,
                _ => {
                    return Err(ParseError::Token(token).into());
                }
            };
            if let Node::Head(ref mut head) = head {
                head.definitions.insert(name, node);
            }
        }
        Ok(head)
    }

    pub fn parse_pattern(&mut self) -> Result<NamedToplevel> {
        let name = self.next_token().context("Parsing pattern.")?;
        if let Token::Id(name) = name {
            self.expect_next(Token::Assign)?;
            let block = self.parse_block()?;
            let pattern_node = Node::Pattern(PatternData { block });
            return Ok((name, pattern_node));
        } else {
            return Err(ParseError::Expected(Token::String("Id".to_string()), name).into());
        }
    }

    pub fn parse_block(&mut self) -> Result<Block> {
        self.expect_next(Token::OpenBlock)
            .context("Parsing block.")?;
        let mut block = Block {
            definitions: HashMap::new(),
            statements: Vec::new(),
        };
        self.expect_next(Token::CloseBlock)?;
        Ok(block)
    }
}
