extern crate walkdir;
extern crate chrono;

use std::path;
use self::walkdir::{WalkDir, DirEntry, Error as WalkDirError};
use std::fs::File;
use std::io::BufReader;
use std::io::BufRead;
use self::chrono::{DateTime, UTC};
use std::error::Error;
use std::str::FromStr;

pub struct ELBLogEntry {
    timestamp: DateTime<UTC>,
    elb_name: String,
    client_address: String,
    backend_address: String,
    request_processing_time: f32,
    backend_processing_time: String,
    response_processing_time: String,
    elb_status_code: String,
    backend_status_code: String,
    received_bytes: String,
    sent_bytes: String,
    request_method: String,
    request_url: String,
    request_http_version: String
}

pub fn file_list(dir: &path::Path, filenames: &mut Vec<DirEntry>) -> Result<usize, WalkDirError> {
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
                let recs: Vec<_> = buffered_file.lines()
                    .map(|x| {
                        parse_line(&(x.unwrap()))
                    })
                    .collect();
                record_count += recs.len();
                debug!(debug, "Found {} records in file {}.", recs.len(), filename.path().display());
            },
            Err(e) => {
                println!("ERROR: {}", e);
            }
        }
    }

    record_count
}

#[derive(Debug)]
pub struct ParsingError{
    property: &'static str,
    inner_description: Box<Error>,
}

#[derive(Debug)]
pub struct ParsingErrors {
    record: String,
    errors: Vec<ParsingError>,
}

const TIMESTAMP: &'static str = "timestamp";
const REQUEST_PROCESSING_TIME: &'static str = "request processing time";

pub fn parse_line(line: &String) -> Result<Box<ELBLogEntry>, Box<ParsingErrors>> {
    let split_line: Vec<_> = line.split(" ").collect();
    let mut errors: Vec<ParsingError> = Vec::new();

    let ts = parse_property::<DateTime<UTC>>(split_line[0], TIMESTAMP, &mut errors);
    let req_proc_time = parse_property::<f32>(split_line[4], REQUEST_PROCESSING_TIME, &mut errors);

    if errors.is_empty() {
        Ok(Box::new(
            ELBLogEntry {
                timestamp: ts.unwrap(),
                elb_name: split_line[1].to_string(),
                client_address: split_line[2].to_string(),
                backend_address: split_line[3].to_string(),
                request_processing_time: req_proc_time.unwrap(),
                backend_processing_time: split_line[5].to_string(),
                response_processing_time: split_line[6].to_string(),
                elb_status_code: split_line[7].to_string(),
                backend_status_code: split_line[8].to_string(),
                received_bytes: split_line[9].to_string(),
                sent_bytes: split_line[10].to_string(),
                request_method: split_line[11].trim_matches('"').to_string(),
                request_url: split_line[12].to_string(),
                request_http_version: split_line[13].trim_matches('"').to_string()
            }
        ))
    } else {
        Err(Box::new(
            ParsingErrors {
                record: line.clone(),
                errors: errors
            }
        ))
    }
}

fn parse_property<T>(raw_prop: &str, prop_name: &'static str, errors: &mut Vec<ParsingError>) -> Option<T>
    where T: FromStr,
    T::Err: Error + 'static,
{
    match raw_prop.parse::<T>() {
        Ok(parsed) => Some(parsed),

        Err(e) => {
            errors.push(
                ParsingError {
                    property: prop_name,
                    inner_description: Box::new(e),
                }
            );
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::parse_line;

    const TEST_LINE: &'static str = "2015-08-15T23:43:05.302180Z elb-name 172.16.1.6:54814 \
    172.16.1.5:9000 0.000039 0.145507 0.00003 200 200 0 7582 \
    \"GET http://some.domain.com:80/path0/path1?param0=p0&param1=p1 HTTP/1.1\" \
    ";

    #[test]
	fn parse_line_returns_a_log_entry_with_the_request_http_version() {
        let elb_log_entry = parse_line(&TEST_LINE.to_string()).unwrap();

		assert_eq!(elb_log_entry.request_http_version, "HTTP/1.1")
	}

    #[test]
	fn parse_line_returns_a_log_entry_with_the_request_url() {
        let elb_log_entry = parse_line(&TEST_LINE.to_string()).unwrap();

		assert_eq!(elb_log_entry.request_url, "http://some.domain.com:80/path0/path1?param0=p0&param1=p1")
	}

    #[test]
	fn parse_line_returns_a_log_entry_with_the_request_method() {
        let elb_log_entry = parse_line(&TEST_LINE.to_string()).unwrap();

		assert_eq!(elb_log_entry.request_method, "GET")
	}

    #[test]
	fn parse_line_returns_a_log_entry_with_the_sent_bytes() {
        let elb_log_entry = parse_line(&TEST_LINE.to_string()).unwrap();

		assert_eq!(elb_log_entry.sent_bytes, "7582")
	}

    #[test]
	fn parse_line_returns_a_log_entry_with_the_received_bytes() {
        let elb_log_entry = parse_line(&TEST_LINE.to_string()).unwrap();

		assert_eq!(elb_log_entry.received_bytes, "0")
	}

    #[test]
	fn parse_line_returns_a_log_entry_with_the_backend_status_code() {
        let elb_log_entry = parse_line(&TEST_LINE.to_string()).unwrap();

		assert_eq!(elb_log_entry.backend_status_code, "200")
	}

    #[test]
	fn parse_line_returns_a_log_entry_with_the_elb_status_code() {
        let elb_log_entry = parse_line(&TEST_LINE.to_string()).unwrap();

		assert_eq!(elb_log_entry.elb_status_code, "200")
	}

    #[test]
	fn parse_line_returns_a_log_entry_with_the_response_processing_time() {
        let elb_log_entry = parse_line(&TEST_LINE.to_string()).unwrap();

		assert_eq!(elb_log_entry.response_processing_time, "0.00003")
	}

    #[test]
	fn parse_line_returns_a_log_entry_with_the_backend_processing_time() {
        let elb_log_entry = parse_line(&TEST_LINE.to_string()).unwrap();

		assert_eq!(elb_log_entry.backend_processing_time, "0.145507")
	}

    #[test]
	fn parse_line_returns_a_log_entry_with_the_request_processing_time() {
        let elb_log_entry = parse_line(&TEST_LINE.to_string()).unwrap();

		assert_eq!(elb_log_entry.request_processing_time, 0.000039)
	}

    #[test]
	fn parse_line_returns_a_log_entry_with_the_backend_address() {
        let elb_log_entry = parse_line(&TEST_LINE.to_string()).unwrap();

		assert_eq!(elb_log_entry.backend_address, "172.16.1.5:9000")
	}

    #[test]
	fn parse_line_returns_a_log_entry_with_the_client_address() {
        let elb_log_entry = parse_line(&TEST_LINE.to_string()).unwrap();

		assert_eq!(elb_log_entry.client_address, "172.16.1.6:54814")
	}

    #[test]
	fn parse_line_returns_a_log_entry_with_the_timestamp() {
        let elb_log_entry = parse_line(&TEST_LINE.to_string()).unwrap();

		assert_eq!(format!("{:?}", elb_log_entry.timestamp), "2015-08-15T23:43:05.302180Z")
	}

    #[test]
	fn parse_line_returns_a_log_entry_with_the_elb_name() {
        let elb_log_entry = parse_line(&TEST_LINE.to_string()).unwrap();

		assert_eq!(elb_log_entry.elb_name, "elb-name")
	}
}
