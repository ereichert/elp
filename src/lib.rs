extern crate chrono;
#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;

use self::chrono::{DateTime, UTC};
use std::error::Error;
use std::str::FromStr;
use std::net::SocketAddrV4;
use std::fmt::{Display, Formatter};
use std::fmt;
use std::ops::Index;

// AWS doesn't version their log file format so these version numbers were
// selected by me to bring some sanity to the various formats.
const ELB_RECORD_V1_FIELD_COUNT: usize = 14;
const ELB_RECORD_V2_FIELD_COUNT: usize = 17;
const UNDEFINED_CHAR: &str = "-";

/// The product of parsing a single AWS ELB log record.
#[derive(Debug)]
pub struct ELBRecord<'a> {
    pub timestamp: DateTime<UTC>,
    pub elb_name: &'a str,
    pub client_address: SocketAddrV4,
    pub backend_address: SocketAddrV4,
    pub request_processing_time: f32,
    pub backend_processing_time: f32,
    pub response_processing_time: f32,
    pub elb_status_code: u16,
    pub backend_status_code: u16,
    pub received_bytes: u64,
    pub sent_bytes: u64,
    pub request_method: &'a str,
    pub request_url: &'a str,
    pub request_http_version: &'a str,
    pub user_agent: &'a str,
    pub ssl_cipher: &'a str,
    pub ssl_protocol: &'a str,
}

/// The result of an attempt to parse an ELB record.
pub type ParsingResult<'a> = Result<ELBRecord<'a>, ParsingErrors<'a>>;

/// The result of a failed attempt to parse an ELB record.
///
/// It is very possible that multiple fields of a record are not parsable.  An attempt is made to
/// parse all of the fields of an ELB record.  An error is returned for each field that was not
/// parsable to make it clear what fields of the record were faulty and allow the user to decide
/// how to handle the failure.
#[derive(Debug, PartialEq)]
pub struct ParsingErrors<'a> {
    /// The raw record.
    pub record: &'a str,
    /// A collection of parsing errors such as fields that could not be parsed or a failure to
    /// open an ELB log file.
    pub errors: Vec<ELBRecordParsingError>,
}

/// Specific parsing errors that are returned as part of the [`ParsingErrors::errors`]
/// (struct.ParsingErrors.html) collection.
#[derive(Debug, PartialEq)]
pub enum ELBRecordParsingError {
    /// Returned if the record does not have the correct number of fields.
    MalformedRecord,
    /// A failed attempt to parse a specific field of the ELB record.
    ParsingError {
        field_name: ELBRecordField,
        description: String,
    },
}

impl Display for ELBRecordParsingError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            ELBRecordParsingError::MalformedRecord => write!(f, "Record is malformed."),
            ELBRecordParsingError::ParsingError { ref field_name, ref description } => {
                write!(f,
                       "Parsing of field {} failed with the following error: {}.",
                       field_name,
                       description)
            }
        }
    }
}

impl Error for ELBRecordParsingError {
    fn description(&self) -> &str {
        match *self {
            ELBRecordParsingError::MalformedRecord => "malformed record",
            ELBRecordParsingError::ParsingError { .. } => "field parsing failed",
        }
    }
}

