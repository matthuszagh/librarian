use clap::{app_from_crate, App, Arg};
use regex::Regex;
use serde::{Deserialize, Serialize};
use sha1::Sha1;
use std::collections::{HashMap, HashSet};
use std::fs::{read, OpenOptions};
use std::io::prelude::*;
use std::io::SeekFrom;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::string::String;
use std::vec::Vec;
use std::env;
use walkdir::WalkDir;

/// Name.
#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
struct Name {
    first: Option<String>,
    middle: Option<String>,
    last: Option<String>,
}

/// Date.
#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
struct Date {
    year: Option<i32>,
    month: Option<i32>,
    day: Option<i32>,
}

/// Library "resource". This represents one unit of library content,
/// which can either be a file (such as a document or video), or a
/// directory (e.g., containing the contents of a webpage).
#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
struct Resource {
    /// Title.
    title: String,
    /// Collection of all resource authors.
    authors: Vec<Name>,
    /// Date of publication or creation. For a website this is the
    /// date the website was last modified or archived. Sometimes it
    /// is not possible to know when a website was last modified and
    /// the archival date should be used instead.
    date: Date,
    edition: Option<i32>,
    version: Option<String>,
    publisher: Option<String>,
    organization: Option<String>,
    tags: Vec<String>,
    /// Current SHA-1 checksum.
    checksum: String,
    /// An ordered collection (oldest to most recent) of all previous
    /// and current checksums of a resource. The current checksum is
    /// the last item in the container.
    historical_checksums: Vec<String>,
    /// Document type (when applicable). This field is also used to
    /// associate a resource with a file extension.
    document_type: Option<String>,
    resource_type: Option<String>,
}

/// Library "tag".
#[derive(Serialize, Deserialize, Debug, Clone)]
struct Tag {}

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

/// BibTeX entry types.
#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
enum BibtexType {
    Article,
    Book,
    Manual,
    Miscellaneous,
    Online,
    TechReport,
}

