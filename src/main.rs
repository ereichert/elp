extern crate rustc_serialize;
extern crate docopt;
#[macro_use]
extern crate aws_abacus;
extern crate walkdir;

use docopt::Docopt;
use std::path;
use std::fs::File;
use std::io::BufReader;
use std::io::BufRead;
use aws_abacus::elb_log_files;
use walkdir::DirEntry;

fn main() {
    let args: Args = Docopt::new(USAGE)
                            .and_then(|d| d.decode())
                            .unwrap_or_else(|e| e.exit());

    let debug = args.flag_debug;

    let log_location = &path::Path::new(&args.arg_log_location);
    debug!(debug, "Running summary on {}.", log_location.to_str().unwrap());

    let mut filenames = Vec::new();
    match elb_log_files::file_list(log_location, &mut filenames) {
        Ok(_) => {
            debug!(debug, "Found {} files.", filenames.len());
            let record_count = handle_files(filenames);
            debug!(debug, "Found {} records.", record_count);
        },
        Err(e) => println!("An error occurred."),
    };
}

fn handle_files(filenames: Vec<walkdir::DirEntry>) -> usize {
    let mut record_count = 0;
    for filename in filenames {
        // debug!(debug, "Processing file {}.", filename.path().display());
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
                // debug!(debug, "Found {} records.", current_file_count);
            },
            Err(e) => {
                println!("{}", e);
            }
        }
    }

    record_count
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
