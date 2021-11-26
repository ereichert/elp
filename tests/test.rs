use elp::{parse_record, ELBRecordField, ELBRecordParsingError};

const V1_TEST_RECORD: &str = "2015-08-15T23:43:05.302180Z elb-name 172.16.1.6:54814 172.16.1.5:9000 0.000039 0.145507 0.00003 200 200 0 7582 \"GET http://some.domain.com:80/path0/path1?param0=p0&param1=p1 HTTP/1.1\"";
const V2_TEST_RECORD: &str = "2015-08-15T23:43:05.302180Z elb-name 172.16.1.6:54814 172.16.1.5:9000 0.000039 0.145507 0.00003 200 200 0 7582 \"GET http://some.domain.com:80/path0/path1?param0=p0&param1=p1 HTTP/1.1\" \"Mozilla/5.0 (cloud; like Mac OS X; en-us) AppleWebKit/537.36.0 (KHTML, like Gecko) Version/4.0.4 Mobile/7B334b Safari/537.36.0\" some_ssl_cipher some_ssl_protocol";

#[test]
fn returns_a_record_with_the_ssl_protocol_set_to_a_not_available_symbol_when_it_is_not_present()
{
    let elb_record = parse_record(V1_TEST_RECORD).unwrap();

    assert_eq!(elb_record.ssl_protocol, "-")
}

#[test]
fn returns_a_record_with_the_ssl_protocol_when_it_is_present() {
    let elb_record = parse_record(V2_TEST_RECORD).unwrap();

    assert_eq!(elb_record.ssl_protocol, "some_ssl_protocol")
}

#[test]
fn returns_a_record_with_the_ssl_cipher_set_to_a_not_available_symbol_when_it_is_not_present() {
    let elb_record = parse_record(V1_TEST_RECORD).unwrap();

    assert_eq!(elb_record.ssl_cipher, "-")
}

#[test]
fn returns_a_record_with_the_ssl_cipher_when_it_is_present() {
    let elb_record = parse_record(V2_TEST_RECORD).unwrap();

    assert_eq!(elb_record.ssl_cipher, "some_ssl_cipher")
}

#[test]
fn returns_a_record_with_the_user_agent_set_to_a_not_available_symbol_when_it_is_not_present() {
    let elb_record = parse_record(V1_TEST_RECORD).unwrap();

    assert_eq!(elb_record.user_agent, "-")
}

#[test]
fn returns_a_record_with_the_user_agent_when_it_is_present() {
    let elb_record = parse_record(V2_TEST_RECORD).unwrap();

    assert_eq!(
        elb_record.user_agent,
        "Mozilla/5.0 (cloud; like Mac OS X; en-us) AppleWebKit/537.36.0 (KHTML, like \
                Gecko) Version/4.0.4 Mobile/7B334b Safari/537.36.0"
    )
}

#[test]
fn returns_a_malformed_record_error_for_records_short_on_values() {
    let short_record = "2015-08-15T23:43:05.302180Z elb-name 172.16.1.6:54814 \
    172.16.1.5:9000 0.000039 200 200 0 7582 \
    \"GET http://some.domain.com:80/path0/path1?param0=p0&param1=p1 HTTP/1.1\" \
    ";

    let malformed_error = parse_record(short_record).unwrap_err().errors.pop();

    assert_eq!(
        malformed_error,
        Some(ELBRecordParsingError::MalformedRecord)
    )
}

#[test]
fn returns_a_record_with_the_request_http_version() {
    let elb_record = parse_record(V1_TEST_RECORD).unwrap();

    assert_eq!(elb_record.request_http_version, "HTTP/1.1")
}

#[test]
fn returns_a_record_with_the_request_url() {
    let elb_record = parse_record(V1_TEST_RECORD).unwrap();

    assert_eq!(
        elb_record.request_url,
        "http://some.domain.com:80/path0/path1?param0=p0&param1=p1"
    )
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
fn returns_a_parsing_error_referencing_the_received_bytes_when_the_received_bytes_is_malformed()
{
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

    assert_eq!(
        elb_record.backend_address,
        "172.16.1.5:9000".parse().unwrap()
    )
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

    assert_eq!(
        elb_record.client_address,
        "172.16.1.6:54814".parse().unwrap()
    )
}

#[test]
fn returns_a_parsing_error_referencing_the_client_address_when_the_client_address_is_malformed()
{
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

    assert_eq!(
        format!("{:?}", elb_record.timestamp),
        "2015-08-15T23:43:05.302180Z"
    )
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
