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
}

impl InvertedIndex {
    pub fn new() -> Self {
        InvertedIndex {
            index: HashMap::new(),
            deleted_docs: HashSet::new(),
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

    posting.positions.push(pos);                       // push position
    posting.field_paths.insert(field_path.to_string()); // insert field path
    posting.term_freq += 1;                             // increment frequency
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
}