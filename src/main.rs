use clap::{app_from_crate, App, Arg};
use serde_json::Value;
use std::env;
use std::fs::{read, read_to_string, OpenOptions};
use std::path::PathBuf;
use std::string::String;
use std::vec::Vec;
use walkdir::WalkDir;
// use serde::{Deserialize, Serialize};

// #[derive(serde::Serialize, Deserialize)]
// struct Resource {
//     title: String,
//     authors: Vec<String>,
//     tags: Vec<String>,
//     checksum: String,
//     previous_checksums: Vec<String>,
// }

// TODO inline this and use `expect`
fn directory_file_hashes(directory: &PathBuf) -> Vec<(PathBuf, String)> {
    let mut file_path_hash: Vec<(PathBuf, String)> = Vec::new();
    let directory_files = WalkDir::new(directory).min_depth(1).max_depth(1);

    for file in directory_files {
        let file = match file {
            Ok(f) => f,
            Err(e) => panic!("{}", e),
        };

        let mut sha = sha1::Sha1::new();

        let file_contents = match read(file.path()) {
            Ok(x) => x,
            Err(e) => panic!("{}", e),
        };

        sha.update(&file_contents);
        file_path_hash.push((file.path().to_path_buf(), sha.digest().to_string()));
    }

    file_path_hash
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
    let config_file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(&config_file_path)
        .expect("Failed to open or create config file");

    let config_contents: String =
        read_to_string(config_file_path).expect("failed to read config file into a string");
    // println!("{:?}", config_contents);

    // TODO handle empty file
    let json_value: Value = match serde_json::from_str(&config_contents) {
        Ok(x) => x,
        Err(e) => panic!("{:?}", e),
    };

    let dir_file_hashes = directory_file_hashes(&resources_directory);
    for file_hash in dir_file_hashes {
        println!("{}: {}", file_hash.0.display(), file_hash.1);
    }
}
