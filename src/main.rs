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
 * make two mods schema less and with schema
 *  we can add more complex function like search attribute names not search them to add flexibility :fixed
 * now time to make complex queries > < smth : fixed
 */
fn main() {
    // tokenizer config (ngrams/stemming)
    let config = TokenizerConfig {
        use_stemming: false,
        min_ngram: Some(2),
        max_ngram: Some(5),
    };

    let mut store = DocumentStore::new(Some(config));

    // Small helper to add & index a document without borrow issues
    fn add_and_index(
        store: &mut DocumentStore,
        data: HashMap<String, Value>,
        max_depth: usize,
    ) -> String {
        let id = store.add_document(data, Some(max_depth));
        // clone data for indexing to avoid borrowing the store while indexing
        if let Some(doc) = store.get_document(&id) {
            let doc_data = doc.data.clone();
            store.index_document(&id, &doc_data, max_depth);
        }
        id
    }

    // --- Document 1
    let mut doc1 = HashMap::new();
    doc1.insert(
        "text".to_string(),
        Value::Text("Rust programming is fun".to_string()),
    );
    let doc1_id = add_and_index(&mut store, doc1, 2);
    println!("Added doc1 id = {}", doc1_id);

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
    let doc2_id = add_and_index(&mut store, doc2, 4);
    println!("Added doc2 id = {}", doc2_id);

    // Print a snapshot of the inverted index (debug)
    println!("Normal inverted index snapshot:\n{:#?}", store.normal_index);

    // --- Document 3 (trainer + nested Pokémon)
    // build Pikachu object
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
    let doc3_id = add_and_index(&mut store, doc3, 4);
    println!("Added doc3 id = {}", doc3_id);

    // show ngram index presence (Option)
    println!("N-gram index enabled? {}", store.n_gram_index.is_some());
    println!("N-gram trie enabled? {}", store.n_gram_trie.is_some());

    // --- Basic term search
    let term = "attack";
    let docs_with_term = store.normal_index.search_term(&[term]);
    println!("Documents containing '{}': {:?}", term, docs_with_term);

    // --- N-gram BM25 search example
    let k1 = 1.2;
    let b = 0.75;
    let alpha = 0.7;
    let beta = 0.3;
    let top_k = 5;
    let query = "mon"; // should match "pokemon"/"digimon" etc. via n-gram
    let ngram_results = store.ngram_bm25(query, k1, b, alpha, beta, top_k);

    println!("ngram_bm25 results for query '{}':", query);
    if ngram_results.is_empty() {
        println!("  (no results — either ngram disabled or no matches)");
    } else {
        for (doc_id, score) in ngram_results {
            println!("  doc {} => score {:.6}", doc_id, score);
        }
    }

    // --- Range queries (numeric/date)
    // Note: ValueTreeIndex stores numbers as value * 1000 (see implementation).
    // Query attributes.year between 2020 and 2030 (example).
    let year_min = 2020 * 1000;
    let year_max = 2030 * 1000;
    let year_matches = store.range_query("attributes.year", year_min, year_max);
    println!(
        "Documents with 'attributes.year' in [2020..2030]: {:?}",
        year_matches
    );

    // Query nested numeric: generation inside trainer.team.pikachu.generation
    // depending on how you stored field paths, you may need the exact field path.
    // We'll attempt a few plausible prefixes (demo)
    let possible_paths = [
        "trainer.team.pikachu.generation",
        "trainer.team.pikachu.stats.generation",
        "trainer.generation",
        "generation",
    ];

    for path in &possible_paths {
        let matches = store.range_query(path, 1 * 1000, 1 * 1000);
        if !matches.is_empty() {
            println!("Found generation=1 at path '{}': {:?}", path, matches);
        }
    }

    // --- Demonstrate search helpers
    // not_word example: returns doc ids that do NOT contain given words
    let excluded = store.not_word(vec!["rust"]);
    println!("Documents that do NOT contain 'rust': {:?}", excluded);

    // get_words example (wraps search_term and returns doc ids)
    let ids_for_attack = store.get_words(vec!["attack"]);
    println!("get_words for 'attack' -> {:?}", ids_for_attack);

    println!("Finished checks. Inspect printed structures above for correctness.");
}
