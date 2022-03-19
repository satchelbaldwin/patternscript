use crate::lexer::{Condition as ConditionToken, Keyword, Lexer, Token};
use anyhow::{Context, Result};
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
    #[error("Only definitions allowed in this block.")]
    Definitions,
    #[error("Invalid number.")]
    InvalidNumber,
    #[error("For definitions must be in the form of a = 1, b = 2...")]
    InvalidForDef,
    #[error("Range must be two Ints, not Floats.")]
    RangeMustBeInt,
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
    Range(i64, i64),
    Block(Block),
    // Vector Types
    Runtime(fn(&Values) -> ExpressionType),
    Unimplemented,
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

impl Block {
    pub fn new() -> Block {
        Block {
            definitions: HashMap::new(),
            statements: Vec::new(),
        }
    }
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
pub struct PathData {
    definitions: Values,
}

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
pub enum Condition {
    When(ExpressionType),
    Unless(ExpressionType),
    None,
}

#[derive(Debug)]
pub struct ForData {
    initial_definitions: Values,
    condition: Condition,
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
        self.lexer
            .next_token()
            .ok_or_else(|| ParseError::EOF.into())
    }

    pub fn lookahead(&mut self, n: u32) -> Result<Token> {
        self.lexer
            .lookahead(n)
            .ok_or_else(|| ParseError::EOF.into())
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
                Token::Keyword(Keyword::Path) => self.parse_path()?,
                Token::Keyword(Keyword::Bullet) => self.parse_bullet()?,
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
        let name = self.next_token().context("Parsing pattern...")?;
        if let Token::Id(name) = name {
            self.expect_next(Token::Assign)?;
            let block = self.parse_block()?;
            let pattern_node = Node::Pattern(PatternData { block });
            Ok((name, pattern_node))
        } else {
            Err(ParseError::Expected(Token::String("Id".to_string()), name).into())
        }
    }

    pub fn parse_block(&mut self) -> Result<Block> {
        self.expect_next(Token::OpenBlock)
            .context("Parsing block...")?;

        let mut block = Block {
            definitions: HashMap::new(),
            statements: Vec::new(),
        };

        let mut t = self.next_token()?;
        while t != Token::CloseBlock {
            // id, wait, for, spawn
            match t {
                Token::Id(id) => {
                    self.expect_next(Token::Assign)?;
                    let expression = self.parse_expression()?;
                    block.definitions.insert(id, expression);
                }
                Token::Keyword(Keyword::For) => {
                    let for_data = self.parse_for()?;
                    block.statements.push(Node::For(for_data));
                }
                Token::Keyword(Keyword::Wait) => {
                    let wait = self.parse_wait()?;
                    block.statements.push(Node::Wait(wait));
                }
                Token::Keyword(Keyword::Spawn) => {
                    let spawn = self.parse_spawn()?;
                    block.statements.push(Node::Spawn(spawn));
                }
                _ => return Err(ParseError::Token(t).into()),
            }
            t = self.next_token()?;
        }

        Ok(block)
    }

    // faster case for block when no imperative/ordered actions
    pub fn parse_values(&mut self) -> Result<Values> {
        self.expect_next(Token::OpenBlock)
            .context("Parsing values...")?;

        let mut definitions = HashMap::new();
        let mut t = self.next_token()?;
        while t != Token::CloseBlock {
            match t {
                Token::Id(id) => {
                    self.expect_next(Token::Assign)?;
                    let expression = self.parse_expression()?;
                    definitions.insert(id, expression);
                }
                _ => return Err(ParseError::Definitions.into()),
            }
            t = self.next_token()?;
        }

        Ok(definitions)
    }

