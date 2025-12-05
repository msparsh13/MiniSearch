use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{BufRead, Seek, SeekFrom, Write};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::index::documents_store::DocumentStore;
use crate::index::value::Value;
use crate::snapshots::snapshot_manager::SnapshotManager;
use crate::utils::validator::validate_document;
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub enum CommitOp {
    Add {
        id: String,
        data: HashMap<String, Value>,
    },
    Delete {
        id: String,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Commit {
    pub id: u64,
    pub op: CommitOp,
    pub timestamp: u64,
}

pub struct CommitManager {
    log_file: File,
    next_commit_id: u64,
    snapshot_manager: SnapshotManager,
}

impl CommitManager {
    pub fn new(log_path: &str, snapshot_path: &str, count: u32) -> Self {
        let file = OpenOptions::new()
            .create(true)
            .read(true)
            .append(true)
            .open(log_path)
            .unwrap();

        Self {
            log_file: file,
            next_commit_id: 1,
            snapshot_manager: SnapshotManager::new(snapshot_path, count),
        }
    }

    fn now_ts() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    /// Create + log + return commit
    fn create_commit(&mut self, op: CommitOp) -> Commit {
        let c = Commit {
            id: self.next_commit_id,
            op,
            timestamp: Self::now_ts(),
        };
        self.next_commit_id += 1;

        c
    }

    fn append_to_log(&mut self, commit: &Commit) {
        // serialize commit as JSON line
        let encoded = serde_json::to_string(commit).unwrap();
        self.log_file.write_all(encoded.as_bytes()).unwrap();
        self.log_file.write_all(b"\n").unwrap();
        self.log_file.flush().unwrap();
        self.log_file.sync_all().unwrap();
    }

    /// Public: Add doc through commit manager
    pub fn add_document(
        &mut self,
        store: &mut DocumentStore,
        data: &HashMap<String, Value>,
        max_depth: Option<usize>,
    ) -> String {
        //validation small for now
        validate_document(&data);
        let id = format!("{}", store.store.len() + 1);

        let commit = self.create_commit(CommitOp::Add {
            id: id.clone(),
            data: data.clone(),
        });
        self.append_to_log(&commit);

        store.add_document(data, max_depth);

        if (commit.id) % 1 == 0 {
            let snapshot = store.to_snapshot();
            self.snapshot_manager.save(&snapshot);
        }

        id
    }

    /// Public: Delete doc through commit manager
    pub fn delete_document(&mut self, store: &mut DocumentStore, id: &str) {
        let commit = self.create_commit(CommitOp::Delete { id: id.to_string() });
        self.append_to_log(&commit);

        if (commit.id % 1 == 0) {
            let snapshot = store.to_snapshot();
            self.snapshot_manager.save(&snapshot);
        }
        store.delete_index(id);
    }

    /// Replay log on startup
    pub fn replay(&mut self, store: &mut DocumentStore) {
        self.log_file.seek(SeekFrom::Start(0)).unwrap();
        let reader = std::io::BufReader::new(&self.log_file);

        for line in reader.lines() {
            let buf = line.unwrap();
            if buf.trim().is_empty() {
                continue;
            }

            let commit: Commit = serde_json::from_str(&buf).unwrap();
            self.next_commit_id = commit.id + 1;

            match commit.op {
                CommitOp::Add { id: _, data } => {
                    store.add_document(&data, None);
                }
                CommitOp::Delete { id } => {
                    store.delete_index(&id);
                }
            }
        }
    }

    pub fn rollback_to(&mut self, store: &mut DocumentStore, commit_id: u64) {
        // 1. load all commits
        self.log_file.seek(SeekFrom::Start(0)).unwrap();
        let reader = std::io::BufReader::new(&self.log_file);

        let mut commits = Vec::new();
        for line in reader.lines() {
            let buf = line.unwrap();
            if buf.trim().is_empty() {
                continue;
            }

            let commit: Commit = serde_json::from_str(&buf).unwrap();
            commits.push(commit);
        }

        *store = DocumentStore::new(None);
        for c in commits.into_iter().filter(|c| c.id <= commit_id) {
            match c.op {
                CommitOp::Add { id: _, data } => {
                    store.add_document(&data, None);
                }
                CommitOp::Delete { id } => {
                    store.delete_index(&id);
                }
            }
        }
    }

    pub fn replay_withSnapshot(&mut self, store: &mut DocumentStore) {
        // 1. Load latest snapshot
        let snapshot_opt = self.snapshot_manager.load();

        let mut last_snapshot_commit = 0;

        if let Some(snapshot) = snapshot_opt {
            // restore index structures
            store.normal_index = snapshot.normal_index;
            store.n_gram_index = snapshot.n_gram_index;
            store.n_gram_trie = snapshot.n_gram_trie;
            store.value_tree = snapshot.value_tree;
            store.forward_index = snapshot.forward_index;
            last_snapshot_commit = snapshot.last_commit_id.parse().unwrap_or(0);

            // documents must already be loaded separately from data.json
            // snapshot only contains indexes
        }

        // 2. Replay log FROM last_snapshot_commit + 1
        self.log_file.seek(SeekFrom::Start(0)).unwrap();
        let reader = std::io::BufReader::new(&self.log_file);

        for line in reader.lines() {
            let buf = line.unwrap();
            if buf.trim().is_empty() {
                continue;
            }

            let commit: Commit = serde_json::from_str(&buf).unwrap();

            // Skip commits already included in snapshot
            if commit.id <= last_snapshot_commit {
                continue;
            }

            // track next commit id
            self.next_commit_id = commit.id + 1;

            match commit.op {
                CommitOp::Add { id: _, data } => {
                    store.add_document(&data, None);
                }
                CommitOp::Delete { id } => {
                    store.delete_index(&id);
                }
            }
        }
    }
}
