pub mod language {

    use crate::engine::query_service::QueryService;
    use crate::query_lang::ast::Expr;
    use crate::query_lang::exec::execute;
    use crate::query_lang::parser::Parser;
    use crate::query_lang::token::Token;
    pub fn run_query(
        query: &str,
        qs: &QueryService,
    ) -> Result<std::collections::HashSet<String>, String> {
        let tokens = tokenize(query);

        let parser = Parser::new(tokens);
        let expr = parser
            .parse()
            .map_err(|e| format!("Query parse error: {:?}", e))?;

        Ok(execute(&expr, qs))
    }

    pub fn tokenize(input: &str) -> Vec<Token> {
        let mut tokens = Vec::new();
        let mut chars = input.chars().peekable();

        while let Some(c) = chars.next() {
            match c {
                // skip whitespace
                c if c.is_whitespace() => {}

                // parentheses
                '(' => tokens.push(Token::LParen),
                ')' => tokens.push(Token::RParen),

                // operators
                '=' => tokens.push(Token::Eq),
                '>' => {
                    if chars.peek() == Some(&'=') {
                        chars.next();
                        tokens.push(Token::Gte);
                    } else {
                        tokens.push(Token::Gt);
                    }
                }
                '<' => {
                    if chars.peek() == Some(&'=') {
                        chars.next();
                        tokens.push(Token::Lte);
                    } else {
                        tokens.push(Token::Lt);
                    }
                }

                // numbers
                c if c.is_ascii_digit() => {
                    let mut num = c.to_string();
                    while let Some(n) = chars.peek() {
                        if n.is_ascii_digit() {
                            num.push(chars.next().unwrap());
                        } else {
                            break;
                        }
                    }
                    tokens.push(Token::Number(num.parse().unwrap()));
                }

                // identifiers / keywords
                c if c.is_ascii_alphabetic() || c == '_' => {
                    let mut ident = c.to_string();
                    while let Some(ch) = chars.peek() {
                        if ch.is_ascii_alphanumeric() || *ch == '_' {
                            ident.push(chars.next().unwrap());
                        } else {
                            break;
                        }
                    }

                    match ident.as_str() {
                        "AND" => tokens.push(Token::And),
                        "OR" => tokens.push(Token::Or),
                        "NOT" => tokens.push(Token::Not),
                        _ => tokens.push(Token::Ident(ident)),
                    }
                }

                _ => {
                    panic!("unexpected character: {}", c);
                }
            }
        }

        tokens
    }
}
