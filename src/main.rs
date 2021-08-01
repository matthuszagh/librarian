use clap::{app_from_crate, App, Arg};
use serde::{Deserialize, Serialize};
use sha1::Sha1;
use std::collections::HashMap;
use std::env;
use std::fs::{read, OpenOptions};
use std::io::prelude::*;
use std::io::SeekFrom;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::string::String;
use std::vec::Vec;
use walkdir::WalkDir;

/// TODO
#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
struct Name {
    first: Option<String>,
    middle: Option<String>,
    last: Option<String>,
}

/// TODO
#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
struct Date {
    year: Option<i32>,
    month: Option<i32>,
    day: Option<i32>,
}

/// TODO
#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
enum ResourceType {
    Document,
    Website,
}

/// Library "resource". This represents one unit of library content,
/// which can either be a file (such as a document or video), or a
/// directory (e.g., containing the contents of a webpage).
#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
struct Resource {
    title: String,
    authors: Vec<Name>,
    date: Date,
    edition: Option<i32>,
    publisher: Option<String>,
    organization: Option<String>,
    tags: Vec<String>,
    checksum: String,
    historical_checksums: Vec<String>,
    resource_type: Option<ResourceType>,
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

/// Library "index" specified by the config file.
#[derive(Serialize, Deserialize, Debug, Clone)]
struct LibraryIndex {
    tags: Vec<Tag>,
    instances: Vec<Instance>,
    resources: Vec<Resource>,
}

fn main() {
    let args = parse_app_args();
    let (resources_directory_path, config_file_path) = library_paths(&args);
    let mut config_file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(&config_file_path)
        .expect("Failed to open or create config file");
    let mut library_index = config_file_library_index(&mut config_file);

    if args.is_present("register") {
        librarian_register(
            &mut config_file,
            &mut library_index,
            &resources_directory_path,
        );
    } else if args.is_present("instantiate") {
        librarian_instantiate(&library_index);
    } else {
        // when no subcommand is provided, register all new files and instantiate all directories
        librarian_register(
            &mut config_file,
            &mut library_index,
            &resources_directory_path,
        );
        librarian_instantiate(&library_index);
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
            Arg::new("config")
                .about("library config file, relative to the library directory path")
                .long_about("TODO")
                .takes_value(true)
                .short('c')
                .long("config")
                .default_value("config.json")
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
                .about("registers all new original resources and adds information about them to the configuration file")
                .long_about("TODO")
        )
        .subcommand(
            App::new("instantiate")
                .about("instantiates one or more instances from the configuration file")
                .long_about("TODO")
        )
        .get_matches()
}

/// Get the resources directory path and config file path according to
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

    // read and parse config file contents
    let config_file_path = directory.join(
        args.value_of("config")
            .expect("failed to retrieve config argument"),
    );

    (resources_directory, config_file_path)
}

/// JSON value of the config file contents. This creates and properly
/// initializes the config file if it doesn't exist or is empty.
fn config_file_library_index(config_file: &mut std::fs::File) -> LibraryIndex {
    let mut config_contents = String::new();
    config_file
        .read_to_string(&mut config_contents)
        .expect("failed to read config file into a string");

    // initialize the config file if it's empty
    if config_contents == "" {
        let new_config_contents = concat!(
            "{\n",
            "    \"tags\": [],\n",
            "\n",
            "    \"instances\": [],\n",
            "\n",
            "    \"resources\": []\n",
            "}",
        );
        config_file.write(new_config_contents.as_bytes()).unwrap();
        // config_contents needs the current valid file contents to parse json
        config_contents = new_config_contents.to_string();
    }

    let library_index: LibraryIndex = serde_json::from_str(&config_contents).unwrap();
    library_index
}

/// Clear the contents of a file.
fn clear_file(file: &mut std::fs::File) {
    file.set_len(0).unwrap();
    file.seek(SeekFrom::Start(0)).unwrap();
}

