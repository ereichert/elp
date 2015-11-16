extern crate walkdir;

use std::path;
use self::walkdir::WalkDir;
use std::io;

pub fn file_list(dir: &path::Path, filenames: &mut Vec<walkdir::DirEntry>) -> Result<(), io::Error> {
    for entry in WalkDir::new(dir).min_depth(1) {
        let entry = entry.unwrap();
        filenames.push(entry);
    }
    Ok(())
}
