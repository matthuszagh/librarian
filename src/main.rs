mod bibtex;
mod cache;
mod catalog;
mod instance;
mod resource;
mod search;

use crate::bibtex::librarian_bibtex;
use crate::catalog::{librarian_catalog, Catalog};
use crate::instance::librarian_instantiate;
use crate::search::librarian_search;

use clap::{app_from_crate, App, Arg};
use std::env;
use std::fs::OpenOptions;
use std::path::PathBuf;

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
        librarian_catalog(
            &mut catalog_file,
            &mut catalog,
            &resources_path,
            match args
                .subcommand_matches("catalog")
                .unwrap()
                .value_of("cache")
                .expect("must provide an argument to the cache option")
            {
                "true" => true,
                "false" => false,
                &_ => panic!("true and false should be the only valid arguments"),
            },
        );
    } else if args.is_present("instantiate") {
        librarian_instantiate(&catalog);
    } else if args.is_present("search") {
        librarian_search(
            &catalog,
            args.subcommand_matches("search")
                .unwrap()
                .value_of("query")
                .expect("must provide a search query"),
        );
    } else if args.is_present("bibtex") {
        librarian_bibtex(
            &catalog,
            &resources_path,
            args.subcommand_matches("bibtex").unwrap().value_of("file"),
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
                .default_value("catalog.json"),
        )
        .arg(
            Arg::new("resources")
                .about("resources directory, relative to the library directory path")
                .takes_value(true)
                .short('r')
                .long("resources")
                .default_value("resources"),
        )
        .subcommand(
            App::new("catalog")
                .about("catalogs all new original resources")
                .arg(
                    Arg::new("cache")
                        .about("use the cache file to reduce the time required for cataloging")
                        .takes_value(true)
                        .short('c')
                        .long("cache")
                        .possible_values(&["true", "false"])
                        .default_value("true"),
                ),
        )
        .subcommand(App::new("catalog").about("catalogs all new original resources"))
        .subcommand(
            App::new("instantiate").about("instantiates one or more instances from the catalog"),
        )
        .subcommand(
            App::new("search")
                .about("retrieve a resource based on its metainformation")
                .arg(Arg::new("query").about("resource query").takes_value(true)),
        )
        .subcommand(
            App::new("bibtex")
                .about("generate a BibTeX bibliography")
                .arg(
                    Arg::new("file")
                        .about("file to write BibTeX data to")
                        .long_about(
                            "If this argument is omitted, BibTeX data will be written to stdout.",
                        ),
                ),
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
