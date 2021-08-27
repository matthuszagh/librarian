use crate::catalog::Catalog;
use crate::resource::Resource;

use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use regex::Regex;
use std::path::PathBuf;

///
enum MatchType {
    Fuzzy,
    Regex,
}

/// Print the path of resources matching a query.
///
pub fn librarian_search(catalog: &Catalog, resources_path: &PathBuf, r#type: &str, query: &str) {
    match r#type {
        "fuzzy" => {
            librarian_fuzzy_search(catalog, resources_path, query);
        }
        "regex" => {
            // TODO regex search should support a rich query syntax.
            panic!("Regular expression search is not yet implemented.")
        }
        _ => {
            // clap should prevent this from being reached
            assert!(false);
        }
    }

    // TODO remove

    // TODO I'd like to support a full-featured query
    // syntax. Something similar to recoll's query syntax but with
    // regex support.

    // TODO currently, we just use the query string as a regex to
    // search the title

    let re = Regex::new(query).expect("invalid regex query");
    catalog
        .resources
        .iter()
        .filter(|r| re.is_match(&r.title))
        .for_each(|r| {
            println!("{:?}", resources_path.join(&r.historical_checksums[0]));
        });
}

fn librarian_fuzzy_search(catalog: &Catalog, resources_path: &PathBuf, query: &str) {
    // let matcher = SkimMatcherV2::default();
    // let mut matching_resources: Vec<&Resource> = std::vec!();

    // for resource in catalog.resources.iter() {
    //     let fields: [Option<&str>] = [Some(&resource.title)];

    //     if matcher.fuzzy_match(&resource.title, query).is_some() {
    //         matching_resources.push(resource);
    //         continue;
    //     }
    //     for author in resource.authors.iter() {
    //         match author.first {
    //             Some(n) => {
    //                 if matcher.fuzzy_match(&n, query).is_some() {
    //                     matching_resources.push(resource);
    //                     continue;
    //                 }
    //             }
    //         }
    //         match author.middle {
    //             Some(n) => {
    //                 if matcher.fuzzy_match(&n, query).is_some() {
    //                     matching_resources.push(resource);
    //                     continue;
    //                 }
    //             }
    //         }
    //         match author.last {
    //             Some(n) => {
    //                 if matcher.fuzzy_match(&n, query).is_some() {
    //                     matching_resources.push(resource);
    //                     continue;
    //                 }
    //             }
    //         }
    //     }
    // }
}
