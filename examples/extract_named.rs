extern crate unrar;

use unrar::{error::UnrarResult, Archive};

fn main() -> UnrarResult<()> {
    // Basic args parsing
    // Usage: cargo run --example basic_list /to/archive.rar
    let args = std::env::args();
    let file = args.skip(1).next().unwrap_or("archive.rar".to_owned());

    let mut archive = Archive::new(&file).open_for_processing().unwrap();
    while let Some(header) = archive.read_header() {
        let header = header?;
        archive = if header.entry().filename.as_os_str() == "README.md" {
            let (data, rest) = header.read()?;
            println!("found README file with len: {}", data.len());
            rest
        } else {
            header.skip()?
        }
    }
    Ok(())
}
