extern crate unrar;

extern crate env_logger;

fn main() {
    env_logger::init().unwrap();
    unrar::Archive::new("archive.rar").extract_to("./archive").unwrap().process().unwrap();
    println!("Done.");
}
