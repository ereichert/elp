### cargo bench runs

01/29/2017 -

Major rewrite of how the state of the record splitter is maintained.

test bench_parse_line ... bench:       1,296 ns/iter (+/- 99)

01/26/2017 -

Minor change to the split_record algorithm

test bench_parse_line ... bench:       1,545 ns/iter (+/- 264)

01/02/2017 -

Upgraded to rustc 1.16.0-nightly (4ecc85beb 2016-12-28)

test bench_parse_line ... bench:       1,775 ns/iter (+/- 283)

02/12/2016 -

Upgraded to Rust 1.7.0 nightly.
Fully implemented algorithm.  Release candidate.

test bench_parse_line ... bench:       2,392 ns/iter (+/- 329)

02/01/2016 -

Moved to_string() calls to to_owned where appropriate.

test bench_parse_line ... bench:       2,097 ns/iter (+/- 348)

27/12/2015 -

The parsing is complete.

test bench_parse_line ... bench:       2,299 ns/iter (+/- 259)

13/12/2015 -

All of the properties have been migrated to specific types including the
client and backend addresses.

test bench_parse_line ... bench:       2,329 ns/iter (+/- 444)

13/12/2015 -

Most of the properties that should be specific types have been converted.

test bench_parse_line ... bench:       2,281 ns/iter (+/- 859)

13/12/2015 -

Upgraded to Rust 1.6.0 nightly
Moved two of the ELBRecord properties to their correct types (that is, not String)

test bench_parse_line ... bench:       2,485 ns/iter (+/- 570)
