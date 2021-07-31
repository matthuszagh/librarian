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

/// Library "resource". This represents one unit of library content,
/// whether a file (such as a document or video), or a directory
/// containing the contents of a webpage.
#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
struct Resource {
    title: String,
    authors: Vec<Name>,
    year: Option<i32>,
    edition: Option<i32>,
    publisher: Option<String>,
    tags: Vec<String>,
    checksum: String,
    historical_checksums: Vec<String>,
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
        .truncate(true)
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
    // TODO
    let mut file_hashes = HashMap::new();
    WalkDir::new(resources_directory_path)
        .min_depth(1)
        .max_depth(1)
        .into_iter()
        .for_each(|f| {
            let file = f.unwrap();

            if file.file_type().is_dir() {
                file_hashes.insert(
                    directory_recursive_sha1(file.path()).digest().to_string(),
                    file.path().to_path_buf(),
                );
            } else {
                let file_contents = read(file.path()).expect("failed to read file");
                let mut sha = sha1::Sha1::new();
                sha.update(&file_contents);
                file_hashes.insert(sha.digest().to_string(), file.path().to_path_buf());
            }
        });

    update_resources(library_index, &file_hashes, config_file);
}

fn librarian_instantiate(library_index: &LibraryIndex) {
    // TODO not yet implemented
    // assert!(false);
}

/// TODO
fn directory_recursive_sha1(directory_path: &Path) -> Sha1 {
    // TODO
    Sha1::new()
}

/// Add new files (or directories) in the resources directory to the
/// config file and change the file to its current SHA-1 checksum.
fn update_resources(
    library_index: &mut LibraryIndex,
    file_hashes: &HashMap<String, PathBuf>,
    config_file: &mut std::fs::File,
) {
    // create a hash of all resources in the config file for fast lookup
    let mut resource_hash = HashMap::<String, Resource>::new();
    for resource in &library_index.resources {
        resource_hash.insert(resource.historical_checksums[0].clone(), resource.clone());
    }

    for (hash, file_path) in file_hashes {
        let file_name = file_path.file_stem().unwrap().to_str().unwrap().to_string();
        match resource_hash.get_mut(&file_name) {
            // update the checksum if it's changed
            Some(r) => {
                if r.checksum != hash.to_string() {
                    r.historical_checksums.push(hash.to_string());
                    r.checksum = hash.to_string();
                }
            }
            None => {
                let new_resource = Resource {
                    title: String::from(""),
                    authors: std::vec!(),
                    year: None,
                    edition: None,
                    publisher: None,
                    tags: std::vec!(),
                    checksum: hash.to_string(),
                    historical_checksums: std::vec!(hash.to_string()),
                };
                library_index.resources.push(new_resource.clone());
                // it's necessary to update the hash in case we added the file twice to the resources directory.
                resource_hash.insert(file_name, new_resource);

                // rename the file the current sha one contents
                let mut new_file_name = hash.to_string();
                new_file_name.push_str(".");
                new_file_name.push_str(file_path.extension().unwrap().to_str().unwrap());
                let new_file_path = file_path.parent().unwrap().join(new_file_name);
                std::fs::rename(file_path, new_file_path).unwrap();
            }
        }
    }

    clear_file(config_file);
    serde_json::to_writer_pretty(config_file, &library_index).unwrap();
}
