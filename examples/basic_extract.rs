extern crate unrar;

fn main() {
    for entry in unrar::Archive::new("archive.rar").extract_to("./archive").unwrap() {
        println!("{}", entry.unwrap().filename);
    }
}
