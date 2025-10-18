use std::collections::{HashMap, HashSet};
#[derive(Debug)]
pub struct Posting {
    pub positions: Vec<usize>,
    pub term_freq: usize,
    pub field_paths: HashSet<String>,
}
#[derive(Debug)]
pub struct InvertedIndex {
    index: HashMap<String, HashMap<usize, Posting>>,
    deleted_docs: HashSet<usize>,
    doc_lengths: HashMap<usize, usize>,
}
/**
 * Todo Create bm 25 here
 */
impl InvertedIndex {
    pub fn new() -> Self {
        InvertedIndex {
            index: HashMap::new(),
            deleted_docs: HashSet::new(),
            doc_lengths: HashMap::new(),
        }
    }

    pub fn add_term(&mut self, term: &str, doc_id: usize, pos: usize, field_path: &str) {
        if self.deleted_docs.contains(&doc_id) {
            return;
        }

        let postings = self
            .index
            .entry(term.to_string())
            .or_insert_with(HashMap::new);

        let posting = postings.entry(doc_id).or_insert_with(|| Posting {
            positions: Vec::new(),
            field_paths: HashSet::new(),
            term_freq: 0,
        });

        posting.positions.push(pos); // push position
        posting.field_paths.insert(field_path.to_string()); // insert field path
        posting.term_freq += 1; // increment frequency
        self.doc_lengths
            .entry(doc_id)
            .and_modify(|len| *len += 1)
            .or_insert(1);
    }

    pub fn get_postings(&self, term: &str) -> Option<HashMap<usize, &Posting>> {
        self.index.get(term).map(|postings| {
            postings
                .iter()
                .filter(|(doc_id, _)| !self.deleted_docs.contains(doc_id))
                .map(|(doc_id, posting)| (*doc_id, posting))
                .collect()
        })
    }

    pub fn search_term(&self, term: &str) -> Vec<usize> {
        match self.index.get(term) {
            Some(postings) => postings
                .keys()
                .filter(|id| !self.deleted_docs.contains(id))
                .cloned()
                .collect(),
            None => Vec::new(),
        }
    }

    pub fn search_term_with_fields(&self, term: &str) -> Vec<(usize, Vec<String>)> {
        match self.index.get(term) {
            Some(postings) => postings
                .iter()
                .filter(|(doc_id, _)| !self.deleted_docs.contains(doc_id))
                .map(|(doc_id, posting)| {
                    // return doc_id + field paths as Vec
                    (*doc_id, posting.field_paths.iter().cloned().collect())
                })
                .collect(),
            None => Vec::new(),
        }
    }

    pub fn doc_freq(&self, term: &str) -> usize {
        self.search_term(term).len()
    }

    pub fn remove_document(&mut self, doc_id: usize) {
        self.deleted_docs.insert(doc_id);
    }

    pub fn is_deleted(&self, doc_id: &usize) -> bool {
        self.deleted_docs.contains(doc_id)
    }

    pub fn compact_index(&mut self) {
        for postings in self.index.values_mut() {
            for doc_id in &self.deleted_docs {
                postings.remove(doc_id);
            }
        }
        self.deleted_docs.clear();
    }

    pub fn bm25_search(&self, query: &[&str], k1: f64, b: f64) -> HashMap<usize, f64> {
        let mut scores: HashMap<usize, f64> = HashMap::new();
        let n_docs = self.doc_lengths.len() as f64;
        let avg_doc_len = self.doc_lengths.values().sum::<usize>() as f64 / n_docs;

        for &term in query {
            if let Some(postings) = self.index.get(term) {
                let df = postings.len() as f64;
                let idf = ((n_docs - df + 0.5) / (df + 0.5) + 1.0).ln();

                for (&doc_id, posting) in postings {
                    if self.deleted_docs.contains(&doc_id) {
                        continue;
                    }
                    let tf = posting.term_freq as f64;
                    let doc_len = *self.doc_lengths.get(&doc_id).unwrap_or(&1) as f64;

                    let denom = tf + k1 * (1.0 - b + b * doc_len / avg_doc_len);
                    let score = idf * (tf * (k1 + 1.0)) / denom;

                    *scores.entry(doc_id).or_insert(0.0) += score;
                }
            }
        }

        scores
    }

    pub fn search_term_in_field(&self, term: &str, field: &str) -> Vec<usize> {
        self.index
            .get(term)
            .map(|postings| {
                postings
                    .iter()
                    .filter(|(_, posting)| posting.field_paths.contains(field))
                    .map(|(doc_id, _)| *doc_id)
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn search_term_in_field_tree(&self, term: &str, field_prefix: &str) -> Vec<usize> {
        let mut results = Vec::new();

        // 1. Get postings for the term
        if let Some(postings) = self.index.get(term) {
            for (&doc_id, posting) in postings {
                if self.deleted_docs.contains(&doc_id) {
                    continue;
                }

                // 2. Check if any field path starts with the given prefix
                let has_nested_match = posting
                    .field_paths
                    .iter()
                    .any(|path| path.split('.').any(|part| part == field_prefix));

                if has_nested_match {
                    results.push(doc_id);
                }
            }
        }

        results
    }
}
