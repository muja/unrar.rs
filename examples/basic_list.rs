extern crate unrar;

fn main() {
    for entry in unrar::Archive::new("archive.rar").list().unwrap() {
        println!("{}", entry.unwrap().filename);
    }
}
