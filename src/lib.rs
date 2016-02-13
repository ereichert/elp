extern crate walkdir;
extern crate chrono;
#[macro_use]
extern crate log;

use std::path::Path;
use self::walkdir::{WalkDir, DirEntry};
use std::fs::File;
use std::io::BufReader;
use std::io::BufRead;
use self::chrono::{DateTime, UTC};
use std::error::Error;
use std::str::FromStr;
use std::net::SocketAddrV4;
use std::fmt::{Formatter, Display};
use std::fmt;
use std::ops::Index;

//For some reason AWS doesn't version their log file format so these version numbers where
//selected by me to bring some sanity to this.
//If a new version comes out we'll refactor this into seperate parsers based on the field count.
const ELB_RECORD_V1_FIELD_COUNT: usize = 14;
const ELB_RECORD_V2_FIELD_COUNT: usize = 17;

///The product of parsing a single AWS ELB log record.
///
///Outside of testing it is doubtful a user would have any reason to construct a ELBRecord
///manually.
#[derive(Debug)]
pub struct ELBRecord {
    pub timestamp: DateTime<UTC>,
    pub elb_name: String,
    pub client_address: SocketAddrV4,
    pub backend_address: SocketAddrV4,
    pub request_processing_time: f32,
    pub backend_processing_time: f32,
    pub response_processing_time: f32,
    pub elb_status_code: u16,
    pub backend_status_code: u16,
    pub received_bytes: u64,
    pub sent_bytes: u64,
    pub request_method: String,
    pub request_url: String,
    pub request_http_version: String,
    pub user_agent: String,
    pub ssl_cipher: String,
    pub ssl_protocol: String
}

pub type ParsingResult = Result<Box<ELBRecord>, ParsingErrors>;

#[derive(Debug)]
pub struct ParsingErrors {
    pub record: String,
    pub errors: Vec<ELBRecordParsingError>,
}

#[derive(Debug, PartialEq)]
pub enum ELBRecordParsingError {
    MalformedRecord,
    ParsingError { field_id: ELBRecordField, description: String },
    LineReadError,
    CouldNotOpenFile { path: String },
}

impl Display for ELBRecordParsingError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            ELBRecordParsingError::MalformedRecord => write!(f, "Record is malformed."),
            ELBRecordParsingError::ParsingError { ref field_id, ref description } => write!(f, "Parsing of field {} failed with the following error: {}.", field_id, description),
            ELBRecordParsingError::LineReadError => write!(f, "Unable to read a line."),
            ELBRecordParsingError::CouldNotOpenFile { ref path } => write!(f, "Unable to open file {}.", path),
        }
    }
}

impl Error for ELBRecordParsingError {
    fn description(&self) -> &str {
        match *self {
            ELBRecordParsingError::MalformedRecord => "malformed record",
            ELBRecordParsingError::ParsingError { .. } => "field parsing failed",
            ELBRecordParsingError::LineReadError => "failed to read line",
            ELBRecordParsingError::CouldNotOpenFile { .. } => "failed to open file",
        }
    }

    fn cause(&self) -> Option<&Error> {
        Some(self)
    }
}

pub fn file_list(dir: &Path, filenames: &mut Vec<DirEntry>) -> Result<usize, walkdir::Error> {
    for entry in WalkDir::new(dir).min_depth(1) {
        match entry {
            Err(err) => return Err(err),
            Ok(entry) => filenames.push(entry),
        }
    }
    Ok(filenames.len())
}

pub fn process_files<H>(filenames: &[DirEntry], record_handler: &mut H) -> usize
    where H: FnMut(ParsingResult) -> () {

    let mut total_record_count = 0;
    for filename in filenames {
        debug!("Processing file {}.", filename.path().display());
        match File::open(filename.path()) {
            Ok(file) => {
                let file_record_count = handle_file(file, record_handler);
                debug!("Found {} records in file {}.", file_record_count, filename.path().display());
                total_record_count += file_record_count;
            },

            Err(_) => {
                record_handler(
                    Err(ParsingErrors {
                        record: "".to_owned(),
                        errors: vec![ELBRecordParsingError::CouldNotOpenFile { path: format!("{}", filename.path().display()) }]
                    })
                )
            }
        }
    }

    total_record_count
}

