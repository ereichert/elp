#ELP - AWS ELB Access Log Parser

ELP is a simple AWS ELB Access Log parser.  

See the [access log docs](http://docs.aws.amazon.com/ElasticLoadBalancing/latest/DeveloperGuide/access-log-collection.html)
for more information about them.

## Be kind and learn from others.

This project supports the [Rust Code of Conduct](https://www.rust-lang.org/conduct.html).

## [Documentation](http://ereichert.github.io/elp/elp/index.html)

##How To

There is a full example of ELP's use in a production environment [here](https://github.com/trafficland/counter).

Add ELP as a dependency.  Add the following to your `Cargo.toml`:

```toml
[dependencies]
elp = "2.0.0"
```
Then reference it as an external crate in your code.

```rust
extern crate elp;
```

Here's a short (incomplete, probably buggy) program that uses ELP to parse all of the records in a file.

```rust
extern crate elp;

fn main() {
  match File::open(some_path) {
    Ok(file) => {
        for possible_record in BufReader::new(file).lines() {
            // See http://ereichert.github.io/elp/elp/type.ParsingResult.html
            if let Ok(record) = elp::parse_record(possible_record) {
                // handle ELBRecord
            } else {
                // handle ParsingErrors
            }
        };
    }
    Err(err) => {} //handle io::Error
  }
}
```

Most of this is pretty standard Rust code.  The only ELP specific code of note is the elp::parse_record call.

An attempt is made to parse each field independently. The ParsingErrors struct includes a list of the fields that could 
not be parsed and, if possible, the reason they could not be parsed.