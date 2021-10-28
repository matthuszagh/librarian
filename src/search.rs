use crate::catalog::Catalog;
use crate::resource::Resource;

/// Print the path of resources matching a query.
///
pub fn librarian_search(catalog: &Catalog, query: &str) {
    librarian_fuzzy_search(catalog, query);
}

fn librarian_fuzzy_search(catalog: &Catalog, query: &str) {
    let mut matching_resources: Vec<(i64, &Resource)> = std::vec!();

    // TODO I expect there's a more efficient way to do this by
    // inserting each new element into the vector to keep it sorted,
    // rather than inserting all elements and sorting at the end.
    catalog.resources.iter().for_each(|r| {
        let score = r.fuzzy_match(query);
        if score > 0 {
            matching_resources.push((score, r));
        }
    });

    matching_resources.sort_by(|(s1, _), (s2, _)| s2.partial_cmp(&s1).unwrap());
    let resources: Vec<&Resource> = matching_resources.iter().map(|(_, r)| r).cloned().collect();

    serde_json::to_writer_pretty(std::io::stdout().lock(), &resources).unwrap();
}
