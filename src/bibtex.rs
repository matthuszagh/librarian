use crate::catalog::Catalog;
use crate::resource::{Name, Resource};

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

/// BibTeX entry types.
#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum BibtexType {
    Article,
    Book,
    Manual,
    Miscellaneous,
    Online,
    Patent,
    Report,
    Software,
    TechReport,
}

fn bibtex_serialize_field(field: &str, value: Option<String>) -> String {
    match value {
        Some(v) => {
            let indent = "    ";
            format!("{}{}={{{}}},\n", indent, field, v)
        }
        None => String::new(),
    }
}

/// Serialize a list of names into a BibTeX format.
///
/// # Arguments
///
/// * `field` - Field identifier (e.g., "author").
/// * `names` - Collection of names.
fn bibtex_serialize_names(field: &str, names: Option<Vec<Name>>) -> String {
    match names {
        Some(x) => {
            if x.len() > 0 {
                bibtex_serialize_field(
                    field,
                    Some(
                        x.iter()
                            .map(|n| String::from(n.clone()))
                            .collect::<Vec<String>>()
                            .join(" and "),
                    ),
                )
            } else {
                String::new()
            }
        }
        None => String::new(),
    }
}

impl Resource {
    /// Serialized BibTeX entry of the current resource.
    ///
    /// # Arguments
    ///
    /// * `content_types` - A collection of content types as defined
    /// in the catalog. The map key is a string identifying the
    /// content type and the map value is the associated BibTeX type.
    /// * `resources_path` - Path to resources directory. This is used
    /// to provide the absolute path to the resource.
    pub fn serialize_bibtex(
        &self,
        content_types: &IndexMap<String, BibtexType>,
        resources_path: &PathBuf,
    ) -> String {
        let mut bibtex_entry = String::new();

        match self.bibtex_type(content_types) {
            Some(bt) => {
                let mut bibtex_type_string = serde_json::to_string(&bt).unwrap();
                bibtex_type_string =
                    bibtex_type_string[1..bibtex_type_string.len() - 1].to_string();
                bibtex_entry.push_str(
                    format!(
                        "{}{}{{{},\n",
                        "@",
                        bibtex_type_string.as_str(),
                        self.historical_checksums[0]
                    )
                    .as_str(),
                );
                bibtex_entry.push_str(&bibtex_serialize_field("title", Some(self.title.clone())));
                bibtex_entry.push_str(&bibtex_serialize_field("subtitle", self.subtitle.clone()));
                bibtex_entry.push_str(&bibtex_serialize_names("author", self.author.clone()));
                bibtex_entry.push_str(&bibtex_serialize_names("editor", self.editor.clone()));
                bibtex_entry.push_str(&bibtex_serialize_field(
                    "date",
                    match &self.date {
                        Some(d) => {
                            let mut date = serde_json::to_string(&d).unwrap();
                            date = date[1..date.len() - 1].to_string();
                            Some(date)
                        }
                        None => None,
                    },
                ));
                bibtex_entry.push_str(&bibtex_serialize_field("edition", self.edition.clone()));
                bibtex_entry.push_str(&bibtex_serialize_field("version", self.version.clone()));
                bibtex_entry.push_str(&bibtex_serialize_field("publisher", self.publisher.clone()));
                // Organization is used to populate BibLaTeX's
                // organization and institution fields. The reason is
                // that I don't understand why these are both
                // needed. See the note in the readme.
                bibtex_entry.push_str(&bibtex_serialize_field(
                    "organization",
                    self.organization.clone(),
                ));
                bibtex_entry.push_str(&bibtex_serialize_field(
                    "institution",
                    self.organization.clone(),
                ));
                // TODO remaining fields
                bibtex_entry.push_str(&bibtex_serialize_field(
                    "file",
                    Some(format!(
                        "{}/{}",
                        resources_path
                            .clone()
                            .into_os_string()
                            .into_string()
                            .unwrap(),
                        self.historical_checksums[0],
                    )),
                ));
                bibtex_entry.push_str("}\n");
                bibtex_entry
            }
            None => bibtex_entry,
        }
    }
}

/// Generate BibTeX entries for cataloged resources.
///
/// # Arguments
///
/// * `catalog` - Library catalog.
/// * `resource_path` - Location of the resources directory on the
/// local filesystem.
/// * `bibtex_file_path` - File where BibTeX data should be written. If no
/// file is given, data will be written to stdout.
pub fn librarian_bibtex(
    catalog: &Catalog,
    resources_path: &PathBuf,
    bibtex_file_path: Option<&str>,
) {
    let bibtex_entries: String = catalog
        .resources
        .iter()
        .map(|r| r.serialize_bibtex(&catalog.content_types, resources_path))
        .collect();

    match bibtex_file_path {
        Some(f) => {
            let mut bibtex_file = OpenOptions::new()
                .read(false)
                .write(true)
                .create(true)
                .open(&f)
                .expect("Failed to open or create catalog");
            bibtex_file.write(bibtex_entries.as_bytes()).ok();
        }
        None => {
            println!("{}", bibtex_entries);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bibtex_serialize_names() {
        let mut names: Vec<Name> = vec![
            Name {
                first: Some(String::from("Richard")),
                middle: Some(String::from("Phillips")),
                last: Some(String::from("Feynman")),
            },
            Name {
                first: Some(String::from("Albert")),
                middle: None,
                last: Some(String::from("Einstein")),
            },
            Name {
                first: None,
                middle: None,
                last: Some(String::from("Dirac")),
            },
        ];

        assert!(
            bibtex_serialize_names("author", Some(names.clone()))
                == "    author={Richard Phillips Feynman and Albert Einstein and Dirac},\n"
        );

        names.pop();
        assert!(
            bibtex_serialize_names("editor", Some(names.clone()))
                == "    editor={Richard Phillips Feynman and Albert Einstein},\n"
        );

        names.pop();
        assert!(
            bibtex_serialize_names("annotator", Some(names.clone()))
                == "    annotator={Richard Phillips Feynman},\n"
        );

        names.pop();
        assert!(bibtex_serialize_names("forward", Some(names)) == "");
    }
}
