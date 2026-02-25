# Opensearch Mini

**Opensearch Mini** is a lightweight, Rust-based search engine for structured JSON documents. It supports multi-field queries, range and fuzzy searches, BM25 + N-gram scoring, multi-field sorting, and a custom query language for advanced filtering.

---

## Features

- **Add/Delete JSON documents** via CLI  
- **Query types**:
  - **Lang** – Custom query language with logical and comparison operators  
  - **Get** – Fetch document by ID  
  - **Words / And / Not** – Boolean word queries  
  - **NgramBm25** – Fuzzy search combining n-grams, edit distance, and BM25 scoring  
  - **Range / Gt / Gte / Lt / Lte / Between** – Numeric field queries using a value tree index  
- **Multi-field Sorting** – Sort by one or more numeric fields, ascending or descending  
- **Internal Stats** – Inspect the document store, forward indices, value tree, and n-gram indices  

---

## Installation

Clone the repository:

```bash
git clone https://github.com/yourusername/opensearch-mini.git
cd opensearch-mini
```

### **Run Tests**
```bash
cargo test
```

## 📂 Example Dataset

Create a file named example.json to test the engine
```json
[
  {
    "trainer": {
      "name": "Ash Ketchum",
      "hometown": "Pallet Town",
      "team": {
        "pikachu": {
          "type": "Electric",
          "generation": 1,
          "stats": { "hp": 35, "attack": 55, "defense": 40 }
        }
      }
    },
    "tournament": "Indigo League"
  },
  {
    "trainer": {
      "name": "Lt. Surge",
      "hometown": "Vermilion City",
      "team": {
        "raichu": {
          "type": "Electric",
          "generation": 1,
          "stats": { "hp": 35, "attack": 90, "defense": 55 }
        }
      }
    },
    "tournament": "Kanto Gym Challenge"
  },
  {
    "trainer": {
      "name": "Misty",
      "hometown": "Cerulean City",
      "team": {
        "starmie": {
          "type": "Water",
          "generation": 1,
          "stats": { "hp": 60, "attack": 75, "defense": 85 }
        }
      }
    },
    "tournament": "Johto League"
  },
  {
    "trainer": {
      "name": "Brock",
      "hometown": "Pewter City",
      "team": {
        "onix": {
          "type": "Rock/Ground",
          "generation": 1,
          "stats": { "hp": 35, "attack": 45, "defense": 160 }
        }
      }
    },
    "tournament": "Kanto Gym Challenge"
  }
]
```

## ⌨️ CLI Usage

#### Add Documents
```bash 
cargo run -- add example.json --max-depth 4
```

## Basic Queries

#### Get by ID
```bash
cargo run -- query get --id "some_doc_id"
```

#### Word Search
```bash
cargo run -- query words --words "Ash Ketchum"
```

#### Logical Queries

```bash
cargo run -- query and --words "Electric Pikachu"
cargo run -- query not --words "Rock Onix"
```
#### Advanced Ranking (Fuzzy N-gram + BM25)

```bash
cargo run -- query ngram-bm25 --query "Pikachu" --k1 1.2 --b 0.75 --alpha 0.6 --beta 0.4 --top-k 10
```

#### Range Queries

```bash
# Specific Range
cargo run -- query range --field "trainer.team.pikachu.stats.hp" --min 30 --max 60
```

#### Greater Than
```bash
cargo run -- query gt --field "trainer.team.onix.stats.attack" --min 40
```

#### Between
```rust
cargo run -- query between --field "trainer.team.starmie.stats.defense" --min 80 --max 90
```

#### View Engine Stats
```bash
cargo run -- stats
```
Displays document store, forward indices, value tree, and n-gram indices.


 ## 🔍 Custom Query Language

The engine supports a structured query string for more expressive searches:

| Keyword / Symbol | Meaning |
|------------------|----------|
| `AND`            | Logical AND between terms |
| `OR`             | Logical OR |
| `NOT`            | Exclude documents |
| `=`, `>`, `>=`, `<`, `<=` | Comparison operators |
| `ASC` / `DESC`   | Sorting order |
| `COUNT`          | Count results |
| `SORT BY`        | Multi-field sorting |



## 🏗 Project Structure

```
opensearch-mini/
├─ src/
│  ├─ engine/          # Core search engine logic (BM25, N-grams)
│  ├─ index/           # Forward index, value tree, and tokenizers
│  ├─ language/        # Query language lexer and processing
│  ├─ query_lang/      # Query parser, sorting, and numeric ops
│  ├─ utils/           # Helper utilities
│  └─ main.rs          # CLI entry point
├─ data/               # Default JSON index storage
├─ commit_logs/        # Persistent logs for document changes
├─ snapshots_dir/      # Index snapshots for recovery
├─ Cargo.toml          # Rust dependencies and config
└─ README.md           # Project documentation
```



## ⚙️ Programmatic Sorting (Rust)

You can perform complex multi-field sorting directly within your Rust 
```rust
let sort_fields = vec![
    SortField { field_path: "trainer.team.onix.stats.hp".to_string(), ascending: true },
    SortField { field_path: "trainer.team.onix.stats.attack".to_string(), ascending: false },
];

let sorted_docs = engine.sort_docs_2(doc_ids, &sort_fields);
println!("{:?}", sorted_docs);
```
