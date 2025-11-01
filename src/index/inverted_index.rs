use std::collections::{HashMap, HashSet};
#[derive(Debug)]
pub struct Posting {
    pub positions: Vec<usize>,
    pub term_freq: usize,
    pub field_paths: HashSet<String>,
}
#[derive(Debug)]
pub struct InvertedIndex {
    index: HashMap<String, HashMap<String, Posting>>,
    deleted_docs: HashSet<String>,
    doc_lengths: HashMap<String, usize>,
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

    pub fn add_term(&mut self, term: &str, doc_id: &str, pos: usize, field_path: &str) {
        if self.deleted_docs.contains(doc_id) {
            return;
        }

        let postings = self
            .index
            .entry(term.to_owned())
            .or_insert_with(HashMap::new);

        let posting = postings
            .entry(doc_id.to_owned())
            .or_insert_with(|| Posting {
                positions: Vec::new(),
                field_paths: HashSet::new(),
                term_freq: 0,
            });

        posting.positions.push(pos); // push position
        posting.field_paths.insert(field_path.to_owned()); // insert field path
        posting.term_freq += 1; // increment frequency
        self.doc_lengths
            .entry(doc_id.to_owned())
            .and_modify(|len| *len += 1)
            .or_insert(1);
    }

    pub fn get_postings(&self, term: &str) -> Option<impl Iterator<Item = (&String, &Posting)>> {
        self.index.get(term).map(|postings| {
            postings
                .iter()
                .filter(|(doc_id, _)| !self.deleted_docs.contains(*doc_id))
        })
    }

    pub fn search_term(&self, terms: &[&str]) -> Vec<String> {
        let mut result: HashSet<&String> = HashSet::new();

        for term in terms {
            let term_lc = term.to_lowercase();
            if let Some(postings) = self.index.get(&term_lc) {
                for id in postings.keys() {
                    if !self.deleted_docs.contains(id) {
                        result.insert(id);
                    }
                }
            }
        }

        result.into_iter().cloned().collect()
    }

    pub fn search_term_with_fields(&self, term: &str) -> Vec<(String, Vec<String>)> {
        match self.index.get(term) {
            Some(postings) => postings
                .iter()
                .filter(|(doc_id, _)| !self.deleted_docs.contains(*doc_id))
                .map(|(doc_id, posting)| {
                    (
                        doc_id.clone(),
                        posting.field_paths.iter().cloned().collect(),
                    )
                })
                .collect(),
            None => Vec::new(),
        }
    }

    pub fn doc_freq(&self, term: &str) -> usize {
        self.search_term(&[term]).len()
    }

    pub fn remove_document(&mut self, doc_id: &str) {
        self.deleted_docs.insert(doc_id.to_owned());
    }

    pub fn is_deleted(&self, doc_id: &str) -> bool {
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

    pub fn bm25_search(&self, query: &[&str], k1: f64, b: f64) -> HashMap<String, f64> {
        let mut scores: HashMap<String, f64> = HashMap::new();
        let n_docs = self.doc_lengths.len() as f64;
        let avg_doc_len = self.doc_lengths.values().sum::<usize>() as f64 / n_docs;

        for &term in query {
            if let Some(postings) = self.index.get(term) {
                let df = postings.len() as f64;
                let idf = ((n_docs - df + 0.5) / (df + 0.5) + 1.0).ln();

                for (doc_id, posting) in postings {
                    if self.deleted_docs.contains(doc_id) {
                        continue;
                    }
                    let tf = posting.term_freq as f64;
                    let doc_len = self.doc_lengths.get(doc_id).copied().unwrap_or(1) as f64;

                    let denom = tf + k1 * (1.0 - b + b * doc_len / avg_doc_len);
                    let score = idf * (tf * (k1 + 1.0)) / denom;

                    *scores.entry(doc_id.clone()).or_insert(0.0) += score;
                }
            }
        }

        scores
    }

    pub fn search_term_in_field(&self, term: &str, field: &str) -> Vec<String> {
        self.index
            .get(term)
            .map(|postings| {
                postings
                    .iter()
                    .filter(|(_, posting)| posting.field_paths.contains(field))
                    .map(|(doc_id, _)| doc_id.clone())
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn search_term_in_field_tree(&self, term: &str, field_prefix: &str) -> Vec<String> {
        let mut results = Vec::new();

        // 1. Get postings for the term
        if let Some(postings) = self.index.get(term) {
            for (doc_id, posting) in postings {
                if self.deleted_docs.contains(doc_id) {
                    continue;
                }

                // 2. Check if any field path starts with the given prefix
                let has_nested_match = posting
                    .field_paths
                    .iter()
                    .any(|path| path.split('.').any(|part| part == field_prefix));

                if has_nested_match {
                    results.push(doc_id.clone());
                }
            }
        }

        results
    }

    pub fn search_term_with_fields_short<'a>(
        &'a self,
        term: &str,
    ) -> Vec<(&'a String, Vec<&'a String>)> {
        self.index
            .get(term)
            .map(|postings| {
                postings
                    .iter()
                    .filter(|(doc_id, _)| !self.deleted_docs.contains(*doc_id))
                    .map(|(doc_id, posting)| (doc_id, posting.field_paths.iter().collect()))
                    .collect()
            })
            .unwrap_or_default()
    }
}
