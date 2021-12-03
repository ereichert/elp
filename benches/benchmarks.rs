use criterion::{black_box, criterion_group, criterion_main, Criterion};

const TEST_LINE: &str = r#"2015-08-15T23:43:05.302180Z elb-name 172.16.1.6:54814 172.16.1.5:9000 0.000039 0.145507 0.00003 200 200 0 7582 "GET http://some.domain.com:80/path0/path1?param0=p0&param1=p1 HTTP/1.1""#;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("parse_line", |b| b.iter(|| elp::parse_record(black_box(TEST_LINE)).unwrap()));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
