extern crate unrar;

extern crate env_logger;

fn main() {
    env_logger::init().unwrap();
    for entry in unrar::Archive::new("archive.rar").list().unwrap() {
        println!("{}", entry.unwrap());
    }
}
