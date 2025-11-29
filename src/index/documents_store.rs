use crate::index::documents_store;
use crate::index::forward_indexer::{ForwardDoc, ForwardIndex};

use crate::index::inverted_index::inverted_index::InvertedIndex;

use crate::index::n_gram::n_gram_index::NgramIndex;
use crate::index::n_gram::n_gram_trie::NgramTrie;
use crate::index::tokenizer::tokenizer::{Tokenizer, TokenizerConfig};
use crate::index::value::Value;
use crate::index::value_tree::b_tree::ValueTreeIndex;
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};
use std::collections::{BinaryHeap, HashMap, HashSet};

/**
 * We will need three inverted indexes:
 * 1. normal search
 * 2. fuzzy search -> done using hashmaps point to inverted index its bit slower but saves memory way more
 * 3. field-aware search: term -> doc_id -> [field_path] not needed any more
 * TODO :
 * Create fuzzy search for *abcd* will need edit distance and bm 25 match :fixed
 * for n gram make it efficient by getting intersection of words :fixed
 * id string :fixed
 */

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: String,
    pub data: HashMap<String, Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DocumentStore {
    pub store: HashMap<String, Document>,
    #[serde(skip)]
    pub tokenizer: Tokenizer,
    pub allow_ngram: bool,
    pub normal_index: InvertedIndex,
    pub n_gram_index: Option<NgramIndex>,
    pub n_gram_trie: Option<NgramTrie>,
    pub value_tree: ValueTreeIndex,
    pub forward_index: ForwardIndex,
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
                Some(NgramIndex::new())
            } else {
                None
            },
            n_gram_trie: if allow_ngram {
                Some(NgramTrie::new())
            } else {
                None
            },
            value_tree: ValueTreeIndex::new(),
            forward_index: ForwardIndex {
                docs: HashMap::new(),
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
            data: data.clone(),
        };
        self.store.insert(id.clone(), doc);
        self.index_document(&id, &data, max_depth);
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
        let mut forwards = ForwardDoc::new();
        print!("{:?}", forwards);
        self.extract_text(data, "", 0, max_depth, &mut texts, &mut forwards);
        self.forward_index.add_doc(doc_id, forwards);
        for (pos, (text, field_path)) in texts.iter().enumerate() {
            let (tokenized_words, tokenized_ngrams) =
                self.tokenizer.tokenize(text, self.allow_ngram);

            for w in &tokenized_words {
                self.normal_index.add_term(w, doc_id, pos, &field_path);
            }

            if let Some(ref word_ngrams) = tokenized_ngrams {
                if let Some(ref mut n_index) = self.n_gram_trie {
                    for wn in word_ngrams {
                        for gram in &wn.ngrams {
                            n_index.insert(gram, &wn.word);
                        }
                    }
                }
            }
        }
    }

    // Recursively extract text for indexing
    // fn extract_text(
    //     &mut self,
    //     data: &HashMap<String, Value>,
    //     prefix: &str,
    //     current_depth: usize,
    //     max_depth: usize,
    //     output: &mut Vec<(String, String)>,
    // ) {
    //     if current_depth > max_depth {
    //         return;
    //     }

    //     for (key, value) in data.iter() {
    //         let field_path = if prefix.is_empty() {
    //             key.clone()
    //         } else {
    //             format!("{}.{}", prefix, key)
    //         };
    //         output.push((key.clone(), field_path.clone()));
    //         match value {
    //             Value::Text(t) => output.push((t.clone(), field_path)),
    //             Value::Number(n) => {
    //                 self.value_tree.add_index(&field_path, value, key);
    //                 output.push((n.to_string(), field_path))
    //             }
    //             Value::Date(d) => {
    //                 self.value_tree.add_index(&field_path, value, key);
    //                 output.push((d.clone(), field_path))
    //             }

    //             Value::Object(obj) => {
    //                 Self::extract_text(self, obj, &field_path, current_depth + 1, max_depth, output)
    //             }
    //         }
    //     }
    // }
    fn extract_text(
        &mut self,
        data: &HashMap<String, Value>,
        prefix: &str,
        current_depth: usize,
        max_depth: usize,
        out_terms: &mut Vec<(String, String)>,
        forward: &mut ForwardDoc,
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

            match value {
                Value::Text(t) => {
                    // store into forward doc
                    forward.text_fields.insert(field_path.clone(), t.clone());

                    // also push to inverted index extraction
                    out_terms.push((t.clone(), field_path));
                }

                Value::Number(n) => {
                    forward.numeric_fields.insert(field_path.clone(), *n);

                    self.value_tree.add_index(&field_path, value, key);
                    out_terms.push((n.to_string(), field_path));
                }

                Value::Date(d) => {
                    forward.date_fields.insert(field_path.clone(), d.clone());

                    self.value_tree.add_index(&field_path, value, key);
                    out_terms.push((d.clone(), field_path));
                }

                Value::Object(obj) => {
                    Self::extract_text(
                        self,
                        obj,
                        &field_path,
                        current_depth + 1,
                        max_depth,
                        out_terms,
                        forward,
                    );
                }
            }
        }
    }

    pub fn delete_index(&mut self, doc_id: &str) {
        // 1️⃣ Get forward document
        let Some(forward_doc) = self.forward_index.get(doc_id).cloned() else {
            return; // nothing to delete
        };

        for (field_path, text_value) in forward_doc.text_fields {
            let (words, ngrams_opt) = self.tokenizer.tokenize(&text_value, self.allow_ngram);

            for w in &words {
                self.normal_index.remove_by_id(doc_id);
            }

            if let Some(ref mut trie) = self.n_gram_trie {
                if let Some(ngrams_list) = ngrams_opt {
                    for word_grams in ngrams_list {
                        for gram in word_grams.ngrams {
                            trie.remove_word(&gram, &word_grams.word);
                        }
                    }
                }
            }
        }

        for (field_path, num_value) in &forward_doc.numeric_fields {
            self.value_tree
                .remove_index(field_path, &Value::Number(*num_value), doc_id);
        }

        for (field_path, date_value) in forward_doc.date_fields {
            self.value_tree
                .remove_index(&field_path, &Value::Date(date_value.to_string()), doc_id);
        }
        self.forward_index.remove(doc_id);
        self.store.remove(doc_id);
    }
}
