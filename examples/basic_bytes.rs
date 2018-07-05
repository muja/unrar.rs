extern crate env_logger;
extern crate unrar;

use std::str;
use unrar::Archive;

fn main() {
    env_logger::init().unwrap();
    println!(
        "{}",
        str::from_utf8(&Archive::new("version.rar".into())
            .read_bytes("VERSION")
            .unwrap())
            .unwrap()
    );
}