/// Attempt to parse a single string into an ELB record.
///
/// This is the main parsing algorithm.  It will attempt to parse every field that is supposed to
/// be in an ELB Access log record.  If it successful it will return an `Ok(ELBRecord)`.  If not,
/// it will return a `Err(ParsingErrors)`.
pub fn parse_record(record: &str) -> ParsingResult {
    let mut errors: Vec<ELBRecordParsingError> = Vec::new();
    let split_record: Vec<&str> = record.split_record();
    let split_len = split_record.len();
    if split_len != ELB_RECORD_V1_FIELD_COUNT && split_len != ELB_RECORD_V2_FIELD_COUNT {
        errors.push(ELBRecordParsingError::MalformedRecord);
        return Err(ParsingErrors {
            record,
            errors,
        });
    }

    let ts = split_record.parse_field(ELBRecordField::Timestamp, &mut errors);
    let clnt_addr = split_record.parse_field(ELBRecordField::ClientAddress, &mut errors);
    let be_addr = split_record.parse_field(ELBRecordField::BackendAddress, &mut errors);
    let req_proc_time =
        split_record.parse_field(ELBRecordField::RequestProcessingTime, &mut errors);
    let be_proc_time = split_record.parse_field(ELBRecordField::BackendProcessingTime, &mut errors);
    let res_proc_time =
        split_record.parse_field(ELBRecordField::ResponseProcessingTime, &mut errors);
    let elb_sc = split_record.parse_field(ELBRecordField::ELBStatusCode, &mut errors);
    let be_sc = split_record.parse_field(ELBRecordField::BackendStatusCode, &mut errors);
    let bytes_received = split_record.parse_field(ELBRecordField::ReceivedBytes, &mut errors);
    let bytes_sent = split_record.parse_field(ELBRecordField::SentBytes, &mut errors);
    let (user_agent, ssl_cipher, ssl_protocol) = if split_len == ELB_RECORD_V2_FIELD_COUNT {
        (split_record[ELBRecordField::UserAgent],
         split_record[ELBRecordField::SSLCipher],
         split_record[ELBRecordField::SSLProtocol])

    } else {
        (UNDEFINED_CHAR, UNDEFINED_CHAR, UNDEFINED_CHAR)
    };

    if errors.is_empty() {
        // If errors is empty it is more than likely parsing was successful and unwrap
        // is safe.
        Ok(ELBRecord {
            timestamp: ts.unwrap(),
            elb_name: split_record[ELBRecordField::ELBName],
            client_address: clnt_addr.unwrap(),
            backend_address: be_addr.unwrap(),
            request_processing_time: req_proc_time.unwrap(),
            backend_processing_time: be_proc_time.unwrap(),
            response_processing_time: res_proc_time.unwrap(),
            elb_status_code: elb_sc.unwrap(),
            backend_status_code: be_sc.unwrap(),
            received_bytes: bytes_received.unwrap(),
            sent_bytes: bytes_sent.unwrap(),
            request_method: split_record[ELBRecordField::RequestMethod],
            request_url: split_record[ELBRecordField::RequestURL],
            request_http_version: split_record[ELBRecordField::RequestHTTPVersion],
            user_agent,
            ssl_cipher,
            ssl_protocol,
        })
    } else {
        Err(ParsingErrors {
            record,
            errors,
        })
    }
}

trait RecordSplitter {
    fn split_record(&self) -> Vec<&str>;
}

impl RecordSplitter for str {
    fn split_record(&self) -> Vec<&str> {
        let mut split_record: Vec<&str> = Vec::with_capacity(ELB_RECORD_V2_FIELD_COUNT);
        let mut field_specs_idx = 0;
        let mut current_field_spec = &ORDERED_FIELD_SPECS[field_specs_idx];
        let mut current_start_delim = current_field_spec.start_delimiter;
        let mut start_of_field_index = 0;

        for (current_idx, current_char) in self.trim_start().char_indices() {
            match current_start_delim {
                None if current_char == current_field_spec.end_delimiter => {
                    split_record.push(&self[start_of_field_index..current_idx]);
                    start_of_field_index = current_idx + 1;
                    field_specs_idx += 1;
                    if field_specs_idx < ELB_RECORD_V2_FIELD_COUNT {
                        current_field_spec = &ORDERED_FIELD_SPECS[field_specs_idx];
                        current_start_delim = current_field_spec.start_delimiter;
                    }
                }
                Some(sd) if current_char == sd => {
                    start_of_field_index = current_idx + 1;
                    current_start_delim = None;
                }
                _ => {}
            }
        }

        let x = &self[start_of_field_index..];
        if !x.is_empty() {
            split_record.push(x);
        }

        debug!("{:?}", split_record);
        split_record
    }
}

