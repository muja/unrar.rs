extern crate unrar;
extern crate env_logger;

use unrar::Archive;

fn main() {
    env_logger::init().unwrap();
    for entry in Archive::new("archive.rar".into()).list().unwrap() {
        println!("{}", entry.unwrap());
    }
}
