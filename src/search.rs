use crate::catalog::Catalog;
use crate::resource::Resource;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;

/// Print the path of resources matching a query.
///
pub fn librarian_search(catalog: &Catalog, query: &str) {
    librarian_fuzzy_search(catalog, query);
}

fn librarian_fuzzy_search(catalog: &Catalog, query: &str) {
    let mut matching_resources: Vec<(i64, &Resource)> = std::vec!();
    // TODO I don't like ignoring case, because I'd like it to be
    // considered. However, results with the wrong case seem to be
    // ignored.
    let matcher = SkimMatcherV2::default().ignore_case();

    // TODO I expect there's a more efficient way to do this by
    // inserting each new element into the vector to keep it sorted,
    // rather than inserting all elements and sorting at the end.
    catalog.resources.iter().for_each(|r| {
        let score = matcher.fuzzy_match(
            &r.concat_fields(vec![
                "title",
                "subtitle",
                "author",
                "editor",
                "date",
                "edition",
                "version",
                "publisher",
                "organization",
                "journal",
                "volume",
                "number",
                "part_number",
                "doi",
                "tags",
                "document",
                "content",
                "url",
                "checksum",
                "historical_checksums",
            ]),
            query,
        );
        match score {
            Some(s) => {
                if s > 0 {
                    matching_resources.push((s, r));
                }
            }
            None => (),
        }
    });

    matching_resources.sort_by(|(s1, _), (s2, _)| s2.partial_cmp(&s1).unwrap());
    let resources: Vec<&Resource> =
        matching_resources.iter().map(|(_, r)| r).cloned().collect();

    serde_json::to_writer_pretty(std::io::stdout().lock(), &resources).unwrap();
}
