extern crate rustc_serialize;
extern crate docopt;

use docopt::Docopt;
use std::fs;
use std::path;
use std::io;
use std::fs::File;
use std::io::BufReader;
use std::io::BufRead;

fn main() {
    let args: Args = Docopt::new(USAGE)
                            .and_then(|d| d.decode())
                            .unwrap_or_else(|e| e.exit());

    let log_location = &path::Path::new(&args.arg_log_location);
    let debug = args.flag_debug;
    if debug {
        println!("WARNING: RUNNING IN DEBUG MODE.");
    }

    let mut filenames = Vec::new();
    match file_list(log_location, &mut filenames){
        Ok(_) => {
            if debug {
                println!("DEBUG: Found {:?} files.", filenames.len());
            }
            let mut line_count = 0;
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
                        line_count += buffered_file.lines().count();
                    },
                    Err(e) => {
                        println!("{}", e);
                    }
                }
            }
            println!("Found {:?} lines.", line_count);
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

// macro_rules! println_stderr(
//     ($($arg:tt)*) => (
//         match writeln!(&mut ::std::io::stderr(), $($arg)* ) {
//             Ok(_) => {},
//             Err(x) => panic!("Unable to write to stderr: {}", x),
//         }
//     )
// );
