extern crate rustc_serialize;
extern crate docopt;

use docopt::Docopt;
use std::fs;
use std::path;
use std::io;

fn main() {
    let args: Args = Docopt::new(USAGE)
                            .and_then(|d| d.decode())
                            .unwrap_or_else(|e| e.exit());

    let log_location = &path::Path::new(&args.arg_log_location);
    let mut filenames = Vec::new();
    match file_list(log_location, &mut filenames){
        Ok(_) => {
            let file_count = filenames.len();
            println!("Found {:?} files.", file_count);
        },
        Err(e) => println!("An error occurred."),
    };
}

fn file_list(dir: &path::Path, filenames: &mut Vec<fs::DirEntry>) -> Result<(), io::Error> {
    if try!(fs::metadata(dir)).is_dir() {
        for entry in try!(fs::read_dir(dir)) {
            let entry = try!(entry);
            if try!(fs::metadata(entry.path())).is_dir() {
                try!(file_list(&entry.path(), filenames));
            } else {
                filenames.push(entry)
            }
        }
    }

    Ok(())
}
const USAGE: &'static str = "
aws-abacus

Usage:
  aws-abacus <log-location>
";

#[derive(Debug, RustcDecodable)]
struct Args {
    arg_log_location: String,
}
