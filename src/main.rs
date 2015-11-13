extern crate rustc_serialize;
extern crate docopt;
#[macro_use]
extern crate aws_abacus;
extern crate walkdir;

use docopt::Docopt;
use std::path;
use std::io;
use std::fs::File;
use std::io::BufReader;
use std::io::BufRead;
use walkdir::WalkDir;

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
            debug!(debug, "Found {} files.", filenames.len());
            let mut record_count = 0;
            for filename in filenames {
                debug!(debug, "Processing file {}.", filename.path().display());
                match File::open(filename.path()) {
                    Ok(file) => {
                        let buffered_file = BufReader::new(&file);
                        // let lines = buffered_file.lines();
                        // for line in lines {
                        //     // let l = line.unwrap();
                        //     // println!("{}", l);
                        //
                        // }
                        let current_file_count = buffered_file.lines().count();
                        record_count += current_file_count;
                        debug!(debug, "Found {} records.", current_file_count);
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

fn file_list(dir: &path::Path, filenames: &mut Vec<walkdir::DirEntry>) -> Result<(), io::Error> {
    for entry in WalkDir::new(dir).min_depth(1) {
        let entry = entry.unwrap();
        filenames.push(entry);
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
