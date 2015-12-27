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
use std::net::SocketAddrV4;

#[derive(Debug)]
pub struct ELBRecord {
    timestamp: DateTime<UTC>,
    elb_name: String,
    client_address: SocketAddrV4,
    backend_address: SocketAddrV4,
    request_processing_time: f32,
    backend_processing_time: f32,
    response_processing_time: f32,
    elb_status_code: u16,
    backend_status_code: u16,
    received_bytes: u64,
    sent_bytes: u64,
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


//TODO Reconsider logging based on the standard interfaces included with Rust.
//TODO We really want to accept a function to handle the parsed lines.
pub fn process_files(runtime_context: &::RuntimeContext, filenames: Vec<walkdir::DirEntry>) -> usize {
    let debug = runtime_context.debug;
    let mut record_count = 0;
    for filename in filenames {
        debug!(debug, "Processing file {}.", filename.path().display());
        match File::open(filename.path()) {
            Ok(file) => {
                let buffered_file = BufReader::new(&file);
                let recs: Vec<_> = buffered_file.lines()
                    .map(|possible_line| {
                        match possible_line {
                            Ok(record) => parse_record(record),

                            Err(_) => {
                                Err(ParsingErrors {
                                    record: "".to_string(),
                                    errors: vec![ELBRecordParsingErrors::LineReadError]
                                })
                            }
                        }
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

//TODO Take a look at the error handling once again.  This doesn't feel write given code you've read.

#[derive(Debug)]
pub struct ParsingErrors {
    record: String,
    errors: Vec<ELBRecordParsingErrors>,
}

#[derive(Debug, PartialEq)]
enum ELBRecordParsingErrors {
    MalformedRecord,
    ParsingError { property: &'static str, description: String },
    LineReadError
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum ELBRecordFields {
    Timestamp = 0,
    ELBName,
    ClientAddress,
    BackendAddress,
    RequestProcessingTime,
    BackendProcessingTime,
    ResponseProcessingTime,
    ELBStatusCode,
    BackendStatusCode,
    ReceivedBytes,
    SentBytes,
    RequestMethod,
    RequestURL,
    RequestHTTPVersion
}

impl ELBRecordFields {

  fn idx(&self) -> usize {
    *self as usize
  }

  fn as_str(&self) -> &'static str {
      match *self {
          ELBRecordFields::Timestamp => "timestamp",
          ELBRecordFields::ELBName => "ELB name",
          ELBRecordFields::ClientAddress => "client address",
          ELBRecordFields::BackendAddress => "backend address",
          ELBRecordFields::RequestProcessingTime => "request processing time",
          ELBRecordFields::BackendProcessingTime => "backend processing time",
          ELBRecordFields::ResponseProcessingTime => "response processing time",
          ELBRecordFields::ELBStatusCode => "ELB status code",
          ELBRecordFields::BackendStatusCode => "backend status code",
          ELBRecordFields::ReceivedBytes => "received bytes",
          ELBRecordFields::SentBytes => "sent bytes",
          ELBRecordFields::RequestMethod => "request method",
          ELBRecordFields::RequestURL => "request URL",
          ELBRecordFields::RequestHTTPVersion => "request HTTP version"
      }
  }
}

const ELB_RECORD_FIELD_COUNT: usize = 14;
pub fn parse_record(record: String) -> Result<Box<ELBRecord>, ParsingErrors> {
    let mut errors: Vec<ELBRecordParsingErrors> = Vec::new();

    {
        //record is borrowed by the split method which means ownership of record cannot be
        //transferred to ParsingErrors until the borrow is complete.
        //Scoping this section of code seems more readable than creating a separate function
        //just to mitigate the borrow.
        let split_line: Vec<&str> = record.split(' ').collect();
        if split_line.len() != ELB_RECORD_FIELD_COUNT {
            errors.push(ELBRecordParsingErrors::MalformedRecord);
            None
        } else {
            let ts = split_line.parse_property(ELBRecordFields::Timestamp, &mut errors);
            let clnt_addr = split_line.parse_property(ELBRecordFields::ClientAddress, &mut errors);
            let be_addr = split_line.parse_property(ELBRecordFields::BackendAddress, &mut errors);
            let req_proc_time = split_line.parse_property(ELBRecordFields::RequestProcessingTime, &mut errors);
            let be_proc_time = split_line.parse_property(ELBRecordFields::BackendProcessingTime, &mut errors);
            let res_proc_time = split_line.parse_property(ELBRecordFields::ResponseProcessingTime, &mut errors);
            let elb_sc = split_line.parse_property(ELBRecordFields::ELBStatusCode, &mut errors);
            let be_sc = split_line.parse_property(ELBRecordFields::BackendStatusCode, &mut errors);
            let bytes_received = split_line.parse_property(ELBRecordFields::ReceivedBytes, &mut errors);
            let bytes_sent = split_line.parse_property(ELBRecordFields::SentBytes, &mut errors);

            if errors.is_empty() {
                //If errors is empty it is more than likely parsing was successful and unwrap is safe.
                Some(
                    ELBRecord {
                        timestamp: ts.unwrap(),
                        elb_name: split_line[ELBRecordFields::ELBName.idx()].to_string(),
                        client_address: clnt_addr.unwrap(),
                        backend_address: be_addr.unwrap(),
                        request_processing_time: req_proc_time.unwrap(),
                        backend_processing_time: be_proc_time.unwrap(),
                        response_processing_time: res_proc_time.unwrap(),
                        elb_status_code: elb_sc.unwrap(),
                        backend_status_code: be_sc.unwrap(),
                        received_bytes: bytes_received.unwrap(),
                        sent_bytes: bytes_sent.unwrap(),
                        request_method: split_line[ELBRecordFields::RequestMethod.idx()].trim_matches('"').to_string(),
                        request_url: split_line[ELBRecordFields::RequestURL.idx()].to_string(),
                        request_http_version: split_line[ELBRecordFields::RequestHTTPVersion.idx()].trim_matches('"').to_string()
                    }
                )
            } else {
                None
            }
        }
    }.map( |elb_rec|
        Ok(Box::new(elb_rec))
    ).unwrap_or_else( ||
        Err(ParsingErrors {
            record: record,
            errors: errors
        })
    )
}

trait ELBRecordFieldParser {
    fn parse_property<T>(
        &self,
        field_id: ELBRecordFields,
        errors: &mut Vec<ELBRecordParsingErrors>
    ) -> Option<T>
        where T: FromStr,
        T::Err: Error + 'static;
}

impl<'a> ELBRecordFieldParser for Vec<&'a str> {

    fn parse_property<T>(
        &self,
        field_id: ELBRecordFields,
        errors: &mut Vec<ELBRecordParsingErrors>
    ) -> Option<T>
        where T: FromStr,
        T::Err: Error + 'static,
    {
        let raw_prop = self[field_id.idx()];
        match raw_prop.parse::<T>() {
            Ok(parsed) => Some(parsed),

            Err(e) => {
                errors.push(
                    ELBRecordParsingErrors::ParsingError {
                        property: field_id.as_str(),
                        description: e.description().to_string(),
                    }
                );
                None
            }
        }
    }
}

//TODO test the error case.
#[cfg(test)]
mod tests {
    use super::parse_record;
    use super::ELBRecordParsingErrors;

    const TEST_RECORD: &'static str = "2015-08-15T23:43:05.302180Z elb-name 172.16.1.6:54814 \
    172.16.1.5:9000 0.000039 0.145507 0.00003 200 200 0 7582 \
    \"GET http://some.domain.com:80/path0/path1?param0=p0&param1=p1 HTTP/1.1\"\
    ";

    #[test]
	fn parse_record_returns_a_malformed_record_error_for_records_short_on_values() {
        let short_record = "2015-08-15T23:43:05.302180Z elb-name 172.16.1.6:54814 \
        172.16.1.5:9000 0.000039 200 200 0 7582 \
        \"GET http://some.domain.com:80/path0/path1?param0=p0&param1=p1 HTTP/1.1\" \
        ";

        let malformed_error = parse_record(short_record.to_string()).unwrap_err().errors.pop();

		assert_eq!(malformed_error, Some(ELBRecordParsingErrors::MalformedRecord))
	}

    #[test]
	fn parse_record_returns_a_record_with_the_request_http_version() {
        let elb_record = parse_record(TEST_RECORD.to_string()).unwrap();

		assert_eq!(elb_record.request_http_version, "HTTP/1.1")
	}

    #[test]
	fn parse_record_returns_a_record_with_the_request_url() {
        let elb_record = parse_record(TEST_RECORD.to_string()).unwrap();

		assert_eq!(elb_record.request_url, "http://some.domain.com:80/path0/path1?param0=p0&param1=p1")
	}

    #[test]
	fn parse_record_returns_a_record_with_the_request_method() {
        let elb_record = parse_record(TEST_RECORD.to_string()).unwrap();

		assert_eq!(elb_record.request_method, "GET")
	}

    #[test]
	fn parse_record_returns_a_record_with_the_sent_bytes() {
        let elb_record = parse_record(TEST_RECORD.to_string()).unwrap();

		assert_eq!(elb_record.sent_bytes, 7582)
	}

    #[test]
	fn parse_record_returns_a_record_with_the_received_bytes() {
        let elb_record = parse_record(TEST_RECORD.to_string()).unwrap();

		assert_eq!(elb_record.received_bytes, 0)
	}

    #[test]
	fn parse_record_returns_a_record_with_the_backend_status_code() {
        let elb_record = parse_record(TEST_RECORD.to_string()).unwrap();

		assert_eq!(elb_record.backend_status_code, 200)
	}

    #[test]
	fn parse_record_returns_a_record_with_the_elb_status_code() {
        let elb_record = parse_record(TEST_RECORD.to_string()).unwrap();

		assert_eq!(elb_record.elb_status_code, 200)
	}

    #[test]
	fn parse_record_returns_a_record_with_the_response_processing_time() {
        let elb_record = parse_record(TEST_RECORD.to_string()).unwrap();

		assert_eq!(elb_record.response_processing_time, 0.00003)
	}

    #[test]
	fn parse_record_returns_a_record_with_the_backend_processing_time() {
        let elb_record = parse_record(TEST_RECORD.to_string()).unwrap();

		assert_eq!(elb_record.backend_processing_time, 0.145507)
	}

    #[test]
	fn parse_record_returns_a_record_with_the_request_processing_time() {
        let elb_record = parse_record(TEST_RECORD.to_string()).unwrap();

		assert_eq!(elb_record.request_processing_time, 0.000039)
	}

    #[test]
	fn parse_record_returns_a_record_with_the_backend_address() {
        let elb_record = parse_record(TEST_RECORD.to_string()).unwrap();

		assert_eq!(elb_record.backend_address, "172.16.1.5:9000".parse().unwrap())
	}

    #[test]
	fn parse_record_returns_a_record_with_the_client_address() {
        let elb_record = parse_record(TEST_RECORD.to_string()).unwrap();

		assert_eq!(elb_record.client_address, "172.16.1.6:54814".parse().unwrap())
	}

    #[test]
	fn parse_record_returns_a_record_with_the_timestamp() {
        let elb_record = parse_record(TEST_RECORD.to_string()).unwrap();

		assert_eq!(format!("{:?}", elb_record.timestamp), "2015-08-15T23:43:05.302180Z")
	}

    #[test]
	fn parse_record_returns_a_record_with_the_elb_name() {
        let elb_record = parse_record(TEST_RECORD.to_string()).unwrap();

		assert_eq!(elb_record.elb_name, "elb-name")
	}
}
