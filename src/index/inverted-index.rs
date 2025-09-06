use std::collections::HashMap;

pub struct Posting {
    pub positions: Vec<usize>, 
    pub term_freq: usize,      
}

pub struct InvertedIndex {
    index: HashMap<String, HashMap<usize, Posting>>,
    deleted_docs: HashSet<usize>,
}

impl InvertedIndex{
     pub fn new() -> Self {
        InvertedIndex {
            index: HashMap::new(),
        }
    }
    pub fn addTerm(&mut self  , term: &str , docId: &str , pos : usize ){
        if self.deleted_docs.contains(&doc_id) {
            return;
        }
        let postings = self.index.entry(term.to_string()).or_insert_with(HashMap::new);

        let posting = postings.entry(doc_id).or_insert_with(|| Posting {
            positions: Vec::new(),
            term_freq: 0,
        });

        posting.positions.push(position);
        posting.term_freq += 1;
    }

    pub fn removeDocument(&mut self, docId: usize) {
       deleted_docs.insert(docId)
    }

    pub fn getPostings(&self, term: &str) -> Option<HashMap<usize, &Posting>> {
        self.index.get(term).map(|postings| {
            postings
                .iter()
                .filter(|(doc_id, _)| !self.deleted_docs.contains(doc_id))
                .map(|(doc_id, posting)| (*doc_id, posting))
                .collect()
        })
    }

    pub fn searchTerm(&self, term: &str) -> Vec<usize> {
        match self.index.get(term) {
            Some(postings) => postings
                .keys()
                .filter(|id| !self.deleted_docs.contains(id))
                .cloned()
                .collect(),
            None => Vec::new(),
        }
    }

      pub fn docFreq(&self, term: &str) -> usize {
        self.search_term(term).len()
    }

    pub fn removeDocument(&mut self, doc_id: usize) {
        self.deleted_docs.insert(doc_id);
    }

    pub fn isDeleted(&self, doc_id: &usize) -> bool {
        self.deleted_docs.contains(doc_id)
    }

    pub fn restoreDocument(&mut self, doc_id: usize) {
        self.deleted_docs.remove(&doc_id);
    }
}