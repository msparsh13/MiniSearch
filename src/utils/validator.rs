use std::collections::HashMap;

use crate::index::value::Value;

pub fn validate_document(data: &HashMap<String, Value>) -> Result<(), String> {
    for (key, val) in data {
        if key.is_empty() {
            return Err("Field name cannot be empty".into());
        }

        val.validate()?; // deep recursive validation
    }
    Ok(())
}
