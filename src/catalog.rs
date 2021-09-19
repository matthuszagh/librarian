use crate::bibtex::BibtexType;
use crate::resource::{DocumentType, Resource};

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use sha1::Sha1;
use std::collections::HashSet;
use std::fs::read;
use std::io::prelude::*;
use std::io::{Read, SeekFrom, Write};
use std::path::PathBuf;
use walkdir::WalkDir;

/// Library catalog contained within the catalog.json file.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Catalog {
    // pub tags: Vec<Tag>,
    pub document_types: IndexMap<String, DocumentType>,
    pub content_types: IndexMap<String, BibtexType>,
    // pub instances: Vec<Instance>,
    pub resources: Vec<Resource>,
}

impl Catalog {
    /// Update the catalog to reflect the current resources.
    ///
    /// This function performs several tasks:
    /// 1. It adds new resources to the catalog.
    /// 2. Updates the checksums of files that have been modified.
    /// 3. Deletes catalog entries no longer backed by a resource (orphans).
    ///
    /// # Arguments
    ///
    /// * `resources` - Checksum and file path for every resource.
    pub fn update(&mut self, resources: &IndexMap<String, PathBuf>) {
        // Create a hashmap of all cataloged resources for fast
        // lookup. The first entry of the hashmap is the initial checksum
        // of the resource, which is used to determine whether a resource
        // has been cataloged. The second entry is the resource itself.
        let mut catalog_resources = IndexMap::<String, Resource>::new();
        // Collection containing the initial checksum of all catalog
        // resources. For each resource, if that resource exists in the
        // catalog we remove it from orphaned catalog entries. The ones
        // that remain after iterating through all resources are the
        // catalog resources that are no longer backed by a resource. We
        // remove these from the catalog.
        let mut orphaned_catalog_resources = HashSet::<String>::new();
        for resource in &self.resources {
            catalog_resources.insert(resource.historical_checksums[0].clone(), resource.clone());
            orphaned_catalog_resources.insert(resource.historical_checksums[0].clone());
        }

        // Catalog each new resource or update the checksum if the
        // resource's contents have changed.
        for (checksum, resource_path) in resources {
            let file_name = resource_path
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string();
            match catalog_resources.get_mut(&file_name) {
                // update the checksum if it's changed
                Some(r) => {
                    let new_checksum = checksum.to_string();
                    if r.checksum != new_checksum {
                        r.historical_checksums.push(new_checksum.clone());
                        r.checksum = new_checksum;
                    }
                    orphaned_catalog_resources.remove(&file_name);
                }
                None => {
                    // rename the file to the current SHA-1 contents
                    let checksum = checksum.to_string();
                    let new_file_path = resource_path.parent().unwrap().join(checksum.clone());
                    std::fs::rename(resource_path, new_file_path.clone()).unwrap();

                    catalog_resources.insert(
                        checksum.clone(),
                        Resource {
                            title: file_name,
                            authors: std::vec!(),
                            datetime: None,
                            version: None,
                            publisher: None,
                            organization: None,
                            tags: std::vec!(),
                            document_type: None,
                            content_type: None,
                            url: None,
                            checksum: checksum.clone(),
                            historical_checksums: std::vec!(checksum),
                        },
                    );
                }
            }
        }

        // remove cataloged resources that are no longer in the resources
        // directory
        for resource in orphaned_catalog_resources.iter() {
            catalog_resources.remove(resource);
        }

        self.resources = catalog_resources.values().cloned().collect();

        // sort resources by title in alphanumeric order
        self.resources
            .sort_by(|a, b| a.title.partial_cmp(&b.title).unwrap());

        self.content_types.sort_keys();
        self.document_types.sort_keys();
    }

    /// Reads a catalog from a file into a `Catalog` instance.
    ///
    /// If the catalog doesn't exist, this function will initialize it to
    /// an empty catalog with the correct structure.
    /// TODO
    pub fn read_from_file(catalog_file: &mut std::fs::File) -> Catalog {
        let mut catalog_contents = String::new();
        catalog_file
            .read_to_string(&mut catalog_contents)
            .expect("failed to read catalog file into a string");

        // initialize the catalog file if it's empty
        if catalog_contents == "" {
            let new_catalog_contents = concat!(
                "{\n",
                // "  \"tags\": [],\n",
                "  \"document_types\": {},\n",
                "  \"content_types\": {},\n",
                // "  \"instances\": [],\n",
                "  \"resources\": []\n",
                "}",
            );
            catalog_file.write(new_catalog_contents.as_bytes()).unwrap();
            // catalog_contents needs the current valid file contents to parse json
            catalog_contents = new_catalog_contents.to_string();
        }

        let catalog: Catalog = serde_json::from_str(&catalog_contents).unwrap();
        catalog
    }
}

/// Clear the contents of a file.
fn clear_file(file: &mut std::fs::File) {
    file.set_len(0).unwrap();
    file.seek(SeekFrom::Start(0)).unwrap();
}

/// Compute a SHA1 checksum of a directory.
///
/// The checksum incorporates the contents of all files in the
/// directory as well as the path and name of every file relative to
/// the directory. That is, two otherwise identical directories at
/// different locations in the filesystem would yield the same
/// checksum, but any difference in the contents of the directory
/// would result in a different checksum.
fn directory_recursive_sha1(directory_path: &PathBuf) -> Sha1 {
    let mut directory_content = Vec::<u8>::new();

    for f in WalkDir::new(directory_path)
        .min_depth(1)
        .sort_by_file_name()
        .into_iter()
    {
        let f = f.unwrap();
        // First, append the file name to the directory content vector.
        directory_content.append(&mut Vec::<u8>::from(
            f.path()
                .strip_prefix(directory_path)
                .unwrap()
                .clone()
                .to_str()
                .unwrap(),
        ));
        // Then, if the file is a file type, also append its contents.
        if f.path().is_file() {
            directory_content.append(&mut read(f.path()).unwrap());
        }
    }
    let mut sha = Sha1::new();
    sha.update(&directory_content);
    sha
}

/// Register new resources and update the checksum of existing
/// resources.
pub fn librarian_catalog(
    catalog_file: &mut std::fs::File,
    catalog: &mut Catalog,
    resources_path: &PathBuf,
) {
    // Construct a hashmap of the SHA-1 checksum and path of each
    // resource.
    let mut resources = IndexMap::<String, PathBuf>::new();
    WalkDir::new(resources_path)
        .min_depth(1)
        .max_depth(1)
        .into_iter()
        .for_each(|f| {
            let file = f.unwrap();

            let content_sha: String;
            if file.file_type().is_dir() {
                content_sha = directory_recursive_sha1(&file.clone().into_path())
                    .digest()
                    .to_string();
            } else {
                let file_contents = read(file.path()).expect("failed to read file");
                let mut sha = sha1::Sha1::new();
                sha.update(&file_contents);
                content_sha = sha.digest().to_string();
            }

            // If a resource exists with identical content to the
            // current resource, delete the current resource.
            if resources.contains_key(&content_sha) {
                std::fs::remove_file(file.path()).unwrap();
            } else {
                resources.insert(content_sha, file.clone().path().to_path_buf());
            }
        });

    catalog.update(&resources);

    // write new catalog contents to file
    clear_file(catalog_file);
    serde_json::to_writer_pretty(catalog_file, &catalog).unwrap();
}
