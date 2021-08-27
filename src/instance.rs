use crate::catalog::Catalog;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
enum InstantiateTagsSpecifier {
    Primary,
    All,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Instance {
    instantiate_tags: InstantiateTagsSpecifier,
    directory_name_space_delimeter: char,
    file_name_pattern: String,
}

pub fn librarian_instantiate(_catalog: &Catalog) {
    // TODO not yet implemented
    // assert!(false);
}