const SPACE: char = ' ';
const DOUBLE_QUOTE: char = '"';
lazy_static! {
    static ref ORDERED_FIELD_SPECS: Vec<ELBRecordFieldParsingSpec> = vec!(
        ELBRecordFieldParsingSpec {
            field: ELBRecordField::Timestamp,
            start_delimiter: None,
            end_delimiter: SPACE,
        },
        ELBRecordFieldParsingSpec {
            field: ELBRecordField::ELBName,
            start_delimiter: None,
            end_delimiter: SPACE,
        },
        ELBRecordFieldParsingSpec {
            field: ELBRecordField::ClientAddress,
            start_delimiter: None,
            end_delimiter: SPACE,
        },
        ELBRecordFieldParsingSpec {
            field: ELBRecordField::BackendAddress,
            start_delimiter: None,
            end_delimiter: SPACE,
        },
        ELBRecordFieldParsingSpec {
            field: ELBRecordField::RequestProcessingTime,
            start_delimiter: None,
            end_delimiter: SPACE,
        },
        ELBRecordFieldParsingSpec {
            field: ELBRecordField::BackendProcessingTime,
            start_delimiter: None,
            end_delimiter: SPACE,
        },
        ELBRecordFieldParsingSpec {
            field: ELBRecordField::ResponseProcessingTime,
            start_delimiter: None,
            end_delimiter: SPACE,
        },
        ELBRecordFieldParsingSpec {
            field: ELBRecordField::ELBStatusCode,
            start_delimiter: None,
            end_delimiter: SPACE,
        },
        ELBRecordFieldParsingSpec {
            field: ELBRecordField::BackendStatusCode,
            start_delimiter: None,
            end_delimiter: SPACE,
        },
        ELBRecordFieldParsingSpec {
            field: ELBRecordField::ReceivedBytes,
            start_delimiter: None,
            end_delimiter: SPACE,
        },
        ELBRecordFieldParsingSpec {
            field: ELBRecordField::SentBytes,
            start_delimiter: None,
            end_delimiter: SPACE,
        },
        ELBRecordFieldParsingSpec {
            field: ELBRecordField::RequestMethod,
            start_delimiter: Some(DOUBLE_QUOTE),
            end_delimiter: SPACE,
        },
        ELBRecordFieldParsingSpec {
            field: ELBRecordField::RequestURL,
            start_delimiter: None,
            end_delimiter: SPACE,
        },
        ELBRecordFieldParsingSpec {
            field: ELBRecordField::RequestHTTPVersion,
            start_delimiter: None,
            end_delimiter: DOUBLE_QUOTE,
        },
        ELBRecordFieldParsingSpec {
            field: ELBRecordField::UserAgent,
            start_delimiter: Some(DOUBLE_QUOTE),
            end_delimiter: DOUBLE_QUOTE,
        },
        ELBRecordFieldParsingSpec {
            field: ELBRecordField::SSLCipher,
            start_delimiter: Some(SPACE),
            end_delimiter: SPACE,
        },
        ELBRecordFieldParsingSpec {
            field: ELBRecordField::SSLProtocol,
            start_delimiter: None,
            end_delimiter: SPACE,
        },
    );
}

#[derive(Debug)]
struct ELBRecordFieldParsingSpec {
    field: ELBRecordField,
    start_delimiter: Option<char>,
    end_delimiter: char,
}

#[derive(Debug, PartialEq, Clone, Copy)]
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
    SSLProtocol,
}

impl<'a> Index<ELBRecordField> for Vec<&'a str> {
    type Output = &'a str;

    fn index(&self, idx: ELBRecordField) -> &&'a str {
        &self[idx as usize]
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
            ELBRecordField::SSLProtocol => write!(f, "SSL protocol"),
        }
    }
}

trait ELBRecordFieldParser {
    fn parse_field<T>(&self,
                      field_name: ELBRecordField,
                      errors: &mut Vec<ELBRecordParsingError>)
                      -> Option<T>
        where T: FromStr,
              T::Err: Error + 'static;
}

impl<'a> ELBRecordFieldParser for Vec<&'a str> {
    fn parse_field<T>(&self,
                      field_name: ELBRecordField,
                      errors: &mut Vec<ELBRecordParsingError>)
                      -> Option<T>
        where T: FromStr,
              T::Err: Error + 'static
    {
        let raw_prop = self[field_name];
        match raw_prop.parse::<T>() {
            Ok(parsed) => Some(parsed),

            Err(e) => {
                errors.push(ELBRecordParsingError::ParsingError {
                    field_name,
                    description: e.to_string(),
                });
                None
            }
        }
    }
}

#[cfg(test)]
mod parse_record_tests {
    use super::parse_record;
    use super::ELBRecordParsingError;
    use super::ELBRecordField;
    use super::UNDEFINED_CHAR;

    const V1_TEST_RECORD: &str = "2015-08-15T23:43:05.302180Z elb-name 172.16.1.6:54814 \
    172.16.1.5:9000 0.000039 0.145507 0.00003 200 200 0 7582 \
    \"GET http://some.domain.com:80/path0/path1?param0=p0&param1=p1 HTTP/1.1\"\
    ";

