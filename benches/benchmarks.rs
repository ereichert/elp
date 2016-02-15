#![feature(test)]
extern crate test;
extern crate aws_abacus;

use test::Bencher;
use aws_abacus::elb_log_files;

const TEST_LINE: &'static str = "2015-08-15T23:43:05.302180Z elb-name 172.16.1.6:54814 \
172.16.1.5:9000 0.000039 0.145507 0.00003 200 200 0 7582 \
\"GET http://some.domain.com:80/path0/path1?param0=p0&param1=p1 HTTP/1.1\"\
";

#[bench]
fn bench_parse_line(b: &mut Bencher) {
    b.iter(|| elb_log_files::parse_record(TEST_LINE.to_string()).unwrap());
}
