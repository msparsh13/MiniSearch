mod index;

use crate::index::{
    documents_store::{DocumentStore, Value},
    inverted_index,
    tokenizer::TokenizerConfig,
};
use std::collections::HashMap;

/**
 * TODO:
 * Objects within objects not being read make them read by inverted index :: fixed
 * Add ngram support:: fixed
 * create function to get proper location of database object:fixed
 * to do create ngram inverted index bit different [implement both hashmap and trie] :fixed
 * check for deleted doc
 * ngram index as trie :: fixed we have both idx and trie
 *  we can add more complex function like search attribute names not search them to add flexibility :fixed 
 * now time to make complex queries > < smth
 */
fn main() {
    let config = TokenizerConfig {
        use_stemming: false,
        min_ngram: Some(2),
        max_ngram: Some(5),
    };

    let mut store = DocumentStore::new(Some(config));

    let mut doc1_data = HashMap::new();
    doc1_data.insert(
        "text".to_string(),
        Value::Text("Rust programming is fun".to_string()),
    );
    let doc1_id = store.add_document(doc1_data, Some(2));

    // Fix: clone the document data to avoid borrow conflicts
    let doc1_data_ref = store.get_document(&doc1_id).unwrap().data.clone();
    store.index_document(&doc1_id, &doc1_data_ref, 2);

    let mut doc2_data = HashMap::new();
    let mut attributes = HashMap::new();
    attributes.insert("language".to_string(), Value::Text("Rust".to_string()));
    attributes.insert("year".to_string(), Value::Number(2025.0));
    let mut inner_map = HashMap::new();
    inner_map.insert("Mew".to_string(), Value::Text("pokemon".to_string()));
    inner_map.insert("Shoutmon".to_string(), Value::Text("digimon".to_string()));
    attributes.insert("KEY".to_string(), Value::Object(inner_map));
    doc2_data.insert("attributes".to_string(), Value::Object(attributes));
    let doc2_id = store.add_document(doc2_data, Some(4));

    let doc2_data_ref = store.get_document(&doc2_id).unwrap().data.clone();
    store.index_document(&doc2_id, &doc2_data_ref, 4);
    //println!("{:#?}", store.normal_index);

    // Pokémon stats
    let mut pikachu_stats = HashMap::new();
    pikachu_stats.insert("hp".to_string(), Value::Number(35.0));
    pikachu_stats.insert("attack".to_string(), Value::Number(55.0));
    pikachu_stats.insert("defense".to_string(), Value::Number(40.0));

    // Pokémon moves
    let mut pikachu_moves = HashMap::new();
    pikachu_moves.insert(
        "quick_attack".to_string(),
        Value::Text("Electric".to_string()),
    );
    pikachu_moves.insert(
        "thunderbolt".to_string(),
        Value::Text("Electric".to_string()),
    );

    // Pokémon abilities (nested object)
    let mut pikachu_abilities = HashMap::new();
    pikachu_abilities.insert("primary".to_string(), Value::Text("Static".to_string()));
    pikachu_abilities.insert(
        "hidden".to_string(),
        Value::Text("Lightning Rod".to_string()),
    );

    // Combine stats, moves, abilities into one Pokémon object
    let mut pikachu_data = HashMap::new();
    pikachu_data.insert("stats".to_string(), Value::Object(pikachu_stats));
    pikachu_data.insert("moves".to_string(), Value::Object(pikachu_moves));
    pikachu_data.insert("abilities".to_string(), Value::Object(pikachu_abilities));
    pikachu_data.insert("type".to_string(), Value::Text("Electric".to_string()));
    pikachu_data.insert("generation".to_string(), Value::Number(1.0));

    // Trainer info
    let mut trainer_info = HashMap::new();
    trainer_info.insert("name".to_string(), Value::Text("Ash Ketchum".to_string()));
    trainer_info.insert(
        "hometown".to_string(),
        Value::Text("Pallet Town".to_string()),
    );
    trainer_info.insert(
        "team".to_string(),
        Value::Object({
            let mut team = HashMap::new();
            team.insert("pikachu".to_string(), Value::Object(pikachu_data));
            team
        }),
    );

    // Full document: Pokémon Trainer with nested Pokémon
    let mut document_data = HashMap::new();
    document_data.insert("trainer".to_string(), Value::Object(trainer_info));
    document_data.insert(
        "tournament".to_string(),
        Value::Text("Indigo League".to_string()),
    );

    let doc3_id = store.add_document(document_data, Some(4));

    let doc3_data_ref = store.get_document(&doc3_id).unwrap().data.clone();
    store.index_document(&doc3_id, &doc3_data_ref, 4);
    //println!("{:#?}", store.normal_index);

    // println!("{:#?}", store.n_gram_index);
    // let term = "attack";
    // let val = store.get_document("2");
    println!("{:?}", store.get_document("3").unwrap());
    // let results = store.normal_index.search_term(term);

    // println!("Documents containing '{}': {:?}", term, results);
    let mut results2: Vec<String> = Vec::new();
    // let mut results2withfields: Vec<(usize , Vec<String>)> = Vec::new();
    let term2 = "tning";
    if let Some(ref n_index) = store.n_gram_index {
        results2 = n_index.get_terms(term2);
        // results2withfields = n_index.get_terms(term2);
        // use results2 here
    }
    let score = store.normal_index.bm25_search(&["shoutmon"], 1.2, 0.75);
    println!("{:?}", score);

    let k1 = 1.2;
    let b = 0.75;
    let alpha = 0.7; // weight for n-gram overlap
    let beta = 0.3; // weight for edit distance
    let top_k = 5;
    let query = "mon";
    let results = store.ngram_bm25(query, k1, b, alpha, beta, top_k);

    // ----------------
    // Step 5: Print results
    println!("Search results for '{}':", query);
    for (doc_id, score) in results {
        println!("Doc {}: score {:.4}", doc_id, score);
    }
}
