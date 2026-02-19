#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Ident(String),
    Number(i64),

    // comparison
    Eq,
    Gt,
    Gte,
    Lt,
    Lte,

    // boolean
    And,
    Or,
    Not,

    // grouping
    LParen, // (
    RParen, // )

    EOF,
}
