use clap::{Parser, Subcommand};
use mini_opensearch_api::{
    engine::search_engine::SearchEngine, index::tokenizer::tokenizer::TokenizerConfig,
    index::value::Value,
};
use serde_json::Value as JsonValue;
use std::{collections::HashMap, env, fs};

#[derive(Parser)]
#[command(name = "mysearch")]
#[command(about = "CLI for your Rust search engine")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a JSON document
    Add {
        file: String,
        #[arg(default_value = "4")]
        max_depth: usize,
    },

    /// Query words
    Query { query: String },

    /// Delete by ID
    Delete { id: String },

    /// Print internal stats
    Stats,
}

fn json_to_value_map(json: JsonValue) -> HashMap<String, Value> {
    fn convert(v: &JsonValue) -> Value {
        match v {
            JsonValue::String(s) => Value::Text(s.clone()),
            JsonValue::Number(n) => Value::Number(n.as_f64().unwrap_or(0.0)),
            JsonValue::Bool(b) => Value::Text(b.to_string()),
            JsonValue::Object(map) => {
                let mut obj = HashMap::new();
                for (k, v) in map {
                    obj.insert(k.clone(), convert(v));
                }
                Value::Object(obj)
            }
            JsonValue::Array(arr) => {
                let mut obj = HashMap::new();
                for (i, v) in arr.iter().enumerate() {
                    obj.insert(i.to_string(), convert(v));
                }
                Value::Object(obj)
            }
            _ => Value::Text(v.to_string()),
        }
    }

    match json {
        JsonValue::Object(map) => map.into_iter().map(|(k, v)| (k, convert(&v))).collect(),
        JsonValue::Array(arr) => {
            let mut obj = HashMap::new();
            for (i, v) in arr.into_iter().enumerate() {
                obj.insert(i.to_string(), convert(&v));
            }
            obj
        }
        _ => HashMap::new(),
    }
}

fn main() {
    let cli = Cli::parse();

    let index_path = env::var("INDEX_DIR").unwrap_or_else(|_| "./data/data.json".into());
    let commit_log_path =
        env::var("COMMIT_DIR").unwrap_or_else(|_| "./commit_logs/commits.log".into());
    let snapshots_path =
        env::var("SNAPSHOTS_DIR").unwrap_or_else(|_| "./snapshots_dir/snapshots".into());

    let config = TokenizerConfig {
        use_stemming: false,
        min_ngram: Some(2),
        max_ngram: Some(5),
    };

    let mut engine = SearchEngine::new(index_path, commit_log_path, snapshots_path, Some(config))
        .expect("Failed to open search engine");

    match cli.command {
        Commands::Add { file, max_depth } => {
            let json: JsonValue = serde_json::from_reader(fs::File::open(&file).unwrap()).unwrap();

            let map = json_to_value_map(json);

            let id = engine.add_document(map, Some(max_depth)).unwrap();
            println!("Added ID: {}", id);
        }

        Commands::Query { query } => {
            let words: Vec<&str> = query.split_whitespace().collect();
            let results = engine.query_service().get_words(words);
            println!("Matches: {:?}", results);
        }

        Commands::Delete { id } => {
            engine.delete_document(id.clone()).unwrap();
            println!("Deleted: {}", id);
        }

        Commands::Stats => {
            println!("{:#?}", engine.store());
        }
    }
}
