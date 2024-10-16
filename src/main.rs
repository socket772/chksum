use checksums::{self, hash_file, Algorithm};
use std::env::args;
use walkdir::WalkDir;

fn main() {
    // args[1] = cartella
    // args[2] = file output
    let args: Vec<String> = args().collect();

    if args.len() != 3 {
        return;
    }

    let walk_dir = WalkDir::new(args[1].clone()).follow_links(true);

    for file in walk_dir {
        let file_path = file.unwrap();
        if file_path.path().is_file() {
            let file_checksum = hash_file(file_path.path(), Algorithm::CRC64);
            println!("{} {:?}", file_checksum, file_path.path())
        }
    }
}
