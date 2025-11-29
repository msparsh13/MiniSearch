use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Value {
    Text(String),
    Number(f64),
    Date(String),
    Object(HashMap<String, Value>),
}

impl Value {
    pub fn validate(&self) -> Result<(), String> {
        match self {
            Value::Number(_) => Ok(()), // valid
            Value::Text(_) => Ok(()),   // valid
            Value::Date(_) => Ok(()),

            Value::Object(map) => {
                for (key, val) in map {
                    if key.is_empty() {
                        return Err("Field name cannot be empty".into());
                    }
                    val.validate()?;
                }
                Ok(())
            }
        }
    }
}
