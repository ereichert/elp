extern crate walkdir;

use std::path;
use self::walkdir::{WalkDir, DirEntry, Error};
use std::fs::File;
use std::io::BufReader;
use std::io::BufRead;

pub fn file_list(dir: &path::Path, filenames: &mut Vec<DirEntry>) -> Result<usize, Error> {
    for entry in WalkDir::new(dir).min_depth(1) {
        match entry {
            Err(err) => return Err(err),
            Ok(entry) => filenames.push(entry),
        }
    }
    Ok(filenames.len())
}

pub fn process_files(runtime_context: &::RuntimeContext, filenames: Vec<walkdir::DirEntry>) -> usize {
    let debug = runtime_context.debug;
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
                debug!(debug, "Found {} records in file {}.", current_file_count, filename.path().display());
            },
            Err(e) => {
                println!("ERROR: {}", e);
            }
        }
    }

    record_count
}
