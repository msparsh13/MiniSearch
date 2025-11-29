use std::collections::HashMap;

use crate::{
    commits::commit_manager::CommitManager,
    engine::query_service::QueryService,
    index::{documents_store::DocumentStore, tokenizer::tokenizer::TokenizerConfig, value::Value},
    storage::local_store::LocalStore,
};

pub struct SearchEngine {
    index_path: String,
    commit_log_path: String,
    documents_store: DocumentStore,
    commit_manager: CommitManager,
}

impl<'a> SearchEngine {
    // pub fn new(index_path: String, config: Option<TokenizerConfig>) -> std::io::Result<Self> {
    //     let documents_store = if LocalStore::exists(&index_path) {
    //         // Try to load the existing store
    //         match LocalStore::load::<DocumentStore>(&index_path) {
    //             Ok(store) => store,
    //             Err(err) => {
    //                 eprintln!(
    //                     "Failed to load DocumentStore from {}: {}. Creating a new one.",
    //                     index_path, err
    //                 );
    //                 DocumentStore::new(config)
    //             }
    //         }
    //     } else {
    //         DocumentStore::new(config)
    //     };
    //     let index_path = index_path;

    //     Ok(Self {
    //         index_path,
    //         documents_store,
    //     })
    // }

    pub fn new(
        index_path: String,
        commit_log_path: String,
        config: Option<TokenizerConfig>,
    ) -> std::io::Result<Self> {
        // 1. Load or create DocumentStore snapshot
        let documents_store = if LocalStore::exists(&index_path) {
            match LocalStore::load::<DocumentStore>(&index_path) {
                Ok(store) => store,
                Err(err) => {
                    eprintln!(
                        "Failed to load DocumentStore from {}: {}. Creating a new one.",
                        index_path, err
                    );
                    DocumentStore::new(config)
                }
            }
        } else {
            DocumentStore::new(config)
        };

        let mut commit_manager = CommitManager::new(&commit_log_path);

        // 2. Replay commit log on top of snapshot
        //    This makes snapshot + log = final truth
        let mut store = documents_store;
        commit_manager.replay(&mut store);

        Ok(Self {
            index_path,
            documents_store: store,
            commit_manager,
            commit_log_path,
        })
    }

    pub fn add_document(
        &mut self,
        data: HashMap<String, Value>,
        max_depth: Option<usize>,
    ) -> std::io::Result<String> {
        let id = self
            .commit_manager
            .add_document(&mut self.documents_store, data, max_depth);
        LocalStore::save(&self.documents_store, &self.index_path)?;

        Ok(id)
    }

    pub fn delete_document(&mut self, doc_id: String) -> std::io::Result<String> {
        let id = self
            .commit_manager
            .delete_document(&mut self.documents_store, &doc_id);

        Ok(doc_id)
    }

    pub fn query_service(&self) -> QueryService<'_> {
        QueryService::new(&self.documents_store)
    }

    pub fn close(&self) -> std::io::Result<()> {
        LocalStore::save(&self.documents_store, &self.index_path)
    }

    pub fn store(&self) -> &DocumentStore {
        &self.documents_store
    }

    pub fn store_mut(&mut self) -> &mut DocumentStore {
        &mut self.documents_store
    }
}
