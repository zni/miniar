use std::env;
use std::path::Path;

use miniar::Archive;

fn main() {
    let mut args: Vec<String> = env::args().collect();
    args.remove(0);

    for arg in args {
        let path = Path::new(arg.as_str());
        println!("file index for {}", path.display());
        let archive = Archive::from_path(&path).unwrap();
        for file in archive.files {
            println!("\t{}", file.name);
        }
        println!("")
    }
}

