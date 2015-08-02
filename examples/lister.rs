extern crate unrar;

use std::io::Write;
use std::fs;

fn main() {
    let args = std::env::args();
    let mut stderr = std::io::stderr();
    let file = args.skip(1).next().unwrap_or_else(|| {
        writeln!(&mut stderr, "Please pass an archive as argument!").unwrap();
        std::process::exit(0)
    });
    match unrar::Archive::new(&file).list_split() {
        Ok(archive) => {
            for entry in archive {
                match entry {
                    Ok(e) => {
                        println!("{}", e.filename);

                        // TODO: make this API more usable
                        if let Some(ref next) = e.next {
                            if let Err(err) = fs::metadata(next) {
                                writeln!(
                                    &mut stderr,
                                    "Couldn't find volume {}: {}", next, err
                                ).unwrap();
                                break;
                            }
                        }
                    }
                    Err(err) => println!("Error: {:?}", err)
                }
            }
        },
        Err(e) => {
            println!("Error opening: {:?}", e);
        }
    }
}