    const V2_TEST_RECORD: &str =
        "2015-08-15T23:43:05.302180Z elb-name 172.16.1.6:54814 172.16.1.5:9000 0.000039 0.145507 \
         0.00003 200 200 0 7582 \"GET http://some.domain.com:80/path0/path1?param0=p0&param1=p1 \
         HTTP/1.1\" \"Mozilla/5.0 (cloud; like Mac OS X; en-us) AppleWebKit/537.36.0 (KHTML, like \
         Gecko) Version/4.0.4 Mobile/7B334b Safari/537.36.0\" some_ssl_cipher some_ssl_protocol";

    #[test]
    fn returns_a_record_with_the_ssl_protocol_set_to_a_not_available_symbol_when_it_is_not_present
        () {
        let elb_record = parse_record(V1_TEST_RECORD).unwrap();

        assert_eq!(elb_record.ssl_protocol, UNDEFINED_CHAR)
    }

    #[test]
    fn returns_a_record_with_the_ssl_protocol_when_it_is_present() {
        let elb_record = parse_record(V2_TEST_RECORD).unwrap();

        assert_eq!(elb_record.ssl_protocol, "some_ssl_protocol")
    }

    #[test]
    fn returns_a_record_with_the_ssl_cipher_set_to_a_not_available_symbol_when_it_is_not_present
        () {
        let elb_record = parse_record(V1_TEST_RECORD).unwrap();

        assert_eq!(elb_record.ssl_cipher, UNDEFINED_CHAR)
    }

    #[test]
    fn returns_a_record_with_the_ssl_cipher_when_it_is_present() {
        let elb_record = parse_record(V2_TEST_RECORD).unwrap();

        assert_eq!(elb_record.ssl_cipher, "some_ssl_cipher")
    }

    #[test]
    fn returns_a_record_with_the_user_agent_set_to_a_not_available_symbol_when_it_is_not_present
        () {
        let elb_record = parse_record(V1_TEST_RECORD).unwrap();

        assert_eq!(elb_record.user_agent, UNDEFINED_CHAR)
    }

    #[test]
    fn returns_a_record_with_the_user_agent_when_it_is_present() {
        let elb_record = parse_record(V2_TEST_RECORD).unwrap();

        assert_eq!(elb_record.user_agent,
                   "Mozilla/5.0 (cloud; like Mac OS X; en-us) AppleWebKit/537.36.0 (KHTML, like \
                    Gecko) Version/4.0.4 Mobile/7B334b Safari/537.36.0")
    }

    #[test]
    fn returns_a_malformed_record_error_for_records_short_on_values() {
        let short_record = "2015-08-15T23:43:05.302180Z elb-name 172.16.1.6:54814 \
        172.16.1.5:9000 0.000039 200 200 0 7582 \
        \"GET http://some.domain.com:80/path0/path1?param0=p0&param1=p1 HTTP/1.1\" \
        ";

        let malformed_error = parse_record(short_record).unwrap_err().errors.pop();

        assert_eq!(malformed_error,
                   Some(ELBRecordParsingError::MalformedRecord))
    }

    #[test]
    fn returns_a_record_with_the_request_http_version() {
        let elb_record = parse_record(V1_TEST_RECORD).unwrap();

        assert_eq!(elb_record.request_http_version, "HTTP/1.1")
    }

    #[test]
    fn returns_a_record_with_the_request_url() {
        let elb_record = parse_record(V1_TEST_RECORD).unwrap();

        assert_eq!(elb_record.request_url,
                   "http://some.domain.com:80/path0/path1?param0=p0&param1=p1")
    }

    #[test]
    fn returns_a_record_with_the_request_method() {
        let elb_record = parse_record(V1_TEST_RECORD).unwrap();

        assert_eq!(elb_record.request_method, "GET")
    }

    #[test]
    fn returns_a_record_with_the_sent_bytes() {
        let elb_record = parse_record(V1_TEST_RECORD).unwrap();

        assert_eq!(elb_record.sent_bytes, 7582)
    }

    #[test]
    fn returns_a_parsing_error_referencing_the_sent_bytes_when_the_sent_bytes_is_malformed() {
        let bad_record = "2015-08-15T23:43:05.302180Z elb-name 172.16.1.6:54814 172.16.1.5:9000 \
                          0.000039 0.145507 0.00003 200 200 0 bad_sent_bytes \"GET \
                          http://some.domain.com:80/path0/path1?param0=p0&param1=p1 HTTP/1.1\"";

        let error_field_name = match parse_record(bad_record).unwrap_err().errors.pop().unwrap() {
            ELBRecordParsingError::ParsingError { field_name, .. } => field_name,
            _ => panic!(),
        };

        assert_eq!(error_field_name, ELBRecordField::SentBytes)
    }

