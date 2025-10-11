use std::collections::{HashMap, HashSet};

#[derive(Debug, Default)]
pub struct TrieNode {
    children: HashMap<char, TrieNode>,
    terms: HashSet<String>,
}

#[derive(Debug, Default)]
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

    pub fn get_terms(&self, word: &str) -> Vec<String> {
        let mut node = &self.root;
        for ch in word.chars() {
            if let Some(next) = node.children.get(&ch) {
                node = next;
            } else {
                return Vec::new();
            }
        }
        node.terms.iter().cloned().collect()
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
}
