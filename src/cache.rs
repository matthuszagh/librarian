use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::prelude::*;

/// Data stored in the cache for each resource.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CacheFields {
    /// Number of seconds since "the epoch".
    pub last_verified: u64,
    pub checksum: String,
}

/// Reads a cache from a file into a `Cache` instance.
///
/// If the catalog doesn't exist, this function will initialize it to
/// an empty cache with the correct structure.
///
/// # Arguments
///
/// * `cache_file` - Cache file.
///
/// # Returns
///
/// The cache as an `IndexMap` where the key is a string of the file
/// name and the value is the `CacheFields` corresponding to that
/// resource.
pub fn read_cache_from_file(
    cache_file: &mut File,
) -> IndexMap<String, CacheFields> {
    let mut cache_contents = String::new();
    cache_file
        .read_to_string(&mut cache_contents)
        .expect("failed to read cache file into a string");

    // initialize the catalog file if it's empty
    if cache_contents == "" {
        let new_cache_contents = concat!("{\n", "}",);
        cache_file.write(new_cache_contents.as_bytes()).unwrap();
        // cache_contents needs the current valid file contents to parse json
        cache_contents = new_cache_contents.to_string();
    }

    let cache: IndexMap<String, CacheFields> =
        serde_json::from_str(&cache_contents).unwrap();
    cache
}
