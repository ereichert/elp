#ELP - AWS ELB Access Log Parser



### Benchmark runs (using the --benchmark cli option)

02/01/2016 -

Moved to_string() calls to to_owned where appropriate.

Processed 121 files having 9340036 records in 28462 milliseconds.

27/12/2015 -

All of the parsing is done.

Processed 121 files having 9340036 records in 32010 milliseconds.

13/12/2015 -

All of the properties have been migrated to specific types including the
client and backend addresses.

Processed 121 files having 9340036 records in 33014 milliseconds.

13/12/2015 -

Most of the properties that should be specific types have been converted.

Processed 121 files having 9340036 records in 33877 milliseconds.

13/12/2015 -

Upgraded to Rust 1.5.0
Moved two of the ELBRecord properties to their correct types (that is, not String)

Processed 121 files having 9340036 records in 39200 milliseconds.

02/12/2015 -

First version, leaving all of the record fields as String.

Processed 121 files having 9340036 records in 38854 milliseconds.
