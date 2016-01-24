extern crate rustc_serialize;
extern crate docopt;
#[macro_use]
extern crate aws_abacus;
#[macro_use]
extern crate log;
extern crate walkdir;
extern crate chrono;
use docopt::Docopt;
use std::path;
use aws_abacus::elb_log_files;
use chrono::{DateTime, UTC};

fn main() {
    let args: Args = Docopt::new(USAGE)
                            .and_then(|d| d.decode())
                            .unwrap_or_else(|e| e.exit());

    let log_location = &path::Path::new(&args.arg_log_location);
    debug!("Running summary on {}.", log_location.to_str().unwrap());

    let start: Option<DateTime<UTC>> = if args.flag_benchmark {
        Some(UTC::now())
    } else {
        None
    };

    let mut number_of_files = 0;
    let mut number_of_records = 0;
    let mut filenames = Vec::new();
    match elb_log_files::file_list(log_location, &mut filenames) {
        Ok(num_files) => {
            number_of_files = num_files;
            debug!("Found {} files.", number_of_files);
            number_of_records = elb_log_files::process_files(&filenames);
            debug!("Processed {} records in {} files.", number_of_records, num_files);
        },
        Err(e) => {
            println!("ERROR: {}", e);
        },
    };

    match start {
        Some(s) => {
            let end = UTC::now();
            let time = end - s;
            println!("Processed {} files having {} records in {} milliseconds.",
                number_of_files,
                number_of_records,
                time.num_milliseconds()
            );
        },
        None => {},
    };
}

const USAGE: &'static str = "
aws-abacus

Usage:
  aws-abacus <log-location>
  aws-abacus (-d | --debug | -b | --benchmark) <log-location>
  aws-abacus (-h | --help)

Options:
  -h --help     Show this screen.
  -d --debug    Turn on debug output
";

#[derive(Debug, RustcDecodable)]
struct Args {
    arg_log_location: String,
    flag_debug: bool,
    flag_benchmark: bool,
}
