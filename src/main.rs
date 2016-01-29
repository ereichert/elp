extern crate rustc_serialize;
extern crate docopt;
#[macro_use]
extern crate aws_abacus;
#[macro_use]
extern crate log;
extern crate walkdir;
extern crate chrono;
extern crate env_logger;
use docopt::Docopt;
use std::path;
use aws_abacus::elb_log_files;
use chrono::{DateTime, UTC};
use aws_abacus::elb_log_files::ParsingResult;
use std::collections::HashMap;

fn main() {
    env_logger::init().unwrap();
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
    let mut agg: HashMap<AggregateELBRecord, i64> = HashMap::new();
    match elb_log_files::file_list(log_location, &mut filenames) {
        Ok(num_files) => {
            number_of_files = num_files;
            debug!("Found {} files.", number_of_files);
            number_of_records = elb_log_files::process_files(&filenames, &mut |parsing_result: ParsingResult| {
                parsing_result_handler(parsing_result, &mut agg);
            });
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
            println!("Processed {} files having {} records in {} milliseconds and produced {} aggregates.",
                number_of_files,
                number_of_records,
                time.num_milliseconds(),
                agg.len()
            );
        },
        None => {},
    };
}


#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct AggregateELBRecord {
    day: String,
    client_address: String,
    system_name: String,
    // public_id: u64,
}

fn parsing_result_handler(parsing_result: ParsingResult, aggregation: &mut HashMap<AggregateELBRecord, i64>) -> () {
    match parsing_result {
        Ok(elb_record) => {
            //36labs,2016-01-07,173.70.188.85,3
            // println!("{}", elb_record.client_address.ip().to_string());
            let aer = AggregateELBRecord {
                day: elb_record.timestamp.format("%Y-%m-%d").to_string(),
                client_address: elb_record.client_address.ip().to_string(),
                system_name: "".to_owned(),
                // public_id: elb_record.sent_bytes
            };
            aggregate_record(aer, aggregation);
        },

        Err(errors) => {

        }
    }
}

//Need to produce system_name,yyyy-mm-dd,req_addr,count
fn aggregate_record(aggregate_record: AggregateELBRecord, aggregation: &mut HashMap<AggregateELBRecord, i64>) -> () {
    let total = aggregation.entry(aggregate_record).or_insert(0);
    *total += 1;
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

#[cfg(test)]
mod tests {

    use std::collections::HashMap;
    use ::AggregateELBRecord;
    use ::aggregate_record;

    const TEST_RECORD: &'static str = "2015-08-15T23:43:05.302180Z elb-name 172.16.1.6:54814 \
    172.16.1.5:9000 0.000039 0.145507 0.00003 200 200 0 7582 \
    \"GET http://some.domain.com:80/path0/path1?param0=p0&param1=p1 HTTP/1.1\"\
    ";

    #[test]
	fn inserting_two_records_with_different_values_creates_two_entries_each_recorded_once() {
        let mut agg: HashMap<AggregateELBRecord, i64> = HashMap::new();

        let ar0 = AggregateELBRecord {
            day: "2015-08-15".to_owned(),
            client_address: "172.16.1.6:54814".to_owned(),
            system_name: "sys1".to_owned(),
            public_id: 8880
        };

        let ar1 = AggregateELBRecord {
            day: "2015-08-15".to_owned(),
            client_address: "172.16.1.6:54814".to_owned(),
            system_name: "sys1".to_owned(),
            public_id: 8888
        };

        aggregate_record(ar0, &mut agg);
        aggregate_record(ar1, &mut agg);

        assert_eq!(agg.len(), 2);
        for (_, total) in agg {
            assert_eq!(total, 1)
        }
	}

    #[test]
	fn inserting_two_records_with_the_same_values_increases_the_total_correctly() {
        let mut agg: HashMap<AggregateELBRecord, i64> = HashMap::new();

        let ar0 = AggregateELBRecord {
            day: "2015-08-15".to_owned(),
            client_address: "172.16.1.6:54814".to_owned(),
            system_name: "sys1".to_owned(),
            public_id: 8888
        };

        let ar1 = ar0.clone();
        let ar3 = ar0.clone();

        aggregate_record(ar0, &mut agg);
        aggregate_record(ar1, &mut agg);

        assert_eq!(agg[&ar3], 2);
	}
}
