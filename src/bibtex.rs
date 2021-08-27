use crate::catalog::Catalog;

use serde::{Deserialize, Serialize};

/// BibTeX entry types.
#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum BibtexType {
    Article,
    Book,
    Manual,
    Miscellaneous,
    Online,
    TechReport,
}

/// TODO
pub fn librarian_bibliography(catalog: &Catalog, format: &mut String) {
    format.make_ascii_lowercase();
    if *format == String::from("bibtex") {
        librarian_bibtex(catalog);
    } else {
        panic!("invalid bibliography format");
    }
}

// TODO rename
fn librarian_bibtex(_catalog: &Catalog) {
    // TODO
}
