extern crate unrar;

use std::io::Write;
use unrar::error::{Code, UnrarError, When};
use unrar::Archive;

fn main() {
    // Basic args parsing
    // Usage: cargo run --example lister path/to/archive.rar

    let args = std::env::args();
    let mut stderr = std::io::stderr();
    let file = args.skip(1).next().unwrap_or_else(|| {
        writeln!(&mut stderr, "Please pass an archive as argument!").unwrap();
        std::process::exit(0)
    });

    match Archive::new(&file).unwrap().list_split() {
        // Everything okay, just list the archive
        Ok(archive) => list_archive(archive),

        // If the error's data field holds an OpenArchive, an error occurred while opening,
        // the archive is partly broken (e.g. broken header), but is still readable from.
        // In this example, we are still going to use the archive and list its contents.
        Err(error @ UnrarError { data: Some(_), .. }) => {
            writeln!(&mut stderr, "Error: {}, continuing.", error).unwrap();
            list_archive(error.data.unwrap());
        }
        // Irrecoverable failure, do nothing.
        Err(e) => {
            writeln!(&mut stderr, "Error: {}", e).unwrap();
        }
    }

    // to be DRY, the archive function is here.
    fn list_archive(archive: unrar::archive::OpenArchive) {
        // create a local copy of stderr.
        let mut stderr = std::io::stderr();
        for entry in archive {
            match entry {
                Ok(e) => println!("{}", e),
                // EOpen @ process() means that next volume was not found / not readable.
                // In this case, the partial entry is stored in the data field of that error.
                Err(UnrarError {
                    code: Code::EOpen,
                    when: When::Process,
                    data: Some(e),
                }) => {
                    // print the partial entry
                    println!("{}", e);
                    // emit warning that an error occured.
                    writeln!(
                        &mut stderr,
                        "Could not find volume: {:?}",
                        e.next_volume.unwrap()
                    )
                    .unwrap();
                    // The iterator will stop by itself, no further action needed.
                }
                Err(err) => writeln!(&mut stderr, "Error: {}", err).unwrap(),
            }
        }
    }
}
