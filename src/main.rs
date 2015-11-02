extern crate rustc_serialize;
extern crate docopt;

use docopt::Docopt;
use std::fs;
use std::path;
use std::io;
use std::fs::File;
use std::io::BufReader;
use std::io::BufRead;

macro_rules! debug {
    ($debug:ident, $fmt:expr, $($arg:tt)*) => {
        if $debug {
            use std::io::Write;
            match writeln!(&mut ::std::io::stderr(), concat!("DEBUG: ", $fmt), $($arg)*) {
                Ok(_) => {},
                Err(x) => panic!("Unable to write to stderr: {}", x),
            }
        }
    };

    ($debug:ident, $msg:expr) => { debug!($debug, $msg, ) }
}

fn main() {
    let args: Args = Docopt::new(USAGE)
                            .and_then(|d| d.decode())
                            .unwrap_or_else(|e| e.exit());

    let debug = args.flag_debug;

    let log_location = &path::Path::new(&args.arg_log_location);
    debug!(debug, "Running summary on {}.", log_location.to_str().unwrap());

    let mut filenames = Vec::new();
    match file_list(log_location, &mut filenames){
        Ok(_) => {
            debug!(debug, "DEBUG: Found {} files.", filenames.len());
            let mut record_count = 0;
            for filename in filenames {
                match File::open(filename.path()) {
                    Ok(file) => {
                        let buffered_file = BufReader::new(&file);
                        // let lines = buffered_file.lines();
                        // for line in lines {
                        //     // let l = line.unwrap();
                        //     // println!("{}", l);
                        //
                        // }
                        record_count += buffered_file.lines().count();
                    },
                    Err(e) => {
                        println!("{}", e);
                    }
                }
            }
            debug!(debug, "Found {} records.", record_count);
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
  aws-abacus (-d | --debug) <log-location>
  aws-abacus (-h | --help)

Options:
  -h --help     Show this screen.
  -d --debug    Turn on debug output
";

#[derive(Debug, RustcDecodable)]
struct Args {
    arg_log_location: String,
    flag_debug: bool,
}
