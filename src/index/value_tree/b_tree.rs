use std::collections::{BTreeMap, HashMap, HashSet};
use std::ops::RangeInclusive;

use serde::{Deserialize, Serialize};

use crate::index::value::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
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
            Value::Number(n) => Some((*n * 1000.0) as i64), // allow decimals
            Value::Date(s) => Self::date_to_key(s),
            _ => None,
        }
    }

    fn date_to_key(date: &str) -> Option<i64> {
        // Expected "YYYY-MM-DD"
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
        let min = min * 1000;
        let max = max * 1000;

        if let Some(tree) = self.data.get(field_path) {
            tree.range(min..=max)
                .flat_map(|(_, docs)| docs.iter().map(|(doc_id, field)| (doc_id, field)))
                .collect()
        } else {
            Vec::new()
        }
    }

    pub fn range_query_with_exclude<'a>(
        &'a self,
        leaf_field: &str,
        min: Option<i64>,
        max: Option<i64>,
        exclude_values: Option<&[i64]>,
    ) -> Vec<(&'a String, &'a String)> {
        let given_min = min.map(|v| v * 1000).unwrap_or(i64::MIN);
        let given_max = max.map(|v| v * 1000).unwrap_or(i64::MAX);

        if given_min > given_max {
            return Vec::new();
        }

        let exclude_set: HashSet<i64> = exclude_values
            .map(|vals| vals.iter().map(|v| v * 1000).collect())
            .unwrap_or_default();

        let mut results = Vec::new();

        for (field_path, tree) in &self.data {
            let matches = field_path
                .rsplit('.')
                .next()
                .map(|leaf| leaf == leaf_field)
                .unwrap_or(false);

            if !matches {
                continue;
            }

            for (value, docs) in tree.range(given_min..=given_max) {
                if exclude_set.contains(value) {
                    continue;
                }

                // ✅ return both doc_id and full field_path
                results.extend(docs.iter().map(|(doc_id, full_path)| (doc_id, full_path)));
            }
        }

        results
    }
    pub fn remove_index(&mut self, field_path: &str, value: &Value, doc_id: &str) {
        let Some(key) = Self::normalize_numeric(value) else {
            return;
        };

        if let Some(tree) = self.data.get_mut(field_path) {
            if let Some(vec) = tree.get_mut(&key) {
                vec.retain(|(d, _)| d != doc_id);

                if vec.is_empty() {
                    tree.remove(&key);
                }
            }

            if tree.is_empty() {
                self.data.remove(field_path);
            }
        }
    }

    pub fn sort_query(
        &self,
        field_path: &str,
        candidates: Option<&HashSet<String>>,
        ascending: bool,
    ) -> Vec<String> {
        let mut result = Vec::new();

        let Some(tree) = self.data.get(field_path) else {
            return result;
        };

        // Forward iteration = ASC
        if ascending {
            for (_value, docs) in tree.iter() {
                for (doc_id, _) in docs {
                    if let Some(filter) = candidates {
                        if !filter.contains(doc_id) {
                            continue;
                        }
                    }
                    result.push(doc_id.clone());
                }
            }
        } else {
            // Reverse iteration = DESC
            for (_value, docs) in tree.iter().rev() {
                for (doc_id, _) in docs {
                    if let Some(filter) = candidates {
                        if !filter.contains(doc_id) {
                            continue;
                        }
                    }
                    result.push(doc_id.clone());
                }
            }
        }

        result
    }
}
