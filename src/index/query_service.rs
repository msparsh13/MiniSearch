use std::{collections::{BinaryHeap, HashMap, HashSet}, string};

use ordered_float::OrderedFloat;
use regex::SetMatches;

use crate::index::{b_tree::ValueTreeIndex, documents_store::{Document, DocumentStore}, inverted_index::InvertedIndex, n_gram_index::NgramIndex, n_gram_trie::NgramTrie, tokenizer::{Tokenizer, TokenizerConfig}};

pub struct QueryService<'a>{
       store:  &'a HashMap<String, Document>,
    allow_ngram : bool,
     tokenizer: &'a Tokenizer,
    normal_index: &'a InvertedIndex,
    n_gram_index: &'a Option<NgramIndex>,
    n_gram_trie: &'a Option<NgramTrie>,
    value_tree: &'a ValueTreeIndex,
}

impl<'a> QueryService<'a>{


     pub fn new(state: &'a DocumentStore) -> Self {
        Self {
            store: &state.store,
            allow_ngram: state.allow_ngram,
            tokenizer: &state.tokenizer,
            normal_index: &state.normal_index,
            n_gram_index: &state.n_gram_index,
            n_gram_trie: &state.n_gram_trie,
            value_tree: &state.value_tree,
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

        if let Some(ngram_index) = self.n_gram_trie.as_ref() {
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


       pub fn greater_than(
        &'a self,
        field_path: &str,
        min: i64,
        exclude: Option<&[i64]>,
    ) -> Vec<(&'a String, &'a String)> {
        let max_bound = i64::MAX;
        // > min → (min+1 ..= max_bound)
        self.value_tree.range_query_with_exclude(
            field_path,
            Some(min + 1),
            Some(max_bound),
            exclude,
        )
    }

    pub fn greater_than_equal(
        &'a self,
        field_path: &str,
        min: i64,
        exclude: Option<&[i64]>,
    ) -> Vec<(&'a String, &'a String)> {
        let max_bound = i64::MAX;
        self.value_tree
            .range_query_with_exclude(field_path, Some(min), Some(max_bound), exclude)
    }

    pub fn less_than(
        &'a self,
        field_path: &str,
        max: i64,
        exclude: Option<&[i64]>,
    ) -> Vec<(&'a String, &'a String)> {
        let min_bound = i64::MIN;
        // < max → (min_bound ..= max-1)
        self.value_tree.range_query_with_exclude(
            field_path,
            Some(min_bound),
            Some(max - 1),
            exclude,
        )
    }

    pub fn less_than_equal(
        &'a self,
        field_path: &str,
        max: i64,
        exclude: Option<&[i64]>,
    ) -> Vec<(&'a String, &'a String)> {
        let min_bound = i64::MIN;
        self.value_tree
            .range_query_with_exclude(field_path, Some(min_bound), Some(max), exclude)
    }

    pub fn between(
        &'a self,
        field_path: &str,
        min: i64,
        max: i64,
        exclude: Option<&[i64]>,
    ) -> Vec<(&'a String, &'a String)> {
        self.value_tree
            .range_query_with_exclude(field_path, Some(min), Some(max), exclude)
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

    pub fn and_word(&self, words: Vec<&str>) -> Vec<String> {
        let mut iter = words.into_iter();
        let first = match iter.next() {
            Some(w) => w,
            None => return Vec::new(),
        };

        let mut result: HashSet<String> = self
            .normal_index
            .search_term(&[first])
            .into_iter()
            .collect();

        for word in iter {
            let ids: HashSet<String> = self.normal_index.search_term(&[word]).into_iter().collect();
            result.retain(|id| ids.contains(id));
            if result.is_empty() {
                break;
            }
        }

        result.into_iter().collect()
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

        if let Some(ngram_index) = self.n_gram_trie.as_ref()  {
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