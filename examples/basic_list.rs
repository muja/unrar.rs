extern crate unrar;

use unrar::Archive;

fn main() {
    for entry in Archive::new("archive.rar".into()).list().unwrap() {
        println!("{}", entry.unwrap());
    }
}
