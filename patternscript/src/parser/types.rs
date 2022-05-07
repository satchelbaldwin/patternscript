//
// shared types between parser and lexer
//

// binary operators
#[derive(Debug, PartialEq, Hash, Clone)]
pub enum Op {
    Test,
    And,
    Or,
    Add,
    Sub,
    Mul,
    Div,
    Exp,
    GT,
    LT,
    GTE,
    LTE,
}
