use crate::*;
use strum_macros::AsRefStr;


#[derive(AsRefStr)]
#[strum(serialize_all = "snake_case")]
pub enum Key {
    DiskPool,
}

fn settings_db() -> sled::Tree {
    database().open_tree(b"settings").unwrap_or_else(|e| halt!("Could not open settings database tree: {}", e))
}


pub fn set(key: Key, value: &str) {
    settings_db().insert(key.as_ref(), value).unwrap_or_else(|e| halt!("Could not insert data: {}", e));
}

pub fn get(key: Key) -> Option<String> {
    let result = settings_db().get(key.as_ref());
    let option = result.unwrap_or_else(|e| halt!("Could not access database: {}", e));
    if let Some(bytes) = option {
        Some( std::str::from_utf8(&bytes).unwrap_or_else(|e| halt!("Error deserializing setting string for {} : {}", key.as_ref(), e)).to_owned() )
    } else {
        None
    }
}

pub fn list() -> Vec<(String, String)> {
    let mut result = Vec::new();
    
    for item in settings_db().iter() {
        if let Ok((key, value)) = item {
            if let (Ok(key_str), Ok(value_str)) = (
                std::str::from_utf8(&key),
                std::str::from_utf8(&value)
            ) {
                result.push((key_str.to_owned(), value_str.to_owned()));
            }
        }
    }
    
    result
}