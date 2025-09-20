mod index;

use std::collections::HashMap;
use crate::index::documents_store::{DocumentStore, Value};


/**
 * TODO: Objects within objects not being read make them read by inverted index
 */
fn main() {
    let mut store = DocumentStore::new(false);

    let mut doc1_data = HashMap::new();
    doc1_data.insert("text".to_string(), Value::Text("Rust programming is fun".to_string()));
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
    let doc2_id = store.add_document(doc2_data, Some(2));

    let doc2_data_ref = store.get_document(&doc2_id).unwrap().data.clone();
    store.index_document(&doc2_id, &doc2_data_ref, 2);
    println!("{:#?}", store.normal_index);

    let term = "Mew";
    let val = store.get_document("2");
    println!("{:?}", store.get_document("2").unwrap());
    let results = store.normal_index.search_term(term);
    println!("Documents containing '{}': {:?}", term, results);
}
