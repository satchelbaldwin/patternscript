use crate::lexer::{ConditionToken, Keyword, Lexer, Token};
use crate::types::Op;
use anyhow::{Context, Result};
use std::collections::HashMap;
//use std::fmt;
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
    #[error("Bad vector element.")]
    BadVecElement,
    #[error("{0}")]
    NeedsClearerError(&'static str),
}

#[derive(Debug)]
pub struct HeadData {
    definitions: HashMap<String, Node>,
}

type Values = HashMap<String, ExpressionType>;

#[derive(Debug)]
pub enum ArithmeticExpression {
    Unary(UnaryOperator, Box<ExpressionType>),
    Binary(Op, Box<ExpressionType>, Box<ExpressionType>),
}

#[derive(Debug)]
pub enum UnaryOperator {
    Negate,
    FunctionCall(String), // Unary(FunctionCall(name of function), (boxed args vec: see above enum))
}

#[derive(Debug)]
pub enum ExpressionType {
    Int(i64),
    Float(f64),
    String(String),
    Range(i64, i64),
    Block(Block),
    Variable(String),
    Duration(Box<WaitData>),
    Vector(Vec<ExpressionType>),
    Expr(ArithmeticExpression),
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
    Frames(ExpressionType),
    Time(ExpressionType),
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
    // Expression(ExpressionType),
    Spawn(SpawnData),
}

/*impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fn fmt_single(n: &Node, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "test")
        }
        fmt_single(&self, f)
    }
}*/

type NamedToplevel = (String, Node);

