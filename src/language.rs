pub mod language {

    use crate::engine::query_service::QueryService;
    use crate::query_lang::ast::Expr;
    use crate::query_lang::exec::execute;
    use crate::query_lang::parser::Parser;
    use crate::query_lang::token::Token;
    pub fn run_query(query: &str, qs: &QueryService) -> std::collections::HashSet<String> {
        let tokens = tokenize(query);
        let mut parser = Parser { tokens, pos: 0 };
        let expr: Expr = parser.parse();
        execute(&expr, qs)
    }

    pub fn tokenize(input: &str) -> Vec<Token> {
        input
            .split_whitespace()
            .map(|s| match s {
                "AND" => Token::And,
                "OR" => Token::Or,
                "NOT" => Token::Not,
                ">=" => Token::Gte,
                ">" => Token::Gt,
                "<=" => Token::Lte,
                "<" => Token::Lt,
                "=" => Token::Eq,
                s if s.parse::<i64>().is_ok() => Token::Number(s.parse::<i64>().unwrap()),
                s => Token::Ident(s.to_string()),
            })
            .collect()
    }
}
