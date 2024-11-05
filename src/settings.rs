use crate::*;
use strum_macros::{AsRefStr, Display};


#[derive(AsRefStr, Display)]
#[strum(serialize_all = "snake_case")]
pub enum Key {
    DiskPool,
}

fn settings_db() -> sled::Tree {
    database().open_tree("settings").unwrap_or_else(|e| halt!("Could not open settings database tree: {}", e))
}


pub fn set(key: Key, value: &str) {
    let db = settings_db();
    let ron_string = ron::to_string(value).unwrap_or_else(|e| halt!("Could not serialise value {} : {}", value, e));
    db.insert(key.as_ref(), ron_string.as_bytes()).unwrap_or_else(|e| halt!("Could not insert data: {}", e));
    db.flush().unwrap_or_else(|e| halt!("Error flushing db: {}", e));
}

pub fn get(key: Key) -> Option<String> {
    let result = settings_db().get(key.as_ref());
    let option = result.unwrap_or_else(|e| halt!("Could not access database: {}", e));
    if let Some(bytes) = option {
        let string = String::from_utf8_lossy(&bytes);
        Some(ron::from_str(&string).unwrap_or_else(|e| halt!("Error deserializing setting string \"{}\" for key {} : {}", string, key, e)))
    } else {
        None
    }
}

pub fn list() -> Vec<(String, String)> {
    let mut result = Vec::new();
    
    for item in settings_db().iter() {
        let (key, value) = item.unwrap_or_else(|e| halt!("Error iterating settings tree: {}", e));
        if let (Ok(key_str), Ok(value_str)) = (
            std::str::from_utf8(&key),
            std::str::from_utf8(&value)
        ) {
            result.push((key_str.to_owned(), value_str.to_owned()));
        }
    }
    
    result
}