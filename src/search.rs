use crate::catalog::Catalog;
use crate::resource::Resource;

/// Print the path of resources matching a query.
///
pub fn librarian_search(catalog: &Catalog, query: &str) {
    librarian_fuzzy_search(catalog, query);
}

fn librarian_fuzzy_search(catalog: &Catalog, query: &str) {
    let mut matching_resources: Vec<&Resource> = std::vec!();

    catalog.resources.iter().for_each(|r| {
        if r.fuzzy_match(query) {
            matching_resources.push(r);
        }
    });

    serde_json::to_writer_pretty(std::io::stdout().lock(), &matching_resources).unwrap();
}
