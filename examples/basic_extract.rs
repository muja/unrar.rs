extern crate unrar;

use unrar::Archive;

fn main() {
    Archive::new("archive.rar").extract_to("./archive").unwrap().process().unwrap();
    println!("Done.");
}
