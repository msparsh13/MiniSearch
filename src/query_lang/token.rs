#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Ident(String),
    Number(i64),
    And,
    Or,
    Not,
    Eq,
    Gt,
    Gte,
    Lt,
    Lte,
}
