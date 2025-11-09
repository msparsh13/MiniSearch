use serde::{Serialize, de::DeserializeOwned};
use std::fs::{self, File};
use std::io::{BufReader, BufWriter};
use std::path::Path;

pub struct LocalStore;

impl LocalStore {
    /// Save any serializable data to the given path in pretty JSON format.
    pub fn save<T: Serialize>(data: &T, path: &str) -> std::io::Result<()> {
        let path_ref = Path::new(path);

        if let Some(parent) = path_ref.parent() {
            fs::create_dir_all(parent)?;
        }

        let file = File::create(path_ref)?;
        let writer = BufWriter::new(file);

        serde_json::to_writer_pretty(writer, data)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
    }

    /// Load JSON data from the given path and deserialize it into the provided type.
    pub fn load<T: DeserializeOwned>(path: &str) -> std::io::Result<T> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        serde_json::from_reader(reader)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    /// Check if a file exists at the given path.
    pub fn exists(path: &str) -> bool {
        Path::new(path).exists()
    }

    /// Delete a file if it exists.
    pub fn delete(path: &str) -> std::io::Result<()> {
        let path_ref = Path::new(path);
        if path_ref.exists() {
            fs::remove_file(path_ref)?;
        }
        Ok(())
    }
}
