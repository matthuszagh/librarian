use clap::{app_from_crate, App, Arg};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha1::Sha1;
use std::env;
use std::fs::{read, read_to_string, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::string::String;
use std::vec::Vec;
use walkdir::WalkDir;

#[derive(Serialize, Deserialize, Debug)]
struct Name {
    first: Option<String>,
    middle: Option<String>,
    last: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Resource {
    title: String,
    authors: Vec<Name>,
    year: Option<i32>,
    edition: Option<i32>,
    publisher: Option<String>,
    tags: Vec<String>,
    checksum: String,
    previous_checksums: Option<Vec<String>>,
}

struct FileHash {
    file: PathBuf,
    hash: Sha1,
}

fn main() {
    // parse command line arguments
    let args = app_from_crate!()
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
        .get_matches();

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
    let mut config_file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(&config_file_path)
        .expect("Failed to open or create config file");

    let mut config_contents: String =
        read_to_string(config_file_path).expect("failed to read config file into a string");

    // initialize the config file if it's empty
    if config_contents == "" {
        let new_config_contents =
            b"{\n    \"tags\": {},\n\n    \"instances\": {},\n\n    \"resources\": {}\n\n}";
        config_file
            .write(new_config_contents)
            .expect("failed to write initial contents to config file");
        // config_contents needs the current valid file contents to parse json
        config_contents = String::from(
            std::str::from_utf8(new_config_contents)
                .expect("could not convert &[u8] to valid UTF-8"),
        );
    }

    let mut json_value: Value =
        serde_json::from_str(&config_contents).expect("config file contains invalid json");

    let mut file_hashes: Vec<FileHash> = Vec::new();
    WalkDir::new(resources_directory)
        .min_depth(1)
        .max_depth(1)
        .into_iter()
        .for_each(|f| {
            let file = f.unwrap();

            if file.file_type().is_dir() {
                file_hashes.push(FileHash {
                    file: file.path().to_path_buf(),
                    hash: directory_recursive_sha1(file.path()),
                });
            } else {
                let file_contents = read(file.path()).expect("failed to read file");
                let mut sha = sha1::Sha1::new();
                sha.update(&file_contents);
                file_hashes.push(FileHash {
                    file: file.path().to_path_buf(),
                    hash: sha,
                });
            }
        });

    register_new_files(&mut json_value, file_hashes);
}

/// TODO
fn directory_recursive_sha1(directory_path: &Path) -> Sha1 {
    // TODO
    Sha1::new()
}

/// TODO
fn register_new_files(json: &mut Value, file_hashes: Vec<FileHash>) {
    for file_hash in file_hashes {}

    let resources = &json["resources"];
    for (_, r) in resources
        .as_object()
        .expect("resources key is not an object")
        .into_iter()
    {
        let resource: Resource = serde_json::from_value(r.clone()).unwrap();
        println!("{:?}", resource.checksum);
    }
    // TODO
}

// fn hash_in_resources(hash: String, resources: serde_json::Value) -> bool {
//     for resource in resources
//         .as_object()
//         .expect("resources key is not an object")
//         .iter()
//     {
//         if resource.checksum == hash {
//             return true;
//         }
//     }
//     false
// }
