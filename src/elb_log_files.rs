extern crate walkdir;

use std::path;
use self::walkdir::{WalkDir, DirEntry};
use std::io;
use std::fs::File;
use std::io::BufReader;
use std::io::BufRead;

pub fn file_list(dir: &path::Path, filenames: &mut Vec<DirEntry>) -> Result<(), io::Error> {
    for entry in WalkDir::new(dir).min_depth(1) {
        let entry = entry.unwrap();
        filenames.push(entry);
    }
    Ok(())
}

pub fn handle_files(runtime_context: &::RuntimeContext, filenames: Vec<walkdir::DirEntry>) -> usize {
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
                debug!(debug, "Found {} records.", current_file_count);
            },
            Err(e) => {
                println!("{}", e);
            }
        }
    }

    record_count
}
