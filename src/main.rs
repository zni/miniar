use std::env;
use std::process;

use miniar::Config;

fn main() {
    let args: Vec<String> = env::args().collect();
    let config = Config::new(&args).unwrap_or_else(|err| {
        println!("miniar: {}", err);
        process::exit(1);
    });

    if let Err(e) = miniar::run(&config) {
        println!("miniar: {}", e);
        process::exit(1);
    }
}