    #[test]
    fn returns_a_record_with_the_received_bytes() {
        let elb_record = parse_record(V1_TEST_RECORD).unwrap();

        assert_eq!(elb_record.received_bytes, 0)
    }

    #[test]
    fn returns_a_parsing_error_referencing_the_received_bytes_when_the_received_bytes_is_malformed
        () {
        let bad_record = "2015-08-15T23:43:05.302180Z elb-name 172.16.1.6:54814 172.16.1.5:9000 \
                          0.000039 0.145507 0.00003 200 200 bad_received_bytes 7582 \"GET \
                          http://some.domain.com:80/path0/path1?param0=p0&param1=p1 HTTP/1.1\"";

        let error_field_name = match parse_record(bad_record).unwrap_err().errors.pop().unwrap() {
            ELBRecordParsingError::ParsingError { field_name, .. } => field_name,
            _ => panic!(),
        };

        assert_eq!(error_field_name, ELBRecordField::ReceivedBytes)
    }

    #[test]
    fn returns_a_record_with_the_backend_status_code() {
        let elb_record = parse_record(V1_TEST_RECORD).unwrap();

        assert_eq!(elb_record.backend_status_code, 200)
    }

    #[test]
    fn returns_a_parsing_error_when_the_backend_status_code_is_malformed() {
        let bad_record = "2015-08-15T23:43:05.302180Z elb-name 172.16.1.6:54814 172.16.1.5:9000 \
                          0.000039 0.145507 0.00003 200 bad_backend_status_code 0 7582 \"GET \
                          http://some.domain.com:80/path0/path1?param0=p0&param1=p1 HTTP/1.1\"";

        let error_field_name = match parse_record(bad_record).unwrap_err().errors.pop().unwrap() {
            ELBRecordParsingError::ParsingError { field_name, .. } => field_name,
            _ => panic!(),
        };

        assert_eq!(error_field_name, ELBRecordField::BackendStatusCode)
    }

    #[test]
    fn returns_a_record_with_the_elb_status_code() {
        let elb_record = parse_record(V1_TEST_RECORD).unwrap();

        assert_eq!(elb_record.elb_status_code, 200)
    }

    #[test]
    fn returns_a_parsing_error_when_the_elb_status_code_is_malformed() {
        let bad_record = "2015-08-15T23:43:05.302180Z elb-name 172.16.1.6:54814 172.16.1.5:9000 \
                          0.000039 0.145507 0.00003 bad_elb_status_code 200 0 7582 \"GET \
                          http://some.domain.com:80/path0/path1?param0=p0&param1=p1 HTTP/1.1\"";

        let error_field_name = match parse_record(bad_record).unwrap_err().errors.pop().unwrap() {
            ELBRecordParsingError::ParsingError { field_name, .. } => field_name,
            _ => panic!(),
        };

        assert_eq!(error_field_name, ELBRecordField::ELBStatusCode)
    }

    #[test]
    fn returns_a_record_with_the_response_processing_time() {
        let elb_record = parse_record(V1_TEST_RECORD).unwrap();

        assert_eq!(elb_record.response_processing_time, 0.00003)
    }

    #[test]
    fn returns_a_parsing_error_when_the_response_processing_time_is_malformed() {
        let bad_record = "2015-08-15T23:43:05.302180Z elb-name 172.16.1.6:54814 172.16.1.5:9000 \
                          0.000039 0.145507 bad_response_processing_time 200 200 0 7582 \"GET \
                          http://some.domain.com:80/path0/path1?param0=p0&param1=p1 HTTP/1.1\"";

        let error_field_name = match parse_record(bad_record).unwrap_err().errors.pop().unwrap() {
            ELBRecordParsingError::ParsingError { field_name, .. } => field_name,
            _ => panic!(),
        };

        assert_eq!(error_field_name, ELBRecordField::ResponseProcessingTime)
    }

    #[test]
    fn returns_a_record_with_the_backend_processing_time() {
        let elb_record = parse_record(V1_TEST_RECORD).unwrap();

        assert_eq!(elb_record.backend_processing_time, 0.145507)
    }

    #[test]
    fn returns_a_parsing_error_when_the_backend_processing_time_is_malformed() {
        let bad_record = "2015-08-15T23:43:05.302180Z elb-name 172.16.1.6:54814 172.16.1.5:9000 \
                          0.000039 bad_backend_processing_time 0.00003 200 200 0 7582 \"GET \
                          http://some.domain.com:80/path0/path1?param0=p0&param1=p1 HTTP/1.1\"";

        let error_field_name = match parse_record(bad_record).unwrap_err().errors.pop().unwrap() {
            ELBRecordParsingError::ParsingError { field_name, .. } => field_name,
            _ => panic!(),
        };

        assert_eq!(error_field_name, ELBRecordField::BackendProcessingTime)
    }

