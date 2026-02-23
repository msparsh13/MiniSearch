use std::collections::HashSet;

use crate::{
    engine::query_service::{QueryService, SortField},
    query_lang::ast::{CmpOp, Expr, SortOrder, Value},
};

/// Convert Vec<(&String, &String)> → HashSet<String>
fn ids_from_pairs(pairs: Vec<(&String, &String)>) -> HashSet<String> {
    pairs.into_iter().map(|(id, _)| id.clone()).collect()
}

pub fn execute(expr: &Expr, qs: &QueryService) -> Vec<String> {
    match expr {
        //-------------------------------
        //Aggregation and sorting
        //-------------------------------
        Expr::Count(inner) => {
            let result = execute(inner, qs);
            println!("Count: {}", result.len());
            result
        }
        Expr::Sort { expr, fields } => {
            let result_set = execute(expr, qs);
            let sort_fields: Vec<SortField> = fields
                .iter()
                .map(|(field, order)| SortField {
                    field_path: field.clone(),
                    ascending: matches!(order, SortOrder::Asc),
                })
                .collect();

            let sorted = qs.sort_docs_2(result_set.into_iter().collect(), &sort_fields);

            sorted.into_iter().collect()
        }
        // ------------------------------
        // Comparisons
        // ------------------------------
        Expr::Compare { field, op, value } => match (op, value) {
            // text equality
            (CmpOp::Eq, Value::Text(v)) => qs.get_words(vec![v.as_str()]).into_iter().collect(),

            // numeric comparisons
            (CmpOp::Gt, Value::Number(n)) => ids_from_pairs(qs.greater_than(field, *n, None)).into_iter()
    .collect::<Vec<String>>(),
            (CmpOp::Gte, Value::Number(n)) => {
                ids_from_pairs(qs.greater_than_equal(field, *n, None)).into_iter()
    .collect::<Vec<String>>()
            }
            (CmpOp::Lt, Value::Number(n)) => ids_from_pairs(qs.less_than(field, *n, None)).into_iter()
    .collect::<Vec<String>>(),
            (CmpOp::Lte, Value::Number(n)) => ids_from_pairs(qs.less_than_equal(field, *n, None)).into_iter()
    .collect::<Vec<String>>(),

            _ => panic!("invalid comparison"),
        },

        // ------------------------------
        // AND
        // ------------------------------
        Expr::And(a, b) => {
            let mut left = execute(a, qs);
            let right = execute(b, qs);
            left.retain(|id| right.contains(id));
            left
        }

        // ------------------------------
        // OR
        // ------------------------------
        Expr::Or(a, b) => {
            let mut left = execute(a, qs);
            left.extend(execute(b, qs));
            left
        }

        // ------------------------------
        // NOT
        // ------------------------------
        Expr::Not(e) => {
            // parser guarantees NOT applies to a single term
            // engine guarantees correctness & fallback
            match &**e {
                Expr::Compare {
                    op: CmpOp::Eq,
                    value: Value::Text(word),
                    ..
                } => qs.not_word([word.as_str()].to_vec()).into_iter().collect(),

                _ => panic!("NOT only supported on term expressions"),
            }
        }
    }
}
