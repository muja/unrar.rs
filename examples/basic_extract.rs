use unrar::Archive;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = std::env::args();
    let file = args.skip(1).next().unwrap_or("archive.rar".to_owned());
    let mut archive = Archive::new(&file).open_for_processing().unwrap();
    while let Some(header) = archive.read_header()? {
        println!(
            "{} bytes: {}",
            header.entry().unpacked_size,
            header.entry().filename.to_string_lossy(),
        );
        archive = if header.entry().is_file() {
            header.extract()?
        } else {
            header.skip()?
        };
    }
    Ok(())
}
