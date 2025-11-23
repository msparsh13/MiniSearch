mod index;
mod storage;

use crate::index::query_service;
use crate::index::search_engine::SearchEngine;
use crate::storage::local_store::LocalStore;
use crate::{
    index::{
        documents_store::{DocumentStore, Value},
        inverted_index,
        query_service::QueryService,
        tokenizer::TokenizerConfig,
    },
    storage::local_store,
};
use std::collections::HashMap;
/*
 * TODO:
 * Objects within objects not being read make them read by inverted index :: fixed
 * Add ngram support:: fixed
 * create function to get proper location of database object:fixed
 * to do create ngram inverted index bit different [implement both hashmap and trie] :fixed
 * check for deleted doc
 * ngram index as trie :: fixed we have both idx and trie
 * make two mods schema less and with schema
 *  we can add more complex function like search attribute names not search them to add flexibility :fixed
 * now time to make complex queries > < smth : fixed
 * Proper Date normalization
 * Proper delete step 1 using forward index
 * Proper update step using delete and add [no partial update]
 */
fn main() -> std::io::Result<()> {
    // tokenizer config (ngrams/stemming)
    let config = TokenizerConfig {
        use_stemming: false,
        min_ngram: Some(2),
        max_ngram: Some(5),
    };

    let index_path = "./data/data.json".to_string();

    // ✅ Use SearchEngine instead of manual DocumentStore
    let mut engine = SearchEngine::new(index_path, Some(config))?;

    // Small helper to add & index a document via SearchEngine
    fn add_and_index(
        engine: &mut SearchEngine,
        data: HashMap<String, Value>,
        max_depth: usize,
    ) -> std::io::Result<String> {
        engine.add_document(data, Some(max_depth))
    }

    // --- Document 1
    let mut doc1 = HashMap::new();
    doc1.insert(
        "text".to_string(),
        Value::Text("Rust programming is fun".to_string()),
    );
    // let doc1_id = add_and_index(&mut engine, doc1, 2)?;
    // println!("Added doc1 id = {}", doc1_id);

    // --- Document 2 (attributes + nested map)
    let mut doc2 = HashMap::new();
    let mut attributes = HashMap::new();
    attributes.insert("language".to_string(), Value::Text("Rust".to_string()));
    attributes.insert("year".to_string(), Value::Number(2025.0));
    let mut inner = HashMap::new();
    inner.insert("Mew".to_string(), Value::Text("pokemon".to_string()));
    inner.insert("Shoutmon".to_string(), Value::Text("digimon".to_string()));
    attributes.insert("KEY".to_string(), Value::Object(inner));
    doc2.insert("attributes".to_string(), Value::Object(attributes));
    // let doc2_id = add_and_index(&mut engine, doc2, 4)?;
    // println!("Added doc2 id = {}", doc2_id);

    // --- Document 3 (trainer + nested Pokémon)
    let mut pikachu_stats = HashMap::new();
    pikachu_stats.insert("hp".to_string(), Value::Number(35.0));
    pikachu_stats.insert("attack".to_string(), Value::Number(55.0));
    pikachu_stats.insert("defense".to_string(), Value::Number(40.0));

    let mut pikachu_moves = HashMap::new();
    pikachu_moves.insert(
        "quick_attack".to_string(),
        Value::Text("Electric".to_string()),
    );
    pikachu_moves.insert(
        "thunderbolt".to_string(),
        Value::Text("Electric".to_string()),
    );

    let mut pikachu_abilities = HashMap::new();
    pikachu_abilities.insert("primary".to_string(), Value::Text("Static".to_string()));
    pikachu_abilities.insert(
        "hidden".to_string(),
        Value::Text("Lightning Rod".to_string()),
    );

    let mut pikachu = HashMap::new();
    pikachu.insert("stats".to_string(), Value::Object(pikachu_stats));
    pikachu.insert("moves".to_string(), Value::Object(pikachu_moves));
    pikachu.insert("abilities".to_string(), Value::Object(pikachu_abilities));
    pikachu.insert("type".to_string(), Value::Text("Electric".to_string()));
    pikachu.insert("generation".to_string(), Value::Number(1.0));

    let mut trainer_info = HashMap::new();
    trainer_info.insert("name".to_string(), Value::Text("Ash Ketchum".to_string()));
    trainer_info.insert(
        "hometown".to_string(),
        Value::Text("Pallet Town".to_string()),
    );
    let mut team = HashMap::new();
    team.insert("pikachu".to_string(), Value::Object(pikachu));
    trainer_info.insert("team".to_string(), Value::Object(team));

    let mut doc3 = HashMap::new();
    doc3.insert("trainer".to_string(), Value::Object(trainer_info));
    doc3.insert(
        "tournament".to_string(),
        Value::Text("Indigo League".to_string()),
    );
    // let doc3_id = add_and_index(&mut engine, doc3, 4)?;
    // println!("Added doc3 id = {}", doc3_id);

    // You can still debug-print from the underlying store
    // println!(
    //     "Normal inverted index snapshot:\n{:#?}",
    //     engine.store().normal_index
    // );
    // println!(
    //     "Normal value tree snapshot:\n{:#?}",
    //     engine.store().value_tree
    // );

    // println!(
    //     "N-gram index enabled? {}",
    //     engine.store().n_gram_index.is_some()
    // );
    println!("N-gram trie enabled? {:#?}", engine.store().n_gram_trie);

    // ✅ Get a QueryService from the engine
    let query_service = engine.query_service();

    // --- Basic term search
    let term = "attack";
    let docs_with_term = query_service.get_words(vec![term]);
    //println!("Documents containing '{}': {:?}", term, docs_with_term);

    // --- N-gram BM25 search example
    let k1 = 1.2;
    let b = 0.75;
    let alpha = 0.7;
    let beta = 0.3;
    let top_k = 5;
    let query = "mon"; // should match "pokemon"/"digimon" etc. via n-gram
    let ngram_results = query_service.ngram_bm25(query, k1, b, alpha, beta, top_k);

    // println!("ngram_bm25 results for query '{}':", query);
    // if ngram_results.is_empty() {
    //     println!("  (no results — either ngram disabled or no matches)");
    // } else {
    //     for (doc_id, score) in ngram_results {
    //         println!("  doc {} => score {:.6}", doc_id, score);
    //     }
    // }

    // --- Range queries (numeric/date)
    let year_min = 2020 * 1000;
    let year_max = 2030 * 1000;
    let year_matches = query_service.range_query("attributes.year", year_min, year_max);
    // println!(
    //     "Documents with 'attributes.year' in [2020..2030]: {:?}",
    //     year_matches
    // );

    let possible_paths = [
        "trainer.team.pikachu.generation",
        "trainer.team.pikachu.stats.generation",
        "trainer.generation",
        "generation",
    ];

    for path in &possible_paths {
        let matches = query_service.range_query(path, 1 * 1000, 1 * 1000);
        // if !matches.is_empty() {
        //     println!("Found generation=1 at path '{}': {:?}", path, matches);
        // }
    }

    let excluded = query_service.not_word(vec!["rust"]);
    // println!("Documents that do NOT contain 'rust': {:?}", excluded);
    println!("{:?}", engine.store().forward_index);
    // let ids_for_attack = query_service.get_words(vec!["attack"]);
    // println!("get_words for 'attack' -> {:?}", ids_for_attack);

    // println!("Finished checks. Inspect printed structures above for correctness.");

    // ✅ No need to manually call LocalStore::save here, SearchEngine already saves on add + close
    engine.close()?;

    Ok(())
}