fn handle_file<H>(file: File, record_handler: &mut H) -> usize
    where H: FnMut(ParsingResult) -> () {
    let mut file_record_count = 0;
    for possible_record in BufReader::new(&file).lines() {
        file_record_count += 1;
        match possible_record {
            Ok(record) => record_handler(parse_record(record)),

            Err(_) => {
                record_handler(
                    Err(ParsingErrors {
                        record: "".to_owned(),
                        errors: vec![ELBRecordParsingError::LineReadError]
                    })
                )
            }
        }
    };

    file_record_count
}

fn parse_record(record: String) -> ParsingResult {
    let mut errors: Vec<ELBRecordParsingError> = Vec::with_capacity(ELB_RECORD_V2_FIELD_COUNT);

    {
        //record is borrowed by split_record which means ownership of
        //record cannot be transferred to ParsingErrors until the borrow is complete.
        //Scoping this section of code seems more readable than creating a separate function
        //just to mitigate the borrow.
        let split_record: Vec<&str> = record.split_record();
        if split_record.len() != ELB_RECORD_V1_FIELD_COUNT && split_record.len() != ELB_RECORD_V2_FIELD_COUNT {
            errors.push(ELBRecordParsingError::MalformedRecord);
            None
        } else {
            let ts = split_record.parse_field(ELBRecordField::Timestamp, &mut errors);
            let clnt_addr = split_record.parse_field(ELBRecordField::ClientAddress, &mut errors);
            let be_addr = split_record.parse_field(ELBRecordField::BackendAddress, &mut errors);
            let req_proc_time = split_record.parse_field(ELBRecordField::RequestProcessingTime, &mut errors);
            let be_proc_time = split_record.parse_field(ELBRecordField::BackendProcessingTime, &mut errors);
            let res_proc_time = split_record.parse_field(ELBRecordField::ResponseProcessingTime, &mut errors);
            let elb_sc = split_record.parse_field(ELBRecordField::ELBStatusCode, &mut errors);
            let be_sc = split_record.parse_field(ELBRecordField::BackendStatusCode, &mut errors);
            let bytes_received = split_record.parse_field(ELBRecordField::ReceivedBytes, &mut errors);
            let bytes_sent = split_record.parse_field(ELBRecordField::SentBytes, &mut errors);
            let mut user_agent = "-";
            let mut ssl_cipher = "-";
            let mut ssl_protocol = "-";

            if split_record.len() == ELB_RECORD_V2_FIELD_COUNT {
                user_agent = split_record[ELBRecordField::UserAgent].trim_matches('"');
                ssl_cipher = &split_record[ELBRecordField::SSLCipher];
                ssl_protocol = &split_record[ELBRecordField::SSLProtocol];
            }

            if errors.is_empty() {
                //If errors is empty it is more than likely parsing was successful and unwrap is safe.
                Some(
                    ELBRecord {
                        timestamp: ts.unwrap(),
                        elb_name: split_record[ELBRecordField::ELBName].to_owned(),
                        client_address: clnt_addr.unwrap(),
                        backend_address: be_addr.unwrap(),
                        request_processing_time: req_proc_time.unwrap(),
                        backend_processing_time: be_proc_time.unwrap(),
                        response_processing_time: res_proc_time.unwrap(),
                        elb_status_code: elb_sc.unwrap(),
                        backend_status_code: be_sc.unwrap(),
                        received_bytes: bytes_received.unwrap(),
                        sent_bytes: bytes_sent.unwrap(),
                        request_method: split_record[ELBRecordField::RequestMethod].trim_matches('"').to_owned(),
                        request_url: split_record[ELBRecordField::RequestURL].to_owned(),
                        request_http_version: split_record[ELBRecordField::RequestHTTPVersion].trim_matches('"').to_owned(),
                        user_agent: user_agent.to_owned(),
                        ssl_cipher: ssl_cipher.to_owned(),
                        ssl_protocol: ssl_protocol.to_owned()
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

trait RecordSplitter {
    fn split_record(&self) -> Vec<&str>;
}

impl RecordSplitter for String {

    fn split_record(&self) -> Vec<&str> {
        let mut split_record: Vec<&str> = Vec::with_capacity(ELB_RECORD_V2_FIELD_COUNT);
        let mut parsing_context = RecordSplitterState::new();
        for (current_idx, next_char) in self.trim_left().char_indices() {
            if current_idx == (self.len() - 1) {
                // The end of the record has been reached. Push the rest of the chars into the vec.
                split_record.push(&self[parsing_context.start_of_field_index..current_idx + 1]);
            } else if parsing_context.skip_next_n_chars > 0 {
                parsing_context.skip_next_n_chars -= 1;
                parsing_context.start_of_field_index += 1;
            } else if next_char == parsing_context.end_delimiter {
                split_record.push(&self[parsing_context.start_of_field_index..current_idx]);
                parsing_context.start_of_field_index = current_idx + 1;
                parsing_context.next();
            }
        }
        debug!("{:?}", parsing_context);
        debug!("{:?}", split_record);
        split_record
    }
}

#[derive(Debug)]
struct RecordSplitterState {
    end_delimiter: char,
    current_field: ELBRecordField,
    next_field: ELBRecordField,
    skip_next_n_chars: usize,
    start_of_field_index: usize
}

const SPACE: char = ' ';
const DOUBLE_QUOTE: char = '"';
impl RecordSplitterState {

    fn new() -> RecordSplitterState {
        RecordSplitterState {
            end_delimiter: SPACE,
            //current_field makes debugging a little easier.
            current_field: ELBRecordField::Timestamp,
            next_field: ELBRecordField::ELBName,
            skip_next_n_chars: 0,
            start_of_field_index: 0
        }
    }

    fn next(&mut self) {
        self.current_field = self.next_field.clone();
        match self.current_field {
            ELBRecordField::Timestamp => self.next_field = ELBRecordField::ELBName,

            ELBRecordField::ELBName => self.next_field = ELBRecordField::ClientAddress,

            ELBRecordField::ClientAddress => self.next_field = ELBRecordField::BackendAddress,

            ELBRecordField::BackendAddress => self.next_field = ELBRecordField::RequestProcessingTime,

            ELBRecordField::RequestProcessingTime => self.next_field = ELBRecordField::BackendProcessingTime,

            ELBRecordField::BackendProcessingTime => self.next_field = ELBRecordField::ResponseProcessingTime,

            ELBRecordField::ResponseProcessingTime => self.next_field = ELBRecordField::ELBStatusCode,

            ELBRecordField::ELBStatusCode => self.next_field = ELBRecordField::BackendStatusCode,

            ELBRecordField::BackendStatusCode => self.next_field = ELBRecordField::ReceivedBytes,

            ELBRecordField::ReceivedBytes => self.next_field = ELBRecordField::SentBytes,

            ELBRecordField::SentBytes => self.next_field = ELBRecordField::RequestMethod,

            ELBRecordField::RequestMethod => {
                self.end_delimiter = SPACE;
                self.next_field = ELBRecordField::RequestURL;
                self.skip_next_n_chars = 1;
            },

            ELBRecordField::RequestURL => {
                self.end_delimiter = SPACE;
                self.next_field = ELBRecordField::RequestHTTPVersion;
            },

            ELBRecordField::RequestHTTPVersion => {
                self.end_delimiter = DOUBLE_QUOTE;
                self.next_field = ELBRecordField::UserAgent;
            },

            ELBRecordField::UserAgent => {
                self.end_delimiter = DOUBLE_QUOTE;
                self.next_field = ELBRecordField::SSLCipher;
                self.skip_next_n_chars = 2;
            },

            ELBRecordField::SSLCipher => {
                self.end_delimiter = SPACE;
                self.next_field = ELBRecordField::SSLProtocol;
                self.skip_next_n_chars = 1;
            },

            ELBRecordField::SSLProtocol => {
                self.end_delimiter = SPACE;
                self.next_field = ELBRecordField::RequestHTTPVersion;
            },
        }
    }
}

//DON'T USE THIS IN YOUR CODE!!!
//This is really an implementation detail and shouldn't be exposed as part of the public API.
//Unfortunately it must be made public in order to implement the Index trait.
//I could use the newtype pattern but the newtype pattern forces another level of indirection
//with no gain besides reducing the exposure a little. I hope that in the future we'll be able to
//implement public methods without having to expose, what should be, private details.
//This behaviour has been changed in 1.7.0 nightly.  This will be made private as soon as 1.7.0 is released.
#[derive(Debug, PartialEq, Clone)]
pub enum ELBRecordField {
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
    RequestHTTPVersion,
    UserAgent,
    SSLCipher,
    SSLProtocol
}

impl<'a> Index<ELBRecordField> for Vec<&'a str> {
    type Output = str;

    fn index(&self, idx: ELBRecordField) -> &str {
        self[idx as usize]
    }
}

impl Display for ELBRecordField {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            ELBRecordField::Timestamp => write!(f, "timestamp"),
            ELBRecordField::ELBName => write!(f, "ELB name"),
            ELBRecordField::ClientAddress => write!(f, "client address"),
            ELBRecordField::BackendAddress => write!(f, "backend address"),
            ELBRecordField::RequestProcessingTime => write!(f, "request processing time"),
            ELBRecordField::BackendProcessingTime => write!(f, "backend processing time"),
            ELBRecordField::ResponseProcessingTime => write!(f, "response processing time"),
            ELBRecordField::ELBStatusCode => write!(f, "ELB status code"),
            ELBRecordField::BackendStatusCode => write!(f, "backend status code"),
            ELBRecordField::ReceivedBytes => write!(f, "received bytes"),
            ELBRecordField::SentBytes => write!(f, "sent bytes"),
            ELBRecordField::RequestMethod => write!(f, "request method"),
            ELBRecordField::RequestURL => write!(f, "request URL"),
            ELBRecordField::RequestHTTPVersion => write!(f, "request HTTP version"),
            ELBRecordField::UserAgent => write!(f, "user agent"),
            ELBRecordField::SSLCipher => write!(f, "SSL cipher"),
            ELBRecordField::SSLProtocol => write!(f, "SSL protocol")
        }
    }
}

trait ELBRecordFieldParser {
    fn parse_field<T>(
        &self,
        field_id: ELBRecordField,
        errors: &mut Vec<ELBRecordParsingError>
    ) -> Option<T>
        where T: FromStr,
        T::Err: Error + 'static;
}

impl<'a> ELBRecordFieldParser for Vec<&'a str> {

    fn parse_field<T>(
        &self,
        field_id: ELBRecordField,
        errors: &mut Vec<ELBRecordParsingError>
    ) -> Option<T>
        where T: FromStr,
        T::Err: Error + 'static,
    {
        let raw_prop = &self[field_id.clone()];
        match raw_prop.parse::<T>() {
            Ok(parsed) => Some(parsed),

            Err(e) => {
                errors.push(
                    ELBRecordParsingError::ParsingError {
                        field_id: field_id,
                        description: e.description().to_owned(),
                    }
                );
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::parse_record;
    use super::ELBRecordParsingError;
    use super::ELBRecordField;

    const V1_TEST_RECORD: &'static str = "2015-08-15T23:43:05.302180Z elb-name 172.16.1.6:54814 \
    172.16.1.5:9000 0.000039 0.145507 0.00003 200 200 0 7582 \
    \"GET http://some.domain.com:80/path0/path1?param0=p0&param1=p1 HTTP/1.1\"\
    ";

    const V2_TEST_RECORD: &'static str = "2015-08-15T23:43:05.302180Z elb-name 172.16.1.6:54814 \
    172.16.1.5:9000 0.000039 0.145507 0.00003 200 200 0 7582 \
    \"GET http://some.domain.com:80/path0/path1?param0=p0&param1=p1 HTTP/1.1\" \
    \"Mozilla/5.0 (cloud; like Mac OS X; en-us) AppleWebKit/537.36.0 (KHTML, like Gecko) Version/4.0.4 Mobile/7B334b Safari/537.36.0\" \
    some_ssl_cipher some_ssl_protocol";

    #[test]
	fn parse_record_returns_a_record_with_the_ssl_protocol_set_to_a_not_available_symbol_when_it_is_not_present() {
        let elb_record = parse_record(V1_TEST_RECORD.to_string()).unwrap();

		assert_eq!(elb_record.ssl_protocol, "-")
	}

    #[test]
	fn parse_record_returns_a_record_with_the_ssl_protocol_when_it_is_present() {
        let elb_record = parse_record(V2_TEST_RECORD.to_string()).unwrap();

		assert_eq!(elb_record.ssl_protocol, "some_ssl_protocol")
	}

    #[test]
	fn parse_record_returns_a_record_with_the_ssl_cipher_set_to_a_not_available_symbol_when_it_is_not_present() {
        let elb_record = parse_record(V1_TEST_RECORD.to_string()).unwrap();

		assert_eq!(elb_record.ssl_cipher, "-")
	}

    #[test]
	fn parse_record_returns_a_record_with_the_ssl_cipher_when_it_is_present() {
        let elb_record = parse_record(V2_TEST_RECORD.to_string()).unwrap();

		assert_eq!(elb_record.ssl_cipher, "some_ssl_cipher")
	}

    #[test]
	fn parse_record_returns_a_record_with_the_user_agent_set_to_a_not_available_symbol_when_it_is_not_present() {
        let elb_record = parse_record(V1_TEST_RECORD.to_string()).unwrap();

		assert_eq!(elb_record.user_agent, "-")
	}

    #[test]
	fn parse_record_returns_a_record_with_the_user_agent_when_it_is_present() {
        let elb_record = parse_record(V2_TEST_RECORD.to_string()).unwrap();

		assert_eq!(elb_record.user_agent, "Mozilla/5.0 (cloud; like Mac OS X; en-us) AppleWebKit/537.36.0 (KHTML, like Gecko) Version/4.0.4 Mobile/7B334b Safari/537.36.0")
	}

    #[test]
	fn parse_record_returns_a_malformed_record_error_for_records_short_on_values() {
        let short_record = "2015-08-15T23:43:05.302180Z elb-name 172.16.1.6:54814 \
        172.16.1.5:9000 0.000039 200 200 0 7582 \
        \"GET http://some.domain.com:80/path0/path1?param0=p0&param1=p1 HTTP/1.1\" \
        ";

        let malformed_error = parse_record(short_record.to_string()).unwrap_err().errors.pop();

		assert_eq!(malformed_error, Some(ELBRecordParsingError::MalformedRecord))
	}

    #[test]
	fn parse_record_returns_a_record_with_the_request_http_version() {
        let elb_record = parse_record(V1_TEST_RECORD.to_string()).unwrap();

		assert_eq!(elb_record.request_http_version, "HTTP/1.1")
	}

    #[test]
	fn parse_record_returns_a_record_with_the_request_url() {
        let elb_record = parse_record(V1_TEST_RECORD.to_string()).unwrap();

		assert_eq!(elb_record.request_url, "http://some.domain.com:80/path0/path1?param0=p0&param1=p1")
	}

    #[test]
	fn parse_record_returns_a_record_with_the_request_method() {
        let elb_record = parse_record(V1_TEST_RECORD.to_string()).unwrap();

		assert_eq!(elb_record.request_method, "GET")
	}

    #[test]
	fn parse_record_returns_a_record_with_the_sent_bytes() {
        let elb_record = parse_record(V1_TEST_RECORD.to_string()).unwrap();

		assert_eq!(elb_record.sent_bytes, 7582)
	}

    #[test]
    fn parse_record_returns_a_parsing_error_referencing_the_sent_bytes_when_the_sent_bytes_is_malformed() {
          let bad_record = "2015-08-15T23:43:05.302180Z elb-name 172.16.1.6:54814 \
          172.16.1.5:9000 0.000039 0.145507 0.00003 200 200 0 bad_sent_bytes \
          \"GET http://some.domain.com:80/path0/path1?param0=p0&param1=p1 HTTP/1.1\"\
          ";

          let error_field_id = match parse_record(bad_record.to_string()).unwrap_err().errors.pop().unwrap() {
              ELBRecordParsingError::ParsingError { field_id, .. } => field_id,
              _ => panic!(),
          };

      assert_eq!(error_field_id, ELBRecordField::SentBytes)
    }

    #[test]
	fn parse_record_returns_a_record_with_the_received_bytes() {
        let elb_record = parse_record(V1_TEST_RECORD.to_string()).unwrap();

		assert_eq!(elb_record.received_bytes, 0)
	}

    #[test]
    fn parse_record_returns_a_parsing_error_referencing_the_received_bytes_when_the_received_bytes_is_malformed() {
          let bad_record = "2015-08-15T23:43:05.302180Z elb-name 172.16.1.6:54814 \
          172.16.1.5:9000 0.000039 0.145507 0.00003 200 200 bad_received_bytes 7582 \
          \"GET http://some.domain.com:80/path0/path1?param0=p0&param1=p1 HTTP/1.1\"\
          ";

          let error_field_id = match parse_record(bad_record.to_string()).unwrap_err().errors.pop().unwrap() {
              ELBRecordParsingError::ParsingError { field_id, .. } => field_id,
              _ => panic!(),
          };

      assert_eq!(error_field_id, ELBRecordField::ReceivedBytes)
    }

    #[test]
	fn parse_record_returns_a_record_with_the_backend_status_code() {
        let elb_record = parse_record(V1_TEST_RECORD.to_string()).unwrap();

		assert_eq!(elb_record.backend_status_code, 200)
	}

    #[test]
    fn parse_record_returns_a_parsing_error_referencing_the_backend_status_code_when_the_backend_status_code_is_malformed() {
          let bad_record = "2015-08-15T23:43:05.302180Z elb-name 172.16.1.6:54814 \
          172.16.1.5:9000 0.000039 0.145507 0.00003 200 bad_backend_status_code 0 7582 \
          \"GET http://some.domain.com:80/path0/path1?param0=p0&param1=p1 HTTP/1.1\"\
          ";

          let error_field_id = match parse_record(bad_record.to_string()).unwrap_err().errors.pop().unwrap() {
              ELBRecordParsingError::ParsingError { field_id, .. } => field_id,
              _ => panic!(),
          };

      assert_eq!(error_field_id, ELBRecordField::BackendStatusCode)
    }

    #[test]
	fn parse_record_returns_a_record_with_the_elb_status_code() {
        let elb_record = parse_record(V1_TEST_RECORD.to_string()).unwrap();

		assert_eq!(elb_record.elb_status_code, 200)
	}

    #[test]
    fn parse_record_returns_a_parsing_error_referencing_the_elb_status_code_when_the_elb_status_code_is_malformed() {
          let bad_record = "2015-08-15T23:43:05.302180Z elb-name 172.16.1.6:54814 \
          172.16.1.5:9000 0.000039 0.145507 0.00003 bad_elb_status_code 200 0 7582 \
          \"GET http://some.domain.com:80/path0/path1?param0=p0&param1=p1 HTTP/1.1\"\
          ";

          let error_field_id = match parse_record(bad_record.to_string()).unwrap_err().errors.pop().unwrap() {
              ELBRecordParsingError::ParsingError { field_id, .. } => field_id,
              _ => panic!(),
          };

      assert_eq!(error_field_id, ELBRecordField::ELBStatusCode)
    }

    #[test]
	fn parse_record_returns_a_record_with_the_response_processing_time() {
        let elb_record = parse_record(V1_TEST_RECORD.to_string()).unwrap();

		assert_eq!(elb_record.response_processing_time, 0.00003)
	}

    #[test]
    fn parse_record_returns_a_parsing_error_referencing_the_response_processing_time_when_the_response_processing_time_is_malformed() {
          let bad_record = "2015-08-15T23:43:05.302180Z elb-name 172.16.1.6:54814 \
          172.16.1.5:9000 0.000039 0.145507 bad_response_processing_time 200 200 0 7582 \
          \"GET http://some.domain.com:80/path0/path1?param0=p0&param1=p1 HTTP/1.1\"\
          ";

          let error_field_id = match parse_record(bad_record.to_string()).unwrap_err().errors.pop().unwrap() {
              ELBRecordParsingError::ParsingError { field_id, .. } => field_id,
              _ => panic!(),
          };

      assert_eq!(error_field_id, ELBRecordField::ResponseProcessingTime)
    }

    #[test]
	fn parse_record_returns_a_record_with_the_backend_processing_time() {
        let elb_record = parse_record(V1_TEST_RECORD.to_string()).unwrap();

		assert_eq!(elb_record.backend_processing_time, 0.145507)
	}

    #[test]
    fn parse_record_returns_a_parsing_error_referencing_the_backend_processing_time_when_the_backend_processing_time_is_malformed() {
          let bad_record = "2015-08-15T23:43:05.302180Z elb-name 172.16.1.6:54814 \
          172.16.1.5:9000 0.000039 bad_backend_processing_time 0.00003 200 200 0 7582 \
          \"GET http://some.domain.com:80/path0/path1?param0=p0&param1=p1 HTTP/1.1\"\
          ";

          let error_field_id = match parse_record(bad_record.to_string()).unwrap_err().errors.pop().unwrap() {
              ELBRecordParsingError::ParsingError { field_id, .. } => field_id,
              _ => panic!(),
          };

      assert_eq!(error_field_id, ELBRecordField::BackendProcessingTime)
    }

    #[test]
	fn parse_record_returns_a_record_with_the_request_processing_time() {
        let elb_record = parse_record(V1_TEST_RECORD.to_string()).unwrap();

		assert_eq!(elb_record.request_processing_time, 0.000039)
	}

    #[test]
    fn parse_record_returns_a_parsing_error_referencing_the_request_processing_time_when_the_request_processing_time_is_malformed() {
          let bad_record = "2015-08-15T23:43:05.302180Z elb-name 172.16.1.6:54814 \
          172.16.1.5:9000 bad_request_processing_time 0.145507 0.00003 200 200 0 7582 \
          \"GET http://some.domain.com:80/path0/path1?param0=p0&param1=p1 HTTP/1.1\"\
          ";

          let error_field_id = match parse_record(bad_record.to_string()).unwrap_err().errors.pop().unwrap() {
              ELBRecordParsingError::ParsingError { field_id, .. } => field_id,
              _ => panic!(),
          };

      assert_eq!(error_field_id, ELBRecordField::RequestProcessingTime)
    }

    #[test]
	fn parse_record_returns_a_record_with_the_backend_address() {
        let elb_record = parse_record(V1_TEST_RECORD.to_string()).unwrap();

		assert_eq!(elb_record.backend_address, "172.16.1.5:9000".parse().unwrap())
	}

    #[test]
	fn parse_record_returns_a_parsing_error_referencing_the_backend_address_when_the_backend_address_is_malformed() {
        let bad_record = "2015-08-15T23:43:05.302180Z elb-name 172.16.1.6:54814 \
        bad_backend_address 0.000039 0.145507 0.00003 200 200 0 7582 \
        \"GET http://some.domain.com:80/path0/path1?param0=p0&param1=p1 HTTP/1.1\"\
        ";

        let error_field_id = match parse_record(bad_record.to_string()).unwrap_err().errors.pop().unwrap() {
            ELBRecordParsingError::ParsingError { field_id, .. } => field_id,
            _ => panic!(),
        };

		assert_eq!(error_field_id, ELBRecordField::BackendAddress)
	}

    #[test]
	fn parse_record_returns_a_record_with_the_client_address() {
        let elb_record = parse_record(V1_TEST_RECORD.to_string()).unwrap();

		assert_eq!(elb_record.client_address, "172.16.1.6:54814".parse().unwrap())
	}

    #[test]
	fn parse_record_returns_a_parsing_error_referencing_the_client_address_when_the_client_address_is_malformed() {
        let bad_record = "2015-08-15T23:43:05.302180Z elb-name bad_client_address \
        172.16.1.5:9000 0.000039 0.145507 0.00003 200 200 0 7582 \
        \"GET http://some.domain.com:80/path0/path1?param0=p0&param1=p1 HTTP/1.1\"\
        ";

        let error_field_id = match parse_record(bad_record.to_string()).unwrap_err().errors.pop().unwrap() {
            ELBRecordParsingError::ParsingError { field_id, .. } => field_id,
            _ => panic!(),
        };

		assert_eq!(error_field_id, ELBRecordField::ClientAddress)
	}

    #[test]
	fn parse_record_returns_a_record_with_the_timestamp() {
        let elb_record = parse_record(V1_TEST_RECORD.to_string()).unwrap();

		assert_eq!(format!("{:?}", elb_record.timestamp), "2015-08-15T23:43:05.302180Z")
	}

    #[test]
	fn parse_record_returns_a_parsing_error_referencing_the_timestamp_when_the_timestamp_is_malformed() {
        let bad_record = "bad_timestamp elb-name 172.16.1.6:54814 \
        172.16.1.5:9000 0.000039 0.145507 0.00003 200 200 0 7582 \
        \"GET http://some.domain.com:80/path0/path1?param0=p0&param1=p1 HTTP/1.1\"\
        ";

        let error_field_id = match parse_record(bad_record.to_string()).unwrap_err().errors.pop().unwrap() {
            ELBRecordParsingError::ParsingError { field_id, .. } => field_id,
            _ => panic!(),
        };

		assert_eq!(error_field_id, ELBRecordField::Timestamp)
	}

    #[test]
	fn parse_record_returns_a_record_with_the_elb_name() {
        let elb_record = parse_record(V1_TEST_RECORD.to_string()).unwrap();

		assert_eq!(elb_record.elb_name, "elb-name")
	}
}
