use crate::lexer::{Keyword, Lexer, Token};
use std::collections::HashMap;

pub enum SyntaxError {
    UnexpectedToken(Token),      // did not expect `x`
    ExpectedToken(Token, Token), // got `x`, expected `y`
}

pub enum ParseTreeValue {
    Head,
    Statement,
    Pattern,
    Bullet,
    Path,
    Assignment,
    Wait,
    For,
    Expression,
    RValue,
    Id,
    Num,
    Int,
    Float,
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

pub struct ParseTreeNode {
    value: ParseTreeValue,
    table: HashMap<String, ParseTreeValue>,
    children: Vec<ParseTreeNode>,
}

impl ParseTreeNode {
    pub fn new_head() -> ParseTreeNode {
        ParseTreeNode {
            value: ParseTreeValue::Head,
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
    pub fn evaluate(&mut self) -> ParseResult {
        let mut head = ParseTreeNode::new_head();

        while let Some(token) = self.lexer.next_token() {
            match self.parse_statements(token) {
                Ok(node) => head.children.push(node),
                Err(e) => {
                    return Err(e);
                }
            }
        }

        Ok(head)
    }

    fn parse_statements(&mut self, t: Token) -> ParseResult {
        if t == Token::Keyword(Keyword::Pattern) {
            return self.parse_pattern();
        }

        Err(SyntaxError::UnexpectedToken(t))
    }

    fn parse_pattern(&mut self) -> ParseResult {
        let mut pattern_node =
            ParseTreeNode::new(ParseTreeValue::Pattern, HashMap::new(), Vec::new());
        let t = self.lexer.next_token();
        match t {
            Token::String(t) => {}
            _ => Err,
        }
    }
}
