extern crate unrar;

use unrar::Archive;

fn main() {
    for entry in Archive::new("archive.rar").list().unwrap() {
        println!("{}", entry.unwrap());
    }
}
