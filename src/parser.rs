use crate::lexer::{Keyword, Lexer, Token};
use ordered_float::NotNan;
use std::collections::HashMap;
use std::fmt;

#[derive(Debug)]
pub enum SyntaxError {
    ExpectedToken(Token, &'static str), // got `x`, expected `y` -- token is x, str shows y
    UnexpectedEOF,
}

#[derive(Debug)]
pub enum ParseTreeValue {
    Head,
    Statement,
    Pattern,
    Bullet,
    Path,
    Assignment,
    Wait,
    Seconds,
    Frames,
    For,
    Expression,
    RValue,
    Id(String),
    Num,
    Int(i64),
    Float(NotNan<f64>),
    FunctionCall,
    Range,
    ForCondition,
    Test,
    Bool,
    Expr,
    Term,
    Factor,
    ArithmeticExpression,
    Block,
    Args,
    ArgumentDefinition,
    ForDeclaration,
    ForBlock,
}

#[derive(Debug)]
pub struct ParseTreeNode {
    value: ParseTreeValue,
    table: HashMap<String, ParseTreeValue>,
    children: Vec<ParseTreeNode>,
}

impl fmt::Display for ParseTreeNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fn single_node_fmt(f: &mut fmt::Formatter, ident: usize, node: &ParseTreeNode) {
            let spaces = |num: usize| String::from(" ".repeat(num));
            let _ = write!(
                f,
                "{}value: {:?}{}table: {:?}\n",
                spaces(ident * 4),
                node.value,
                spaces(30),
                node.table
            );
            for child in &node.children {
                single_node_fmt(f, ident + 1, &child);
            }
        }
        single_node_fmt(f, 0, &self);
        write!(f, "")
    }
}

impl ParseTreeNode {
    pub fn new_head() -> ParseTreeNode {
        ParseTreeNode {
            value: ParseTreeValue::Head,
            table: HashMap::new(),
            children: Vec::new(),
        }
    }

    pub fn new_empty(value: ParseTreeValue) -> ParseTreeNode {
        ParseTreeNode {
            value,
            table: HashMap::new(),
            children: Vec::new(),
        }
    }

    pub fn new(
        value: ParseTreeValue,
        table: HashMap<String, ParseTreeValue>,
        children: Vec<ParseTreeNode>,
    ) -> ParseTreeNode {
        ParseTreeNode {
            value,
            table,
            children,
        }
    }
}

pub struct Parser {
    lexer: Lexer,
}

pub type ParseResult = Result<ParseTreeNode, SyntaxError>;

impl Parser {
    pub fn new(lexer: Lexer) -> Parser {
        Parser { lexer }
    }

    pub fn evaluate(&mut self) -> ParseResult {
        let mut head = ParseTreeNode::new_head();

        while let Some(token) = self.lexer.next_token() {
            if token != Token::EOF {
                let statement = self.parse_statements(token, &mut head)?;
                head.children.push(statement);
            } else {
                break;
            }
        }

        Ok(head)
    }

    fn unimplemented_result(&self) -> ParseResult {
        Ok(ParseTreeNode::new_head())
    }

    fn parse_statements(&mut self, t: Token, parent: &mut ParseTreeNode) -> ParseResult {
        if t == Token::Keyword(Keyword::Pattern) {
            return self.parse_pattern(parent);
        }

        Err(SyntaxError::ExpectedToken(
            t,
            "Unexpected token for toplevel definition.",
        ))
    }

    fn parse_pattern(&mut self, parent: &mut ParseTreeNode) -> ParseResult {
        let mut pattern_node =
            ParseTreeNode::new(ParseTreeValue::Pattern, HashMap::new(), Vec::new());

        let t = self.lexer.next_token().ok_or(SyntaxError::UnexpectedEOF)?;

        match t {
            Token::Id(t) => {
                parent.table.insert(t.clone(), ParseTreeValue::Pattern);

                let id_node = ParseTreeNode::new_empty(ParseTreeValue::Id(t));
                pattern_node.children.push(id_node);

                let t = self.lexer.next_token().ok_or(SyntaxError::UnexpectedEOF)?;
                let node = match t {
                    Token::Assign => self.parse_block(&mut pattern_node)?,
                    _ => {
                        return Err(SyntaxError::ExpectedToken(
                            t,
                            "Expected an `=` between pattern name and block.",
                        ))
                    }
                };
                pattern_node.children.push(node);
            }
            t => {
                return Err(SyntaxError::ExpectedToken(
                    t,
                    "Unexpected token for pattern ID.",
                ))
            }
        }

        Ok(pattern_node)
    }

    fn parse_block(&mut self, parent: &mut ParseTreeNode) -> ParseResult {
        let t = self.lexer.next_token();
        if t != Some(Token::OpenBlock) {
            return Err(SyntaxError::ExpectedToken(
                Token::OpenBlock,
                "Expected block",
            ));
        }

        let mut block_node = ParseTreeNode::new_empty(ParseTreeValue::Block);

        while let Some(t) = self.lexer.next_token() {
            let node = match t {
                Token::Id(id) => self.parse_assignment(&mut block_node)?,
                Token::Keyword(Keyword::Wait) => self.parse_wait(&mut block_node)?,
                Token::Keyword(Keyword::For) => self.parse_for(&mut block_node)?,
                Token::Keyword(Keyword::Spawn) => self.parse_spawn(&mut block_node)?,
                Token::CloseBlock => {
                    break;
                }
                _ => {
                    return Err(SyntaxError::ExpectedToken(
                        t,
                        "expected one of: wait, for, spawn, variable",
                    ));
                }
            };
            block_node.children.push(node);
        }

        Ok(block_node)
    }

    fn parse_wait(&mut self, parent: &mut ParseTreeNode) -> ParseResult {
        let t = self.lexer.next_token().ok_or(SyntaxError::UnexpectedEOF)?;
        // float -> time(seconds),
        // int   -> time(frames), time(seconds)
        let number_node = match t {
            Token::Number(num_string) => {
                
            }
            _ => {
                return Err(SyntaxError::ExpectedToken(t, "expected number after wait"))
            }
        }
    }

    fn parse_for(&mut self, parent: &mut ParseTreeNode) -> ParseResult {
        self.unimplemented_result()
    }
    fn parse_assignment(&mut self, parent: &mut ParseTreeNode) -> ParseResult {
        self.unimplemented_result()
    }
    fn parse_spawn(&mut self, parent: &mut ParseTreeNode) -> ParseResult {
        self.unimplemented_result()
    }
}