    // note: currently, this should consume ending `;` or `,`
    //       should it? is this a good idea?
    pub fn parse_expression(&mut self) -> Result<ExpressionType> {
        let mut t = self.lookahead(1)?;
        if t == Token::OpenBlock {
            return Ok(ExpressionType::Block(self.parse_block()?));
        }

        t = self.next_token()?;

        // cheat and count parens to avoid commas...
        let mut unmatchedp: usize = 0;

        while (t != Token::Semicolon && t != Token::Comma && t != Token::CloseParen)
            || (unmatchedp > 0)
        {
            if t == Token::OpenParen {
                unmatchedp += 1;
            }

            if t == Token::CloseParen {
                unmatchedp -= 1;
            }

            t = self.next_token()?;
        }
        Ok(ExpressionType::Unimplemented)
    }

    pub fn parse_for(&mut self) -> Result<ForData> {
        let mut for_data = ForData {
            initial_definitions: HashMap::new(),
            condition: Condition::None,
            body: Block::new(),
        };

        self.expect_next(Token::OpenParen)?;
        let mut t = self.next_token()?;
        while t != Token::CloseParen {
            match t {
                Token::Id(id) => {
                    self.expect_next(Token::Assign)?;
                    let range = self.parse_range()?;
                    for_data.initial_definitions.insert(id, range);
                }
                Token::Comma => {}
                _ => return Err(ParseError::InvalidForDef.into()),
            }
            t = self.next_token()?;
        }

        if self.lookahead(1)? != Token::OpenBlock {
            let t = self.next_token()?;
            self.expect_next(Token::OpenParen)?;
            let expression = self.parse_expression()?;
            match t {
                Token::Condition(ConditionToken::When) => {
                    for_data.condition = Condition::When(expression);
                }
                Token::Condition(ConditionToken::Unless) => {
                    for_data.condition = Condition::Unless(expression);
                }
                _ => return Err(ParseError::Token(t).into()),
            }
            //self.expect_next(Token::CloseParen)?;
        }

        for_data.body = self.parse_block()?;

        Ok(for_data)
    }

    pub fn parse_number(&mut self) -> Result<ExpressionType> {
        let t = self.next_token()?;
        match t {
            Token::Number(n) => {
                if n.contains(".") {
                    Ok(ExpressionType::Float(n.parse::<f64>()?))
                } else {
                    Ok(ExpressionType::Int(n.parse::<i64>()?))
                }
            }
            _ => Err(ParseError::InvalidNumber.into()),
        }
    }

    pub fn parse_range(&mut self) -> Result<ExpressionType> {
        let start = self.parse_number()?;
        self.expect_next(Token::RangeSeparator)?;
        let end = self.parse_number()?;

        // both must be ints
        let error = Err(ParseError::RangeMustBeInt.into());
        match start {
            ExpressionType::Int(s) => match end {
                ExpressionType::Int(e) => Ok(ExpressionType::Range(s, e)),
                _ => error,
            },
            _ => error,
        }
    }

    pub fn parse_wait(&mut self) -> Result<WaitData> {
        self.next_token()?;
        self.next_token()?;
        self.next_token()?;
        Ok(WaitData::Frames(10))
    }

    pub fn parse_spawn(&mut self) -> Result<SpawnData> {
        let block = self.parse_block()?;
        Ok(SpawnData {
            definitions: block.definitions,
        })
    }

    pub fn parse_path(&mut self) -> Result<NamedToplevel> {
        let name = self.next_token().context("Parsing path...")?;
        if let Token::Id(name) = name {
            self.expect_next(Token::Assign)?;
            let definitions = self.parse_values()?;
            let path_node = Node::Path(PathData { definitions });
            Ok((name, path_node))
        } else {
            Err(ParseError::Expected(Token::String("Id".to_string()), name).into())
        }
    }

    pub fn parse_bullet(&mut self) -> Result<NamedToplevel> {
        let name = self.next_token().context("Parsing bullet...")?;
        if let Token::Id(name) = name {
            self.expect_next(Token::Assign)?;
            let definitions = self.parse_values()?;
            let bullet_node = Node::Bullet(BulletData { definitions });
            Ok((name, bullet_node))
        } else {
            Err(ParseError::Expected(Token::String("Id".to_string()), name).into())
        }
    }
}
