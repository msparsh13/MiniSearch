use crate::index::b_tree::ValueTreeIndex;
use crate::index::documents_store;
use crate::index::inverted_index::{self, InvertedIndex};
use crate::index::n_gram_index::{self, NgramIndex};
use crate::index::n_gram_trie::{self, NgramTrie};
use crate::index::tokenizer::{self, Tokenizer, TokenizerConfig};
use ordered_float::OrderedFloat;
use std::collections::{BinaryHeap, HashMap, HashSet};

/**
 * We will need three inverted indexes:
 * 1. normal search
 * 2. fuzzy search -> done using hashmaps point to inverted index its bit slower but saves memory way more
 * 3. field-aware search: term -> doc_id -> [field_path] not needed any more
 * TODO :
 * Create fuzzy search for *abcd* will need edit distance and bm 25 match :fixed
 * for n gram make it efficient by getting intersection of words
 * id string
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

#[derive(Debug)]
pub struct DocumentStore {
    pub store: HashMap<String, Document>,
    tokenizer: Tokenizer,
    allow_ngram: bool,
    pub normal_index: InvertedIndex,
    pub n_gram_index: Option<NgramIndex>,
    pub n_gram_trie: Option<NgramTrie>,
    pub value_tree: ValueTreeIndex,
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
        self.extract_text(data, "", 0, max_depth, &mut texts);

        for (pos, (text, field_path)) in texts.iter().enumerate() {
            let (tokenized_words, tokenized_ngrams) =
                self.tokenizer.tokenize(text, self.allow_ngram);

            // 1️⃣ Index words
            for w in &tokenized_words {
                self.normal_index.add_term(w, doc_id, pos, &field_path);
            }

            // 2️⃣ Index n-grams (if enabled)
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
    fn extract_text(
        &mut self,
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
                Value::Number(n) => {
                    self.value_tree.add_index(&field_path, value, key);
                    output.push((n.to_string(), field_path))
                }
                Value::Date(d) => {
                    self.value_tree.add_index(&field_path, value, key);
                    output.push((d.clone(), field_path))
                }

                Value::Object(obj) => {
                    Self::extract_text(self, obj, &field_path, current_depth + 1, max_depth, output)
                }
            }
        }
    }

 pub fn ngram_bm25(
    &self,
    query: &str,
    k1: f64,
    b: f64,
    alpha: f64,
    beta: f64,
    top_k: usize,
) -> Vec<(String, f64)> {
    if !self.allow_ngram {
        return Vec::new();
    }

    // ----------------
    // Step 1: tokenize query
    let (tokenized_words, tokenized_ngrams) = self.tokenizer.tokenize(query, self.allow_ngram);

    // ----------------
    // Step 2: collect candidate words
    let mut word_counts: HashMap<&str, usize> = HashMap::new();
    let mut n_total = 0usize;

    if let Some(ref ngram_index) = self.n_gram_trie {
        for grams in &tokenized_ngrams {
            n_total += grams.len();
            for g in grams {
                for gr in &g.ngrams {
                    for term in ngram_index.get_terms(gr) {
                        *word_counts.entry(term).or_insert(0) += 1;
                    }
                }
            }
        }
    }

    // ----------------
    // Step 3: score candidates by n-gram overlap + edit distance
    let mut heap = BinaryHeap::new(); // (score, &str)

    let query_text = tokenized_words.join(" ");

    for (&word, &count) in &word_counts {
        let jaccard_score = count as f64 / n_total.max(1) as f64;

        let ed = self.edit_distance(&query_text, word);
        let edit_score = 1.0 - (ed as f64 / word.len().max(1) as f64);

        let candidate_score = alpha * jaccard_score + beta * edit_score;

        heap.push((OrderedFloat(candidate_score), word));
    }

    // ----------------
    // Step 4: select top-k
    let mut candidates: Vec<(&str, f64)> = heap
        .into_sorted_vec() // sort descending
        .into_iter()
        .rev()
        .take(top_k)
        .map(|(score, word)| (word, score.into_inner()))
        .collect();

    // ----------------
    // Step 5: run BM25 and combine
    let mut doc_scores: HashMap<String, f64> = HashMap::new();

    for (term, weight) in candidates.drain(..) {
        let doc_scores_map = self.normal_index.bm25_search(&[term], k1, b);

        for (doc_id, bm25_score) in doc_scores_map {
            *doc_scores.entry(doc_id).or_insert(0.0) += bm25_score * weight;
        }
    }

    let mut doc_scores_vec: Vec<(String, f64)> = doc_scores.into_iter().collect();
    doc_scores_vec.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    doc_scores_vec
}


    // finally using dp in protect lol
    fn edit_distance(&self, a: &str, b: &str) -> usize {
        let a_chars: Vec<char> = a.chars().collect();
        let b_chars: Vec<char> = b.chars().collect();
        let n = a_chars.len();
        let m = b_chars.len();

        // Create a (n+1) x (m+1) DP table
        let mut dp = vec![vec![0; m + 1]; n + 1];

        // Initialize base cases
        for i in 0..=n {
            dp[i][0] = i; // deletion
        }
        for j in 0..=m {
            dp[0][j] = j; // insertion
        }

        // Fill DP table
        for i in 1..=n {
            for j in 1..=m {
                let cost = if a_chars[i - 1] == b_chars[j - 1] {
                    0
                } else {
                    1
                };
                dp[i][j] = std::cmp::min(
                    std::cmp::min(
                        dp[i - 1][j] + 1, // deletion
                        dp[i][j - 1] + 1, // insertion
                    ),
                    dp[i - 1][j - 1] + cost, // substitution
                );
            }
        }

        dp[n][m]
    }

    pub fn range_query(&self, field_path: &str, min: i64, max: i64) -> Vec<(&String, &String)> {
        self.value_tree.range_query(field_path, min, max)
    }

    pub fn greater_than<'a>(&'a self, field_path: &str, min: i64) -> Vec<(&'a String, &'a String)> {
        let max_bound = i64::MAX;
        // > min → (min+1 ..= max_bound)
        self.value_tree.range_query(field_path, min + 1, max_bound)
    }

    pub fn greater_than_equal<'a>(
        &'a self,
        field_path: &str,
        min: i64,
    ) -> Vec<(&'a String, &'a String)> {
        let max_bound = i64::MAX;
        self.value_tree.range_query(field_path, min, max_bound)
    }

    pub fn less_than<'a>(&'a self, field_path: &str, max: i64) -> Vec<(&'a String, &'a String)> {
        let min_bound = i64::MIN;
        self.value_tree.range_query(field_path, min_bound, max - 1)
    }

    pub fn less_than_equal<'a>(
        &'a self,
        field_path: &str,
        max: i64,
    ) -> Vec<(&'a String, &'a String)> {
        let min_bound = i64::MIN;
        self.value_tree.range_query(field_path, min_bound, max)
    }

    pub fn between<'a>(
        &'a self,
        field_path: &str,
        min: i64,
        max: i64,
    ) -> Vec<(&'a String, &'a String)> {
        self.value_tree.range_query(field_path, min, max)
    }

    pub fn not_word(&self, word: Vec<&str>) -> Vec<String> {
        let excluded_ids: HashSet<String> =
            self.normal_index.search_term(&word).into_iter().collect();
        let mut result = Vec::with_capacity(self.store.len());

        for doc_id_str in self.store.keys() {
            match doc_id_str.parse::<String>() {
                Ok(doc_id) => {
                    if !excluded_ids.contains(&doc_id) {
                        result.push(doc_id_str.clone());
                    }
                }
                Err(_) => {
                    result.push(doc_id_str.clone());
                }
            }
        }

        result
    }

    pub fn get_words(&self, word: Vec<&str>) -> Vec<String> {
        let ids = self
            .normal_index
            .search_term(&word)
            .into_iter()
            .map(|id| id.to_string())
            .collect();
        ids
    }

       pub fn ngram_bm25_old(
        &self,
        query: &str,  // query tokens
        k1: f64,      // BM25 parameter
        b: f64,       // BM25 parameter
        alpha: f64,   // weight for n-gram similarity
        beta: f64,    // weight for edit distance
        top_k: usize, // number of candidate words to keep
    ) -> Vec<(String, f64)> {
        // return (doc_id, score)
        if !self.allow_ngram {
            return Vec::new();
        }

        // ----------------
        // Step 1: tokenize query into n-grams
        let (tokenized_words, tokenized_ngrams) = self.tokenizer.tokenize(query, self.allow_ngram);

        // ----------------
        // Step 2: collect candidate words
        let mut word_counts: HashMap<String, usize> = HashMap::new();
        let mut n_total = 0;

        if let Some(ref ngram_index) = self.n_gram_trie {
            for grams in tokenized_ngrams.iter() {
                for g in grams.iter() {
                    n_total += grams.len();
                    for gr in g.ngrams.clone() {
                        let terms = ngram_index.get_terms(&gr);
                        for t in terms {
                            *word_counts.entry(t.to_owned()).or_insert(0) += 1;
                        }
                    }
                }
            }
        }

        // ----------------
        // Step 3: score candidates by n-gram overlap + edit distance
        let mut heap = BinaryHeap::new(); // max-heap (score, word)

        for (word, count) in word_counts {
            let jaccard_score = (count as f64) / (n_total as f64);

            let ed = self.edit_distance(&(tokenized_words.join(" ")), &word);
            let edit_score = 1.0 - (ed as f64 / word.len().max(1) as f64);

            let candidate_score = alpha * jaccard_score + beta * edit_score;

            heap.push((OrderedFloat(candidate_score), word));
        }

        // ----------------
        // Step 4: select top-k candidate words
        let mut candidates = Vec::new();
        for _ in 0..top_k {
            if let Some((score, word)) = heap.pop() {
                candidates.push((word, score.into_inner()));
            }
        }

        // ----------------
        // Step 5: run BM25 for candidate words
        let mut doc_scores = HashMap::new();

        for (term, weight) in candidates {
            let doc_scores_map = self.normal_index.bm25_search(&[&term], k1, b);

            for (doc_id, bm25_score) in doc_scores_map {
                let weighted_score = bm25_score * weight;

                // Aggregate scores for the same doc_id
                *doc_scores.entry(doc_id).or_insert(0.0) += weighted_score;
            }
        }
        let mut doc_scores_vec: Vec<(String, f64)> = doc_scores.into_iter().collect();
        doc_scores_vec.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        doc_scores_vec
    }


}