    #[test]
    fn returns_a_record_with_the_request_processing_time() {
        let elb_record = parse_record(V1_TEST_RECORD).unwrap();

        assert_eq!(elb_record.request_processing_time, 0.000039)
    }

    #[test]
    fn returns_a_parsing_error_when_the_request_processing_time_is_malformed() {
        let bad_record = "2015-08-15T23:43:05.302180Z elb-name 172.16.1.6:54814 172.16.1.5:9000 \
                          bad_request_processing_time 0.145507 0.00003 200 200 0 7582 \"GET \
                          http://some.domain.com:80/path0/path1?param0=p0&param1=p1 HTTP/1.1\"";

        let error_field_name = match parse_record(bad_record).unwrap_err().errors.pop().unwrap() {
            ELBRecordParsingError::ParsingError { field_name, .. } => field_name,
            _ => panic!(),
        };

        assert_eq!(error_field_name, ELBRecordField::RequestProcessingTime)
    }

    #[test]
    fn returns_a_record_with_the_backend_address() {
        let elb_record = parse_record(V1_TEST_RECORD).unwrap();

        assert_eq!(elb_record.backend_address,
                   "172.16.1.5:9000".parse().unwrap())
    }

    #[test]
    fn returns_a_parsing_error_when_the_backend_address_is_malformed() {
        let bad_record = "2015-08-15T23:43:05.302180Z elb-name 172.16.1.6:54814 \
        bad_backend_address 0.000039 0.145507 0.00003 200 200 0 7582 \
        \"GET http://some.domain.com:80/path0/path1?param0=p0&param1=p1 HTTP/1.1\"\
        ";

        let error_field_name = match parse_record(bad_record).unwrap_err().errors.pop().unwrap() {
            ELBRecordParsingError::ParsingError { field_name, .. } => field_name,
            _ => panic!(),
        };

        assert_eq!(error_field_name, ELBRecordField::BackendAddress)
    }

    #[test]
    fn returns_a_record_with_the_client_address() {
        let elb_record = parse_record(V1_TEST_RECORD).unwrap();

        assert_eq!(elb_record.client_address,
                   "172.16.1.6:54814".parse().unwrap())
    }

    #[test]
    fn returns_a_parsing_error_referencing_the_client_address_when_the_client_address_is_malformed
        () {
        let bad_record = "2015-08-15T23:43:05.302180Z elb-name bad_client_address \
        172.16.1.5:9000 0.000039 0.145507 0.00003 200 200 0 7582 \
        \"GET http://some.domain.com:80/path0/path1?param0=p0&param1=p1 HTTP/1.1\"\
        ";

        let error_field_name = match parse_record(bad_record).unwrap_err().errors.pop().unwrap() {
            ELBRecordParsingError::ParsingError { field_name, .. } => field_name,
            _ => panic!(),
        };

        assert_eq!(error_field_name, ELBRecordField::ClientAddress)
    }

    #[test]
    fn returns_a_record_with_the_timestamp() {
        let elb_record = parse_record(V1_TEST_RECORD).unwrap();

        assert_eq!(format!("{:?}", elb_record.timestamp),
                   "2015-08-15T23:43:05.302180Z")
    }

    #[test]
    fn returns_a_parsing_error_referencing_the_timestamp_when_the_timestamp_is_malformed() {
        let bad_record = "bad_timestamp elb-name 172.16.1.6:54814 \
        172.16.1.5:9000 0.000039 0.145507 0.00003 200 200 0 7582 \
        \"GET http://some.domain.com:80/path0/path1?param0=p0&param1=p1 HTTP/1.1\"\
        ";

        let error_field_name = match parse_record(bad_record).unwrap_err().errors.pop().unwrap() {
            ELBRecordParsingError::ParsingError { field_name, .. } => field_name,
            _ => panic!(),
        };

        assert_eq!(error_field_name, ELBRecordField::Timestamp)
    }

    #[test]
    fn returns_a_record_with_the_elb_name() {
        let elb_record = parse_record(V1_TEST_RECORD).unwrap();

        assert_eq!(elb_record.elb_name, "elb-name")
    }
}
