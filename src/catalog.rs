use crate::bibtex::BibtexType;
use crate::cache::{read_cache_from_file, CacheFields};
use crate::resource::{DocumentType, Resource};

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use sha1::Sha1;
use std::cmp::Ordering;
use std::collections::HashSet;
use std::fs::{read, OpenOptions};
use std::io::{prelude::*, Read, SeekFrom, Write};
use std::path::PathBuf;
use std::time::SystemTime;
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
    /// This function performs several tasks. It:
    /// 1. Adds new resources to the catalog.
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
                            subtitle: None,
                            author: None,
                            editor: None,
                            date: None,
                            edition: None,
                            version: None,
                            publisher: None,
                            organization: None,
                            journal: None,
                            volume: None,
                            number: None,
                            doi: None,
                            tags: None,
                            document: None,
                            content: None,
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

        // Sort resources by title in alphanumeric order and by
        // datetime, when the title results in a tie.
        self.resources.sort_by(|a, b| {
            let title_cmp = a.title.partial_cmp(&b.title).unwrap();
            if title_cmp == Ordering::Equal {
                a.date.partial_cmp(&b.date).unwrap()
            } else {
                title_cmp
            }
        });

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
    let mut sha = Sha1::new();

    for f in WalkDir::new(directory_path)
        .min_depth(1)
        .sort_by_file_name()
        .into_iter()
    {
        let f = f.unwrap();
        // First, incorporate the file name.
        sha.update(
            f.path()
                .strip_prefix(directory_path)
                .unwrap()
                .clone()
                .to_str()
                .unwrap()
                .as_bytes(),
        );
        // Then, if the file is a file type, also incorporate its
        // contents.
        if f.path().is_file() {
            sha.update(&read(f.path()).unwrap());
        }
    }
    sha
}

/// Compute the checksum of a file or directory.
///
/// # Arguments
///
/// * `file_or_dir` - File or directory for which the checksum should
/// be computed.
fn sha1(file_or_dir: &walkdir::DirEntry) -> String {
    let content_sha: String;
    if file_or_dir.file_type().is_dir() {
        content_sha = directory_recursive_sha1(&file_or_dir.clone().into_path())
            .digest()
            .to_string();
    } else {
        let sha = sha1::Sha1::from(&read(file_or_dir.path()).expect("failed to read file"));
        content_sha = sha.digest().to_string();
    }
    content_sha
}

/// Register new resources and update the checksum of existing
/// resources.
///
/// # Arguments
///
/// * `use_cache` - If `true`, only compute the checksum of resources
/// modified more recently than the last time their checksum was
/// verified as reported by the cache file. If `false`, the checksum
/// of all resources will be computed, but the cache file will still
/// be updated.
pub fn librarian_catalog(
    catalog_file: &mut std::fs::File,
    catalog: &mut Catalog,
    resources_path: &PathBuf,
    use_cache: bool,
) {
    // Construct the cache object from the cache file. This is
    // necessary regardless of whether we use this file to avoid
    // computing checksums because we will still need to update the
    // cache with the last time the checksum of each resource was
    // verified.
    let mut cache_file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(
            resources_path
                .parent()
                .expect("resources path does not have a parent")
                .join(".cache"),
        )
        .expect("Failed to open or create catalog");
    let mut cache = read_cache_from_file(&mut cache_file);

    // `SystemTime` is used to calculate the number of seconds since
    // "the epoch". This will work regardless of your local timezone.
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // When we iterate through all resources we remove each resource
    // from `cache_orphans`. The entries that remain after iterating
    // through all resources are "orphans" (i.e., not backed by a
    // resource) and should be removed from the cache.
    let mut cache_orphans = cache.clone();

    // Construct a hashmap of the SHA-1 checksum and path of each
    // resource.
    let mut resources = IndexMap::<String, PathBuf>::new();
    WalkDir::new(resources_path)
        .min_depth(1)
        .max_depth(1)
        .into_iter()
        .for_each(|f| {
            let file = f.unwrap();
            let file_name: String = file.file_name().to_str().unwrap().to_string();

            cache_orphans.remove(&file_name);

            let mut cache_invalid = false;
            let mut cache_checksum = String::new();

            // If the resource's checksum was verified more recently
            // than the resource was modified, use that catalog
            // checksum. Otherwise, recompute the checksum and update
            // the cache verification time.
            match use_cache {
                true => match cache.get(&file_name) {
                    Some(cache_data) => match file.metadata() {
                        Ok(m) => match m.modified() {
                            Ok(modified) => {
                                if modified
                                    .duration_since(SystemTime::UNIX_EPOCH)
                                    .unwrap()
                                    .as_secs()
                                    > cache_data.last_verified
                                {
                                    cache_invalid = true;
                                } else {
                                    cache_checksum = cache_data.checksum.clone();
                                }
                            }
                            Err(_) => {
                                cache_invalid = true;
                            }
                        },
                        Err(_) => {
                            cache_invalid = true;
                        }
                    },
                    None => {
                        cache_invalid = true;
                    }
                },
                false => {
                    cache_invalid = true;
                }
            }

            let content_sha: String = match cache_invalid {
                true => {
                    let checksum = sha1(&file);
                    let mut cache_key = file_name;
                    // If the cache does not contain an entry whose
                    // key is the file name, then the entry
                    // corresponds to a new resource and the index
                    // should be set to the checksum, not the old file
                    // name. This is necessary because the file is not
                    // renamed to the initial checksum until we call
                    // `catalog.update`.
                    if !cache.contains_key(&cache_key) {
                        cache_key = checksum.clone();
                    }
                    // insert updates an existing key if it already exists
                    cache.insert(
                        cache_key,
                        CacheFields {
                            last_verified: now,
                            checksum: checksum.clone(),
                        },
                    );
                    checksum
                }
                false => cache_checksum,
            };

            // If a resource exists with identical content to the
            // current resource, delete the current resource.
            if resources.contains_key(&content_sha) {
                std::fs::remove_file(file.path()).unwrap();
            } else {
                resources.insert(content_sha, file.clone().path().to_path_buf());
            }
        });

    // remove all orphans from the cache
    cache_orphans.iter().for_each(|o| {
        cache.remove(o.0);
    });

    cache.sort_by(|a_key, _, b_key, _| a_key.partial_cmp(&b_key).unwrap());

    // write new cache contents to file
    clear_file(&mut cache_file);
    serde_json::to_writer_pretty(&mut cache_file, &cache).unwrap();

    // update catalog and write it to disk
    catalog.update(&resources);
    clear_file(catalog_file);
    serde_json::to_writer_pretty(catalog_file, &catalog).unwrap();
}
