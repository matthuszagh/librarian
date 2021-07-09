use std::fs::read;
use walkdir::WalkDir;

fn main() {
    // let directory = "/home/matt/doc/library-beta/contents";
    let directory = "/home/matt/doc/library/engineering/electrical/general";
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
        println!("{}: {}", file.path().display(), sha.digest().to_string());
    }
}
