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

    /// Query commands
    Query {
        #[command(subcommand)]
        query: QueryCommands,
    },

    /// Delete by ID
    Delete { id: String },

    /// Print internal stats
    Stats,
}

#[derive(Subcommand)]
enum QueryCommands {
    Get {
        id: String,
    },

    /// Simple OR word search
    Words {
        #[arg(required = true)]
        words: Vec<String>,
    },

    /// AND search (all words must exist)
    And {
        #[arg(required = true)]
        words: Vec<String>,
    },

    /// NOT search (exclude documents containing words)
    Not {
        #[arg(required = true)]
        words: Vec<String>,
    },

    /// N-gram + BM25 fuzzy search
    NgramBm25 {
        query: String,

        #[arg(default_value = "1.2")]
        k1: f64,

        #[arg(default_value = "0.75")]
        b: f64,

        #[arg(default_value = "0.6")]
        alpha: f64,

        #[arg(default_value = "0.4")]
        beta: f64,

        #[arg(default_value = "10")]
        top_k: usize,
    },

    /// Field range query
    Range {
        field: String,
        min: i64,
        max: i64,
    },

    /// field > value
    Gt {
        field: String,
        min: i64,
    },

    /// field >= value
    Gte {
        field: String,
        min: i64,
    },

    /// field < value
    Lt {
        field: String,
        max: i64,
    },

    /// field <= value
    Lte {
        field: String,
        max: i64,
    },

    /// min <= field <= max
    Between {
        field: String,
        min: i64,
        max: i64,
    },
}

//#[derive(Subcommand)]
// enum Commands {
//     /// Add a JSON document
//     Add {
//         file: String,
//         #[arg(default_value = "4")]
//         max_depth: usize,
//     },

//     /// Query words
//     Query { query: String },

//     /// Delete by ID
//     Delete { id: String },

//     /// Print internal stats
//     Stats,
// }

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

            match json {
                // ---- Case 1: Single object ----
                JsonValue::Object(_) => {
                    let map = json_to_value_map(json);
                    let id = engine.add_document(map, Some(max_depth)).unwrap();
                    println!("Added ID: {}", id);
                }

                // ---- Case 2: Array of objects ----
                JsonValue::Array(arr) => {
                    for (idx, item) in arr.into_iter().enumerate() {
                        if !item.is_object() {
                            eprintln!("Skipping index {}: expected object, got {:?}", idx, item);
                            continue;
                        }

                        let map = json_to_value_map(item);
                        let id = engine.add_document(map, Some(max_depth)).unwrap();
                        println!("Added ID: {}", id);
                    }
                }

                // ---- Invalid top-level JSON ----
                _ => {
                    panic!("Input JSON must be an object or an array of objects");
                }
            }
        }

        // Commands::Query { query } => {
        //     let words: Vec<&str> = query.split_whitespace().collect();
        //     let results = engine.query_service().get_words(words);
        //     println!("Matches: {:?}", results);
        // }
        Commands::Delete { id } => {
            engine.delete_document(id.clone()).unwrap();
            println!("Deleted: {}", id);
        }

        Commands::Stats => {
            println!("{:#?}", engine.store());
        }

        Commands::Query { query } => {
            let qs = engine.query_service();

            match query {
                QueryCommands::Get { id } => match qs.get_doc_by_id(&id) {
                    Some(doc) => println!("{:#?}", doc),
                    None => println!("Document not found"),
                },

                QueryCommands::Words { words } => {
                    let refs: Vec<&str> = words.iter().map(String::as_str).collect();
                    let res = qs.get_words(refs);
                    println!("{:#?}", res);
                }

                QueryCommands::And { words } => {
                    let refs: Vec<&str> = words.iter().map(String::as_str).collect();
                    let res = qs.and_word(refs);
                    println!("{:#?}", res);
                }

                QueryCommands::Not { words } => {
                    let refs: Vec<&str> = words.iter().map(String::as_str).collect();
                    let res = qs.not_word(refs);
                    println!("{:#?}", res);
                }

                QueryCommands::NgramBm25 {
                    query,
                    k1,
                    b,
                    alpha,
                    beta,
                    top_k,
                } => {
                    let res = qs.ngram_bm25(&query, k1, b, alpha, beta, top_k);
                    println!("{:#?}", res);
                }

                QueryCommands::Range { field, min, max } => {
                    let res = qs.range_query(&field, min, max);
                    println!("{:#?}", res);
                }

                QueryCommands::Gt { field, min } => {
                    let res = qs.greater_than(&field, min, None);
                    println!("{:#?}", res);
                }

                QueryCommands::Gte { field, min } => {
                    let res = qs.greater_than_equal(&field, min, None);
                    println!("{:#?}", res);
                }

                QueryCommands::Lt { field, max } => {
                    let res = qs.less_than(&field, max, None);
                    println!("{:#?}", res);
                }

                QueryCommands::Lte { field, max } => {
                    let res = qs.less_than_equal(&field, max, None);
                    println!("{:#?}", res);
                }

                QueryCommands::Between { field, min, max } => {
                    let res = qs.between(&field, min, max, None);
                    println!("{:#?}", res);
                }
            }
        }
    }
}
