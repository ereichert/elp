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

    let mut file_count = 0;
    walk_fs(log_location, &|dir| {
        // file_count += 1;
    });

    println!("Found {:?} files.", file_count);
}

fn walk_fs(dir: &path::Path, cb: &Fn(&fs::DirEntry)) -> io::Result<()> {
    if try!(fs::metadata(dir)).is_dir() {
        for entry in try!(fs::read_dir(dir)) {
            let entry = try!(entry);
            if try!(fs::metadata(entry.path())).is_dir() {
                try!(walk_fs(&entry.path(), cb));
            } else {
                cb(&entry);
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
