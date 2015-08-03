extern crate unrar;

fn main() {
    unrar::Archive::new("archive.rar").extract_to("./archive").unwrap().process().unwrap();
    println!("Done.");
}
