extern crate unrar;
extern crate env_logger;

use unrar::Archive;

fn main() {
    env_logger::init().unwrap();
    Archive::new("archive.rar".into()).extract_to("./archive".into()).unwrap().process().unwrap();
    println!("Done.");
}
