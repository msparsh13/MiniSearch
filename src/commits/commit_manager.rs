use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{BufRead, Seek, SeekFrom, Write};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::index::documents_store::DocumentStore;
use crate::index::value::Value;
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
}

impl CommitManager {
    pub fn new(log_path: &str) -> Self {
        let file = OpenOptions::new()
            .create(true)
            .read(true)
            .append(true)
            .open(log_path)
            .unwrap();

        Self {
            log_file: file,
            next_commit_id: 1,
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
        data: HashMap<String, Value>,
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
        id
    }

    /// Public: Delete doc through commit manager
    pub fn delete_document(&mut self, store: &mut DocumentStore, id: &str) {
        let commit = self.create_commit(CommitOp::Delete { id: id.to_string() });
        self.append_to_log(&commit);

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
                    store.add_document(data, None);
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
                    store.add_document(data, None);
                }
                CommitOp::Delete { id } => {
                    store.delete_index(&id);
                }
            }
        }
    }
}