/// Resource type.
#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
struct ResourceType {
    /// Resource name. This must match the value of "resource_type"
    /// for each resource.
    name: String,
    /// BibTeX type associated with this resource type. This is used
    /// when exporting the resource to a BibTeX entry.
    bibtex: BibtexType,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct DocumentType {
    name: String,
    extension: String,
}

/// Library catalog contained within the catalog.json file.
#[derive(Serialize, Deserialize, Debug, Clone)]
struct Catalog {
    tags: Vec<Tag>,
    document_types: Vec<DocumentType>,
    resource_types: Vec<ResourceType>,
    instances: Vec<Instance>,
    resources: Vec<Resource>,
}

fn main() {
    let args = parse_app_args();
    let (resources_path, catalog_path) = library_paths(&args);
    let mut catalog_file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(&catalog_path)
        .expect("Failed to open or create catalog");
    let mut catalog = read_catalog(&mut catalog_file);

    // Invoke the function for the appropriate subcommand. If no
    // subcommand is given, perform "register" followed by
    // "instantiate".
    if args.is_present("register") {
        librarian_register(&mut catalog_file, &mut catalog, &resources_path);
    } else if args.is_present("instantiate") {
        librarian_instantiate(&catalog);
    } else if args.is_present("search") {
        librarian_search(
            &catalog,
            &resources_path,
            &String::from(
                args.subcommand_matches("search")
                    .unwrap()
                    .value_of("query")
                    .expect("must provide a search query"),
            ),
        );
    } else if args.is_present("bibliography") {
        librarian_bibliography(
            &catalog,
            &mut String::from(
                args.subcommand_matches("bibliography")
                    .unwrap()
                    .value_of("format")
                    .expect("must provide a search query"),
            ),
        );
    } else {
        // when no subcommand is provided, register all new files and
        // instantiate all directories
        librarian_register(&mut catalog_file, &mut catalog, &resources_path);
        librarian_instantiate(&catalog);
    }
}

/// Parse and return command line arguments.
fn parse_app_args() -> clap::ArgMatches {
    app_from_crate!()
        .arg(
            Arg::new("directory")
                .about("library directory path")
                .long_about("TODO")
                .takes_value(true)
                .short('d')
                .long("directory")
                .default_value(
                    env::current_dir()
                        .expect("unable to get current working directory")
                        .into_os_string()
                        .into_string()
                        .expect("current working directory is not valid UTF-8")
                        .as_str(),
                ),
        )
        .arg(
            Arg::new("catalog")
                .about("library catalog file, relative to the library directory path")
                .long_about("TODO")
                .takes_value(true)
                .short('c')
                .long("catalog")
                .default_value("catalog.json")
        )
        .arg(
            Arg::new("resources")
                .about("resources directory, relative to the library directory path")
                .long_about("TODO")
                .takes_value(true)
                .short('r')
                .long("resources")
                .default_value("resources")
        )
        .subcommand(
            App::new("register")
                .about("registers all new original resources and adds information about them to the catalog")
                .long_about("TODO")
        )
        .subcommand(
            App::new("instantiate")
                .about("instantiates one or more instances from the catalog")
                .long_about("TODO")
        )
        .subcommand(
            App::new("validate")
                .about("validates the catalog")
                .long_about("TODO")
        )
        .subcommand(
            App::new("search")
                .about("get the path of a resource from information about it")
                .long_about("TODO")
                .arg(
                    Arg::new("query")
                        .about("resource query")
                        .takes_value(true)
                )
        )
        .subcommand(
            App::new("bibliography")
                .about("generate a bibliography")
                .long_about("Generates a bibliography in a specified format and optionally restricted to a subset of catalog entries.")
                .arg(
                    Arg::new("format")
                        .about("bibliographic format")
                        .takes_value(true)
                        .default_value("bibtex")
                        .possible_values(&["bibtex"])
                )
        )
        .get_matches()
}

/// Get the resources directory path and catalog file path according to
/// the user's command line arguments.
fn library_paths(args: &clap::ArgMatches) -> (PathBuf, PathBuf) {
    let directory: PathBuf = PathBuf::from(
        args.value_of("directory")
            .expect("failed to retrieve directory argument"),
    )
    .canonicalize()
    .expect("failed to resolve an absolute path from the specified directory path");

    let resources_directory = directory.join(
        args.value_of("resources")
            .expect("failed to retrieve resources argument"),
    );

    // read and parse catalog file contents
    let catalog_path = directory.join(
        args.value_of("catalog")
            .expect("failed to retrieve catalog argument"),
    );

    (resources_directory, catalog_path)
}

/// Reads the library catalog into a Catalog instance.
///
/// If the catalog doesn't exist, this function will initialize it to
/// an empty catalog with the correct structure.
/// TODO
fn read_catalog(catalog_file: &mut std::fs::File) -> Catalog {
    let mut catalog_contents = String::new();
    catalog_file
        .read_to_string(&mut catalog_contents)
        .expect("failed to read catalog file into a string");

    // initialize the catalog file if it's empty
    if catalog_contents == "" {
        let new_catalog_contents = concat!(
            "{\n",
            "  \"tags\": [],\n",
            "  \"extensions\": [],\n",
            "  \"instances\": [],\n",
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

/// Clear the contents of a file.
fn clear_file(file: &mut std::fs::File) {
    file.set_len(0).unwrap();
    file.seek(SeekFrom::Start(0)).unwrap();
}

/// Register new resources and update the checksum of existing
/// resources.
fn librarian_register(
    catalog_file: &mut std::fs::File,
    catalog: &mut Catalog,
    resources_path: &PathBuf,
) {
    // Construct a hashmap of the SHA-1 checksum and path of each
    // resource.
    let mut resources = HashMap::<String, PathBuf>::new();
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

    update_catalog(catalog, &resources);

    // write new catalog contents to file
    clear_file(catalog_file);
    serde_json::to_writer_pretty(catalog_file, &catalog).unwrap();
}

fn librarian_instantiate(_catalog: &Catalog) {
    // TODO not yet implemented
    // assert!(false);
}

/// Print the path of resources matching a query.
/// TODO
fn librarian_search(catalog: &Catalog, resources_path: &PathBuf, query: &String) {
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

/// TODO
fn librarian_bibliography(catalog: &Catalog, format: &mut String) {
format.make_ascii_lowercase();
    if *format == String::from("bibtex") {
        librarian_bibtex(catalog);
    }
    else {
        panic!("invalid bibliography format");
    }
}

fn librarian_bibtex(catalog: &Catalog) {
    // TODO
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

/// Update the catalog to reflect the current resources.
///
/// This function performs several tasks:
/// 1. It adds new resources to the catalog.
/// 2. Updates the checksums of files that have been modified.
/// 3. Deletes catalog entries no longer backed by a resource (orphans).
///
/// # Arguments
///
/// * `catalog` - Library catalog.
/// * `resources` - Checksum and file path for every resource.
fn update_catalog(catalog: &mut Catalog, resources: &HashMap<String, PathBuf>) {
    // Create a hashmap of all cataloged resources for fast
    // lookup. The first entry of the hashmap is the initial checksum
    // of the resource, which is used to determine whether a resource
    // has been cataloged. The second entry is the resource itself.
    let mut catalog_resources = HashMap::<String, Resource>::new();
    // Collection containing the initial checksum of all catalog
    // resources. For each resource, if that resource exists in the
    // catalog we remove it from orphaned catalog entries. The ones
    // that remain after iterating through all resources are the
    // catalog resources that are no longer backed by a resource. We
    // remove these from the catalog.
    let mut orphaned_catalog_resources = HashSet::<String>::new();
    for resource in &catalog.resources {
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
                        authors: std::vec!(Name {
                            first: None,
                            middle: None,
                            last: None,
                        }),
                        date: {
                            Date {
                                year: None,
                                month: None,
                                day: None,
                            }
                        },
                        edition: None,
                        version: None,
                        publisher: None,
                        organization: None,
                        tags: std::vec!(),
                        checksum: checksum.clone(),
                        historical_checksums: std::vec!(checksum),
                        document_type: None,
                        resource_type: None,
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
    catalog.resources = catalog_resources.values().cloned().collect();
    // sort resources by title in alphanumeric order
    catalog
        .resources
        .sort_by(|a, b| a.title.partial_cmp(&b.title).unwrap());
}
