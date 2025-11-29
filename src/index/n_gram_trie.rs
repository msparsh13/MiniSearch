use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct TrieNode {
    children: HashMap<char, TrieNode>,
    terms: HashSet<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct NgramTrie {
    root: TrieNode,
}

impl NgramTrie {
    pub fn new() -> Self {
        NgramTrie {
            root: TrieNode::default(),
        }
    }

    pub fn insert(&mut self, word: &str, term: &str) {
        let mut node = &mut self.root;
        for ch in word.chars() {
            node = node.children.entry(ch).or_insert_with(TrieNode::default);
        }
        node.terms.insert(term.to_string());
    }

    pub fn get_terms<'a>(&'a self, word: &str) -> Vec<&'a str> {
        let mut node = &self.root;
        for ch in word.chars() {
            match node.children.get(&ch) {
                Some(next) => node = next,
                None => return Vec::new(),
            }
        }
        node.terms.iter().map(|s| s.as_str()).collect()
    }

    pub fn get_terms_with_prefix(&self, prefix: &str) -> Vec<String> {
        let mut node = &self.root;
        for ch in prefix.chars() {
            if let Some(next) = node.children.get(&ch) {
                node = next;
            } else {
                return Vec::new();
            }
        }
        let mut result = HashSet::new();
        Self::collect_terms(node, &mut result);
        result.into_iter().collect()
    }

    fn collect_terms(node: &TrieNode, result: &mut HashSet<String>) {
        for term in &node.terms {
            result.insert(term.clone());
        }
        for child in node.children.values() {
            Self::collect_terms(child, result);
        }
    }

    pub fn remove_word(&mut self, word: &str, term: &str) -> bool {
        Self::remove_rec(&mut self.root, word, term, 0)
    }

    fn remove_rec(node: &mut TrieNode, word: &str, term: &str, idx: usize) -> bool {
        if idx == word.len() {
            node.terms.remove(term);
            return node.terms.is_empty() && node.children.is_empty();
        }

        let ch = word.chars().nth(idx).unwrap();

        if let Some(child) = node.children.get_mut(&ch) {
            let should_delete_child = Self::remove_rec(child, word, term, idx + 1);

            if should_delete_child {
                node.children.remove(&ch);
            }
        }

        node.terms.is_empty() && node.children.is_empty()
    }
}
