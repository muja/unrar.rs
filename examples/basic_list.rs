extern crate unrar;

use unrar::Archive;

fn main() {
    for entry in Archive::new("archive.rar").unwrap().list().unwrap() {
        println!("{}", entry.unwrap());
    }
}
