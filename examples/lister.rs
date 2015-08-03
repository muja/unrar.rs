extern crate unrar;

extern crate env_logger;

use unrar::Archive;
use unrar::error::{Code, When, UnrarError};
use std::io::Write;

fn main() {
    env_logger::init().unwrap();
    let args = std::env::args();
    let mut stderr = std::io::stderr();
    let file = args.skip(1).next().unwrap_or_else(|| {
        writeln!(&mut stderr, "Please pass an archive as argument!").unwrap();
        std::process::exit(0)
    });

    match Archive::new(&file).list_split() {
        Ok(archive) => {
            for entry in archive {
                match entry {
                    Ok(e) => println!("{}", e),
                    Err(UnrarError { code: Code::EOpen, when: When::Process, data: Some(e) }) => {
                        println!("{}", e);
                        writeln!(
                            &mut stderr,
                            "Couldn't find volume: {}", e.next_volume.unwrap()
                        ).unwrap();
                    }
                    Err(err) => println!("Error: {:?}", err.code)
                }
            }
        },
        Err(e) => {
            println!("Error opening: {:?}", e);
        }
    }
}
