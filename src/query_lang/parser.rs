use crate::query_lang::{
    ast::{CmpOp, Expr, SortOrder, Value},
    token::Token,
};

#[derive(Debug)]
pub enum ParseError {
    UnexpectedToken {
        expected: String,
        found: Option<Token>,
    },
    UnexpectedEof,
}

type ParseResult<T> = Result<T, ParseError>;

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    /* ------------------ helpers ------------------ */

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn advance(&mut self) {
        self.pos += 1;
    }

    fn consume(&mut self, expected: &Token) -> bool {
        if self.peek() == Some(expected) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn expect_ident(&mut self) -> ParseResult<String> {
        match self.peek() {
            Some(Token::Ident(name)) => {
                let name = name.clone();
                self.advance();
                Ok(name)
            }
            found => Err(ParseError::UnexpectedToken {
                expected: "identifier".into(),
                found: found.cloned(),
            }),
        }
    }

    fn expect_op(&mut self) -> ParseResult<CmpOp> {
        let op = match self.peek() {
            Some(Token::Eq) => CmpOp::Eq,
            Some(Token::Gt) => CmpOp::Gt,
            Some(Token::Gte) => CmpOp::Gte,
            Some(Token::Lt) => CmpOp::Lt,
            Some(Token::Lte) => CmpOp::Lte,
            found => {
                return Err(ParseError::UnexpectedToken {
                    expected: "comparison operator".into(),
                    found: found.cloned(),
                });
            }
        };
        self.advance();
        Ok(op)
    }

    fn expect_value(&mut self) -> ParseResult<Value> {
        match self.peek() {
            Some(Token::Number(n)) => {
                let v = Value::Number(*n);
                self.advance();
                Ok(v)
            }
            Some(Token::Ident(s)) => {
                let v = Value::Text(s.clone());
                self.advance();
                Ok(v)
            }
            found => Err(ParseError::UnexpectedToken {
                expected: "value".into(),
                found: found.cloned(),
            }),
        }
    }

    /* ------------------ entry ------------------ */

    pub fn parse(mut self) -> ParseResult<Expr> {
        // COUNT prefix support
        let mut expr = if self.consume(&Token::Cnt) {
            let inner = self.parse_or()?;
            Expr::Count(Box::new(inner))
        } else {
            self.parse_or()?
        };

        // Optional SORT
        if self.consume(&Token::Asc) {
            let field = self.expect_ident()?;
            expr = Expr::Sort {
                expr: Box::new(expr),
                field,
                order: SortOrder::Asc,
            };
        } else if self.consume(&Token::Desc) {
            let field = self.expect_ident()?;
            expr = Expr::Sort {
                expr: Box::new(expr),
                field,
                order: SortOrder::Desc,
            };
        }

        // Ensure no trailing tokens
        if self.pos != self.tokens.len() {
            return Err(ParseError::UnexpectedToken {
                expected: "end of input".into(),
                found: self.peek().cloned(),
            });
        }

        Ok(expr)
    }

    /* ------------------ grammar ------------------ */
    // expr  := or
    // or    := and (OR and)*
    // and   := not (AND not)*
    // not   := NOT not | primary
    // primary := comparison | '(' expr ')'

    fn parse_or(&mut self) -> ParseResult<Expr> {
        let mut left = self.parse_and()?;

        while self.consume(&Token::Or) {
            let right = self.parse_and()?;
            left = Expr::Or(Box::new(left), Box::new(right));
        }

        Ok(left)
    }

    fn parse_and(&mut self) -> ParseResult<Expr> {
        let mut left = self.parse_not()?;

        while self.consume(&Token::And) {
            let right = self.parse_not()?;
            left = Expr::And(Box::new(left), Box::new(right));
        }

        Ok(left)
    }

    fn parse_not(&mut self) -> ParseResult<Expr> {
        if self.consume(&Token::Not) {
            Ok(Expr::Not(Box::new(self.parse_not()?)))
        } else {
            self.parse_primary()
        }
    }

    fn parse_primary(&mut self) -> ParseResult<Expr> {
        if self.consume(&Token::LParen) {
            let expr = self.parse_or()?;

            if !self.consume(&Token::RParen) {
                return Err(ParseError::UnexpectedToken {
                    expected: ")".into(),
                    found: self.peek().cloned(),
                });
            }

            Ok(expr)
        } else {
            self.parse_comparison()
        }
    }

    fn parse_comparison(&mut self) -> ParseResult<Expr> {
        let field = self.expect_ident()?;
        let op = self.expect_op()?;
        let value = self.expect_value()?;

        Ok(Expr::Compare { field, op, value })
    }
}