/// Register new resources and update the checksum of existing resources.
fn librarian_register(
    config_file: &mut std::fs::File,
    library_index: &mut LibraryIndex,
    resources_directory_path: &PathBuf,
) {
    let mut file_hashes = HashMap::<String, PathBuf>::new();
    WalkDir::new(resources_directory_path)
        .min_depth(1)
        .max_depth(1)
        .into_iter()
        .for_each(|f| {
            let file = f.unwrap();

            if file.file_type().is_dir() {
                file_hashes.insert(
                    directory_recursive_sha1(&file.clone().into_path())
                        .digest()
                        .to_string(),
                    file.clone().path().to_path_buf(),
                );
            } else {
                let file_contents = read(file.path()).expect("failed to read file");
                let mut sha = sha1::Sha1::new();
                sha.update(&file_contents);
                file_hashes.insert(sha.digest().to_string(), file.clone().path().to_path_buf());
            }
        });

    update_resources(library_index, &file_hashes, config_file);
}

fn librarian_instantiate(library_index: &LibraryIndex) {
    // TODO not yet implemented
    // assert!(false);
}

/// TODO
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

/// Add new files (or directories) in the resources directory to the
/// config file and change the file to its current SHA-1 checksum.
///
/// # Arguments
///
/// * `file_hashes` - File path and checksum for every file and directory in the resources directory.
fn update_resources(
    library_index: &mut LibraryIndex,
    file_hashes: &HashMap<String, PathBuf>,
    config_file: &mut std::fs::File,
) {
    // TODO this implementation could probably be more efficient

    // create a hash of all resources in the config file for fast lookup
    let mut library_index_resource_hash = HashMap::<String, Resource>::new();
    for resource in &library_index.resources {
        library_index_resource_hash
            .insert(resource.historical_checksums[0].clone(), resource.clone());
    }

    for (hash, file_path) in file_hashes {
        let file_name = file_path.file_stem().unwrap().to_str().unwrap().to_string();
        match library_index_resource_hash.get_mut(&file_name) {
            // update the checksum if it's changed
            Some(r) => {
                if r.checksum != hash.to_string() {
                    r.historical_checksums.push(hash.to_string());
                    r.checksum = hash.to_string();
                }
                // Remove resources from hash map as we iterate
                // through, so we can remove all resources from the
                // config file that no longer have corresponding
                // resource files.
                library_index_resource_hash.remove(&file_name);
            }
            None => {
                let new_resource = Resource {
                    title: String::from(""),
                    authors: std::vec!(),
                    date: {
                        Date {
                            year: None,
                            month: None,
                            day: None,
                        }
                    },
                    edition: None,
                    publisher: None,
                    organization: None,
                    tags: std::vec!(),
                    checksum: hash.to_string(),
                    historical_checksums: std::vec!(hash.to_string()),
                    resource_type: None,
                };
                library_index.resources.push(new_resource.clone());

                // rename the file to the current sha one contents
                let mut new_file_name = hash.to_string();
                // unless the file is a directory, add back the extension
                if file_path.is_file() {
                    new_file_name.push_str(".");
                    new_file_name.push_str(file_path.extension().unwrap().to_str().unwrap());
                }
                let new_file_path = file_path.parent().unwrap().join(new_file_name);
                std::fs::rename(file_path, new_file_path).unwrap();
            }
        }
    }

    // remove config file resources no longer in the resources directory
    let mut resource_hash = HashMap::<String, Resource>::new();
    for resource in &library_index.resources {
        resource_hash.insert(resource.historical_checksums[0].clone(), resource.clone());
    }
    for resource in library_index_resource_hash.keys() {
        resource_hash.remove(resource);
    }
    library_index.resources = resource_hash.values().cloned().collect();
    // sort resources by title into alphanumeric order
    library_index
        .resources
        .sort_by(|a, b| a.title.partial_cmp(&b.title).unwrap());

    clear_file(config_file);
    serde_json::to_writer_pretty(config_file, &library_index).unwrap();
}
