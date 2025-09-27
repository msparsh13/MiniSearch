use crate::index::inverted_index::InvertedIndex;
use crate::index::tokenizer::{Tokenizer, TokenizerConfig};
use std::collections::HashMap;

/**
 * We will need three inverted indexes:
 * 1. normal search
 * 2. fuzzy search
 * 3. field-aware search: term -> doc_id -> [field_path]
 */

#[derive(Debug, Clone)]
pub enum Value {
    Text(String),
    Number(f64),
    Date(String),
    Object(HashMap<String, Value>),
}

#[derive(Debug, Clone)]
pub struct Document {
    pub id: String,
    pub data: HashMap<String, Value>,
}

pub struct DocumentStore {
    pub store: HashMap<String, Document>,
    tokenizer: Tokenizer,
    allow_ngram: bool,
    pub normal_index: InvertedIndex,
    pub n_gram_index: Option<InvertedIndex>,
}

impl DocumentStore {
    pub fn new(config: Option<TokenizerConfig>) -> Self {
        // Determine tokenizer config
        let tokenizer_config = config.unwrap_or_default();

        // Determine allow_ngram: true if min_ngram or max_ngram is Some
        let allow_ngram =
            tokenizer_config.min_ngram.is_some() || tokenizer_config.max_ngram.is_some();

        Self {
            store: HashMap::new(),
            allow_ngram,
            tokenizer: Tokenizer::new(tokenizer_config),
            normal_index: InvertedIndex::new(),
            n_gram_index: if allow_ngram {
                Some(InvertedIndex::new())
            } else {
                None
            },
        }
    }

    pub fn add_document(
        &mut self,
        mut data: HashMap<String, Value>,
        max_depth: Option<usize>,
    ) -> String {
        let id = format!("{}", self.store.len() + 1);
        let max_depth = max_depth.unwrap_or(1);

        for (_, value) in data.iter_mut() {
            Self::normalize_value(value, max_depth);
        }

        let doc = Document {
            id: id.clone(),
            data,
        };
        self.store.insert(id.clone(), doc);
        id
    }

    pub fn get_document(&self, id: &str) -> Option<&Document> {
        self.store.get(id)
    }

    fn normalize_value(value: &mut Value, max_depth: usize) {
        Self::normalize_value_rec(value, 0, max_depth);
    }

    fn normalize_value_rec(value: &mut Value, current_depth: usize, max_depth: usize) {
        if current_depth >= max_depth {
            return;
        }

        match value {
            Value::Text(t) => {
                *t = t.to_lowercase().trim().to_string();
            }
            Value::Number(_) => { /* nothing to do */ }
            Value::Date(s) => {
                *s = s.trim().to_string(); // can normalize date format if needed
            }
            Value::Object(obj) => {
                // print!("{:?}",obj);
                for (_, v) in obj.iter_mut() {
                    Self::normalize_value_rec(v, current_depth + 1, max_depth);
                }
            }
        }
    }

    pub fn index_document(
        &mut self,
        doc_id: &str,
        data: &HashMap<String, Value>,
        max_depth: usize,
    ) {
        let mut texts = Vec::new();
        Self::extract_text(data, "", 0, max_depth, &mut texts);

        for (pos, (text, field_path)) in texts.iter().enumerate() {
            let (words, ngrams) = self.tokenizer.tokenize(text, self.allow_ngram);

            for w in &words {
                self.normal_index
                    .add_term(w, doc_id.parse::<usize>().unwrap(), pos, &field_path);
            }

            if let Some(ref mut n_index) = self.n_gram_index {
                for gram in &ngrams {
                    for g in gram {
                        n_index.add_term(g, doc_id.parse::<usize>().unwrap(), pos, &field_path);
                    }
                }
            }
        }
    }

    // Recursively extract text for indexing
    fn extract_text(
        data: &HashMap<String, Value>,
        prefix: &str,
        current_depth: usize,
        max_depth: usize,
        output: &mut Vec<(String, String)>,
    ) {
        if current_depth > max_depth {
            return;
        }

        for (key, value) in data.iter() {
            let field_path = if prefix.is_empty() {
                key.clone()
            } else {
                format!("{}.{}", prefix, key)
            };
            output.push((key.clone(), field_path.clone()));
            match value {
                Value::Text(t) => output.push((t.clone(), field_path)),
                Value::Number(n) => output.push((n.to_string(), field_path)),
                Value::Date(d) => output.push((d.clone(), field_path)),
                Value::Object(obj) => {
                    Self::extract_text(obj, &field_path, current_depth + 1, max_depth, output)
                }
            }
        }
    }
}
