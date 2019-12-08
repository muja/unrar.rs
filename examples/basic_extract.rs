extern crate unrar;

use unrar::Archive;

fn main() {
    Archive::new("archive.rar".into()).extract_to("./archive".into()).unwrap().process().unwrap();
    println!("Done.");
}