pub struct Parser {
    lexer: Lexer,
}

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
    //
    // note: excellent article https://www.engr.mun.ca/~theo/Misc/exp_parsing.htm with details about
    //       three approaches to this problem, cited papers are also extremely helpful in the
    //       bibliography of the linked post.
    pub fn parse_expression(&mut self) -> Result<ExpressionType> {
        let expr = self.parse_expression_r();
        // special case for pseudo-datatypes
        match self.lookahead(1)? {
            Token::Keyword(Keyword::Seconds) => {
                self.next_token()?;
                self.expect_next(Token::Semicolon)?;
                Ok(ExpressionType::Duration(Box::new(WaitData::Time(expr?))))
            }
            Token::Keyword(Keyword::Frames) => {
                self.next_token()?;
                self.expect_next(Token::Semicolon)?;
                Ok(ExpressionType::Duration(Box::new(WaitData::Frames(expr?))))
            }
            Token::Semicolon => {
                self.next_token()?;
                expr
            }
            Token::CloseBlock => {
                // leave token to be consumed by parse_block
                expr
            }
            Token::Assign => {
                // leave token to be consumed by parse_path
                expr
            }
            Token::CloseParen | Token::Comma => {
                // leave to be consumed by op_or_vec
                expr
            }
            x => {
                println!("{:?}", x);
                return Err(
                    ParseError::NeedsClearerError("Expressions should end in } or ;.").into(),
                );
            }
        }
    }
    // inner recursive loop in the case of vectors
    pub fn parse_expression_r(&mut self) -> Result<ExpressionType> {
        let t = self.lookahead(1)?;
        match t {
            Token::OpenBlock => {
                // self.next_token()?;
                return Ok(ExpressionType::Block(self.parse_block()?));
            }
            Token::String(s) => {
                self.next_token()?;
                return Ok(ExpressionType::String(s));
            }
            _ => {
                return self.parse_expression_p(0);
            }
        }
    }
    // precedence handling -- the meat
    pub fn parse_expression_p(&mut self, precedence: u32) -> Result<ExpressionType> {
        // initially set tree to first value -- we'll move ownership at each step
        let mut tree = self.parse_operator_or_value()?;
        loop {
            let next = self.lookahead(1)?;
            match next {
                Token::Operator(op) => {
                    let p = self.operator_precedence(&op);
                    if p < precedence {
                        break;
                    }
                    // invariant: next is a binary operator with precedence higher than fn call
                    self.next_token()?;
                    // left assoc: p + 1, right assoc: p
                    let new_precedence = match op {
                        Op::Exp => p + 1,
                        _ => p,
                    };
                    let rhs = self.parse_expression_p(new_precedence)?;
                    tree = ExpressionType::Expr(ArithmeticExpression::Binary(
                        op,
                        Box::new(tree),
                        Box::new(rhs),
                    ));
                }
                _ => break,
            }
        }
        Ok(tree)
    }

    pub fn operator_precedence(&self, op: &Op) -> u32 {
        // precedence:
        //   0?: PAREN/VECTOR
        //   1L: OR
        //   2L: AND
        //   2L: ==
        //   3L: +-
        //   4L: - (UNARY)
        //   5L: */
        //   6R: ^
        match op {
            Op::Or => 0,
            Op::And => 1,
            Op::Test => 2,
            // unary minus: => 4, somewhere else
            Op::Add | Op::Sub => 3,
            Op::Div | Op::Mul => 5,
            Op::Exp => 6,
        }
    }

    pub fn parse_operator_or_value(&mut self) -> Result<ExpressionType> {
        // lookahead then consume on branch

        let mut t = self.lookahead(1)?;
        if matches!(t, Token::Operator(Op::Sub)) {
            // unary - precedence 4 left assoc
            self.next_token()?;
            let expr = self.parse_expression_p(4)?;
            Ok(ExpressionType::Expr(ArithmeticExpression::Unary(
                UnaryOperator::Negate,
                Box::new(expr),
            )))
        } else if t == Token::OpenParen {
            // in the outermost parenthesis loop, are we a vector?
            // (1, 2)       -- yes
            // ((x + 1), 2) -- yes
            // ((x + 1)- 2) -- no
            let mut found_vector = false;
            let mut nested_paren_level = 0;
            let mut lookahead_n = 1; // t (open paren), we start at next
            while !(t == Token::CloseParen && nested_paren_level < 1) {
                t = self.lookahead(lookahead_n)?;
                if t == Token::OpenParen {
                    nested_paren_level += 1;
                }
                if t == Token::CloseParen {
                    nested_paren_level -= 1;
                }
                if t == Token::Comma && nested_paren_level == 1 {
                    found_vector = true;
                }
                lookahead_n += 1;
            }

            // we are dealing with a vector
            if found_vector {
                let mut v: Vec<ExpressionType> = Vec::new();
                // consume open paren
                self.expect_next(Token::OpenParen)?;
                loop {
                    v.push(self.parse_expression_r()?);
                    match self.next_token()? {
                        Token::Comma => {
                            continue;
                        }
                        Token::CloseParen => {
                            return Ok(ExpressionType::Vector(v));
                        }
                        _ => return Err(ParseError::BadVecElement.into()),
                    }
                }
            } else {
                self.next_token()?;
                let r = self.parse_expression_r();
                self.expect_next(Token::CloseParen)?;
                r
            }
        } else {
            // could be value or fn call; hard to discern what a floating id means
            // right now string is handled above for performance? that might change
            // not sure
            match t {
                // value
                Token::Number(_n) => self.parse_number(),
                // might be value? could also be fn call here
                Token::Id(id) => {
                    //lookahead next run parse r as function call -- we have not consumed, so look 2
                    let next_t = self.lookahead(2)?;
                    if next_t == Token::OpenParen {
                        // consume singular so we know we have ( on deck
                        self.next_token()?;
                        let expr = self.parse_expression_r()?;
                        // hopefully a vector? should be or we crash probs
                        Ok(ExpressionType::Expr(ArithmeticExpression::Unary(
                            UnaryOperator::FunctionCall(id),
                            Box::new(expr),
                        )))
                    } else {
                        self.next_token()?;
                        Ok(ExpressionType::Variable(id))
                    }
                }
                // can this ever be reached?
                x => {
                    return Err(ParseError::NeedsClearerError(
                        "Expected value within operator parsing",
                    )
                    .into());
                }
            }
        }
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
            self.expect_next(Token::CloseParen)?;
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
        let expr = self.parse_expression()?;
        match expr {
            ExpressionType::Duration(wd) => Ok(*wd),
            _err => Err(ParseError::NeedsClearerError("Found invalid time expression").into()),
        }
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
            let arguments = self.parse_expression()?;
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
