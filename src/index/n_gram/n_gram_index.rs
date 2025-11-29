use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct NgramIndex {
    map: HashMap<String, HashSet<String>>,
}

impl NgramIndex {
    pub fn new() -> Self {
        NgramIndex {
            map: HashMap::new(),
        }
    }

    pub fn insert(&mut self, word: &str, term: &str) {
        self.map
            .entry(word.to_string())
            .or_insert_with(HashSet::new)
            .insert(term.to_string());
    }

    pub fn get_terms(&self, word: &str) -> Vec<String> {
        self.map
            .get(word)
            .map(|set| set.iter().cloned().collect())
            .unwrap_or_default()
    }
}
