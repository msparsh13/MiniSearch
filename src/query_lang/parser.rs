use crate::query_lang::{
    ast::{CmpOp, Expr, Value},
    token::Token,
};

pub struct Parser {
    pub tokens: Vec<Token>,
    pub pos: usize,
}

impl Parser {
    fn match_token(&mut self, expected: Token) -> bool {
        if self.pos < self.tokens.len() && self.tokens[self.pos] == expected {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    fn expect_ident(&mut self) -> String {
        match self.tokens.get(self.pos) {
            Some(Token::Ident(name)) => {
                self.pos += 1;
                name.clone()
            }
            _ => panic!("expected identifier"),
        }
    }

    fn expect_op(&mut self) -> CmpOp {
        let op = match self.tokens.get(self.pos) {
            Some(Token::Eq) => CmpOp::Eq,
            Some(Token::Gt) => CmpOp::Gt,
            Some(Token::Gte) => CmpOp::Gte,
            Some(Token::Lt) => CmpOp::Lt,
            Some(Token::Lte) => CmpOp::Lte,
            _ => panic!("expected comparison operator"),
        };
        self.pos += 1;
        op
    }

    fn expect_value(&mut self) -> Value {
        match self.tokens.get(self.pos) {
            Some(Token::Number(n)) => {
                self.pos += 1;
                Value::Number(*n)
            }
            Some(Token::Ident(s)) => {
                self.pos += 1;
                Value::Text(s.clone())
            }
            _ => panic!("expected value"),
        }
    }

    pub fn parse(mut self) -> Expr {
        self.parse_or()
    }

    fn parse_or(&mut self) -> Expr {
        let mut left = self.parse_and();

        while self.match_token(Token::Or) {
            let right = self.parse_and();
            left = Expr::Or(Box::new(left), Box::new(right));
        }

        left
    }

    fn parse_and(&mut self) -> Expr {
        let mut left = self.parse_not();

        while self.match_token(Token::And) {
            let right = self.parse_not();
            left = Expr::And(Box::new(left), Box::new(right));
        }

        left
    }

    fn parse_not(&mut self) -> Expr {
        if self.match_token(Token::Not) {
            Expr::Not(Box::new(self.parse_not()))
        } else {
            self.parse_cmp()
        }
    }

    fn parse_cmp(&mut self) -> Expr {
        let field = self.expect_ident();
        let op = self.expect_op();
        let value = self.expect_value();

        Expr::Compare { field, op, value }
    }
}
