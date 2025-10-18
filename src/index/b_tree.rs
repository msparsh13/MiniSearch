use std::{
    collections::{BTreeMap, HashMap},
    string,
};

use crate::index::documents_store::Value;

#[derive(Debug, Clone)]
pub struct ValueTreeIndex {
    // field_path -> BTreeMap<normalized_value, Vec<(doc_id, field_path)>>
    pub data: HashMap<String, BTreeMap<i64, Vec<(String, String)>>>,
}

impl ValueTreeIndex {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    fn normalize_numeric(value: &Value) -> Option<i64> {
        match value {
            Value::Number(n) => Some((*n * 1000.0) as i64), // allowing decimals
            Value::Date(s) => Self::date_to_key(s),
            _ => None,
        }
    }

    fn date_to_key(date: &str) -> Option<i64> {
        // Expected "YYYY-MM-DD" format
        let parts: Vec<&str> = date.split('-').collect();
        if parts.len() != 3 {
            return None;
        }

        let y = parts[0].parse::<i64>().ok()?;
        let m = parts[1].parse::<i64>().ok()?;
        let d = parts[2].parse::<i64>().ok()?;

        Some(y * 10000 + m * 100 + d)
    }

    pub fn add_index(&mut self, field_path: &str, value: &Value, doc_id: &str) {
        if let Some(key) = Self::normalize_numeric(value) {
            let tree = self
                .data
                .entry(field_path.to_string())
                .or_insert_with(BTreeMap::new);

            tree.entry(key)
                .or_insert_with(Vec::new)
                .push((doc_id.to_string(), field_path.to_string()));
        }
    }

    pub fn range_query<'a>(
        &'a self,
        field_path: &str,
        min: i64,
        max: i64,
    ) -> Vec<(&'a String, &'a String)> {
        if let Some(tree) = self.data.get(field_path) {
            tree.range(min..=max)
                .flat_map(|(_, docs)| docs.iter().map(|(doc_id, field)| (doc_id, field)))
                .collect()
        } else {
            Vec::new()
        }
    }
}
