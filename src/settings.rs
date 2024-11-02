use crate::*;
use strum_macros::AsRefStr;


#[derive(AsRefStr)]
#[strum(serialize_all = "snake_case")]
pub enum Key {
    DiskPool,
}


pub fn set(key: Key, value: &str) {
    open_database().insert(&format!("settings::{}", key.as_ref()), value).unwrap_or_else(|e| halt!("Could not insert data: {}", e));
}

pub fn get(key: Key) -> Option<String> {
    let result = open_database().get(&format!("settings::{}", key.as_ref()));
    let option = result.unwrap_or_else(|e| halt!("Could not access database: {}", e));
    if let Some(bytes) = option {
        Some( std::str::from_utf8(&bytes).unwrap_or_else(|e| halt!("Error deserializing setting string for {} : {}", key.as_ref(), e)).to_owned() )
    } else {
        None
    }
}