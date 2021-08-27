mod bibtex;
mod catalog;
mod instance;
mod resource;
mod search;

use crate::bibtex::librarian_bibliography;
use crate::catalog::{librarian_catalog, Catalog};
use crate::instance::librarian_instantiate;
use crate::search::librarian_search;

use clap::{app_from_crate, App, Arg};
use std::env;
use std::fs::OpenOptions;
use std::path::PathBuf;
use std::string::String;

fn main() {
    let args = parse_app_args();
    let (resources_path, catalog_path) = library_paths(&args);
    let mut catalog_file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(&catalog_path)
        .expect("Failed to open or create catalog");
    let mut catalog = Catalog::read_from_file(&mut catalog_file);

    // Invoke the function for the appropriate subcommand. If no
    // subcommand is given, perform "catalog" followed by
    // "instantiate".
    if args.is_present("catalog") {
        librarian_catalog(&mut catalog_file, &mut catalog, &resources_path);
    } else if args.is_present("instantiate") {
        librarian_instantiate(&catalog);
    } else if args.is_present("search") {
        librarian_search(
            &catalog,
            &resources_path,
            args.subcommand_matches("search")
                .unwrap()
                .value_of("type")
                .unwrap(),
            args.subcommand_matches("search")
                .unwrap()
                .value_of("query")
                .expect("must provide a search query"),
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
        panic!("Subcommand required.");
    }
}

/// Parse and return command line arguments.
fn parse_app_args() -> clap::ArgMatches {
    app_from_crate!()
        .arg(
            Arg::new("directory")
                .about("library directory path")
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
            Arg::new("catalog_file")
                .about("library catalog file, relative to the library directory path")
                .takes_value(true)
                .short('c')
                .long("catalog")
                .default_value("catalog.json")
        )
        .arg(
            Arg::new("resources")
                .about("resources directory, relative to the library directory path")
                .takes_value(true)
                .short('r')
                .long("resources")
                .default_value("resources")
        )
        .subcommand(
            App::new("catalog")
                .about("catalogs all new original resources")
        )
        .subcommand(
            App::new("instantiate")
                .about("instantiates one or more instances from the catalog")
        )
        .subcommand(
            App::new("validate")
                .about("validates the catalog")
        )
        .subcommand(
            App::new("search")
                .about("retrieve a resource based on its metainformation")
                .arg(
                    Arg::new("type")
                        .about("search type")
                        .short('t')
                        .long("type")
                        .takes_value(true)
                        .default_value("fuzzy")
                        .possible_values(&["fuzzy", "regex"])
                )
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
        args.value_of("catalog_file")
            .expect("failed to retrieve catalog argument"),
    );

    (resources_directory, catalog_path)
}
