extern crate rustc_serialize;
extern crate docopt;
#[macro_use]
extern crate aws_abacus;
#[macro_use]
extern crate log;
extern crate walkdir;
extern crate chrono;
use docopt::Docopt;
use std::path;
use aws_abacus::elb_log_files;
use chrono::{DateTime, UTC};
use aws_abacus::elb_log_files::ELBRecord;
use aws_abacus::elb_log_files::ParsingResult;

fn main() {
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
    match elb_log_files::file_list(log_location, &mut filenames) {
        Ok(num_files) => {
            number_of_files = num_files;
            debug!("Found {} files.", number_of_files);
            number_of_records = elb_log_files::process_files(&filenames, &parsing_result_handler);
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
            println!("Processed {} files having {} records in {} milliseconds.",
                number_of_files,
                number_of_records,
                time.num_milliseconds()
            );
        },
        None => {},
    };
}

fn parsing_result_handler(parsing_result: ParsingResult) -> () {
    //println!("made it to the handler.");
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

    const TEST_RECORD: &'static str = "2015-08-15T23:43:05.302180Z elb-name 172.16.1.6:54814 \
    172.16.1.5:9000 0.000039 0.145507 0.00003 200 200 0 7582 \
    \"GET http://some.domain.com:80/path0/path1?param0=p0&param1=p1 HTTP/1.1\"\
    ";

    #[derive(Clone, Debug, Eq, Hash, PartialEq)]
    struct AggregateELBRecord {
        day: String,
        client_address: String,
        system_name: String,
        public_id: i64,
    }

    //Need to produce system_name,yyyy-mm-dd,req_addr,count
    fn aggregate_record(aggregation: &mut HashMap<AggregateELBRecord, i64>, aggregate_record: AggregateELBRecord) -> () {
        let total = aggregation.entry(aggregate_record).or_insert(0);
        *total += 1;
    }

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

        aggregate_record(&mut agg, ar0);
        aggregate_record(&mut agg, ar1);

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

        aggregate_record(&mut agg, ar0);
        aggregate_record(&mut agg, ar1);

        assert_eq!(agg[&ar3], 2);
	}
}
