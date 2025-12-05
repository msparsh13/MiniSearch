use serde::{Deserialize, Serialize};
use serde_json;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use crate::index::documents_store::DocumentStore;
use crate::index::forward_indexer::ForwardIndex;
use crate::index::inverted_index::inverted_index::InvertedIndex;
use crate::index::n_gram::n_gram_index::NgramIndex;
use crate::index::n_gram::n_gram_trie::NgramTrie;
use crate::index::value_tree::b_tree::ValueTreeIndex;
use crate::storage::local_store::LocalStore;

#[derive(Serialize, Deserialize)]
pub struct Snapshot {
    pub allow_ngram: bool,
    pub normal_index: InvertedIndex,
    pub n_gram_index: Option<NgramIndex>,
    pub n_gram_trie: Option<NgramTrie>,
    pub value_tree: ValueTreeIndex,
    pub forward_index: ForwardIndex,
    pub last_commit_id: String,
}

#[derive(Serialize, Deserialize)]
pub struct SnapshotMeta {
    pub curr: u32, // tracks latest snapshot index
}

pub struct SnapshotManager {
    path: String, // base path, e.g., "snapshot"
    count: u32,   // number of rotating slots
    curr: u32,    // latest snapshot index
}

impl SnapshotManager {
    pub fn new(path: impl Into<String>, count: u32) -> Self {
        Self {
            path: path.into(),
            count,
            curr: 0,
        }
    }

    /// Save snapshot in rotating slots

    fn snapshot_path(&self, idx: u32) -> String {
        format!("{}/snapshot_{}.json", self.path, idx)
    }

    fn meta_path(&self) -> String {
        format!("{}/meta.json", self.path)
    }

    /// Save a rotating snapshot
    pub fn save(&mut self, snapshot: &Snapshot) -> std::io::Result<()> {
        // 1–N rotation
        self.curr = (self.curr % self.count) + 1;

        let path = self.snapshot_path(self.curr);
        let tmp_path = format!("{}.tmp", path);
        print!("{}", path);
        // --- Use LocalStore to save temp file ---
        LocalStore::save(snapshot, &tmp_path)?;

        // --- Atomic replace ---
        std::fs::rename(tmp_path, &path)?;

        // --- Save meta using LocalStore ---
        let meta = SnapshotMeta { curr: self.curr };
        LocalStore::save(&meta, &self.meta_path())?;

        Ok(())
    }

    /// Load the latest snapshot
    pub fn load(&self) -> Option<Snapshot> {
        // Read meta
        let meta: SnapshotMeta = LocalStore::load(&self.meta_path()).ok()?;
        let path = self.snapshot_path(meta.curr);

        // Read snapshot if file exists
        if LocalStore::exists(&path) {
            LocalStore::load(&path).ok()
        } else {
            None
        }
    }
}
