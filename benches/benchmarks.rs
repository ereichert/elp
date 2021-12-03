use criterion::{black_box, criterion_group, criterion_main, Criterion};

const TEST_LINE_V1: &str = r#"2015-08-15T23:43:05.302180Z elb-name 172.16.1.6:54814 172.16.1.5:9000 0.000039 0.145507 0.00003 200 200 0 7582 "GET http://some.domain.com:80/path0/path1?param0=p0&param1=p1 HTTP/1.1""#;
const TEST_LINE_V2: &str = r#"2015-08-15T23:43:05.302180Z elb-name 172.16.1.6:54814 172.16.1.5:9000 0.000039 0.145507 0.00003 200 200 0 7582 "GET http://some.domain.com:80/path0/path1?param0=p0&param1=p1 HTTP/1.1" "Mozilla/5.0 (cloud; like Mac OS X; en-us) AppleWebKit/537.36.0 (KHTML, like Gecko) Version/4.0.4 Mobile/7B334b Safari/537.36.0" some_ssl_cipher some_ssl_protocol"#;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("parsing v1 line", |b| b.iter(|| elp::parse_record(black_box(TEST_LINE_V1)).unwrap()));
    c.bench_function("parsing v2 line", |b| b.iter(|| elp::parse_record(black_box(TEST_LINE_V2)).unwrap()));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
