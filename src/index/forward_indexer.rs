use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForwardDoc {
    pub text_fields: HashMap<String, String>,
    pub numeric_fields: HashMap<String, f64>,
    pub date_fields: HashMap<String, String>,
}

impl ForwardDoc {
    pub fn new() -> Self {
        Self {
            text_fields: HashMap::new(),
            numeric_fields: HashMap::new(),
            date_fields: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForwardIndex {
    pub docs: HashMap<String, ForwardDoc>,
}

impl ForwardIndex {
    pub fn new() -> Self {
        Self {
            docs: HashMap::new(),
        }
    }

    pub fn add_doc(&mut self, doc_id: &str, forward: ForwardDoc) {
        self.docs.insert(doc_id.to_string(), forward);
    }

    pub fn get(&self, doc_id: &str) -> Option<&ForwardDoc> {
        self.docs.get(doc_id)
    }

    pub fn remove(&mut self, doc_id: &str) {
        self.docs.remove(doc_id);
    }
}
