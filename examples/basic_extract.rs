extern crate unrar;

use unrar::Archive;

fn main() {
    Archive::new("archive.rar")
        .unwrap()
        .extract_to("./archive")
        .unwrap()
        .process()
        .unwrap();
    println!("Done.");
}
